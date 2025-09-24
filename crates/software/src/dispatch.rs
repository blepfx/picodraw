use crate::{
    buffer::{BufferMut, BufferRef},
    pack_rgba,
    util::{Pod, SimdDispatcher, SparseMap, ThreadPool},
    vm::{CompiledShader, VMContext, VMMemory, VMSlot, VMTile, VMTile4, VMTile8, VMTile16},
};
use bumpalo::{Bump, collections::Vec};
use picodraw_core::{Bounds, DrawError};
use std::{hint::unreachable_unchecked, ops::Range};

const TILE_SIZE: usize = 16;

enum DispatchObject<'a> {
    Draw {
        shader: &'a CompiledShader,
        data: Range<usize>,
        textures: Range<usize>,
        bounds: Bounds,
    },

    Clear {
        bounds: Bounds,
    },
}

pub struct Dispatcher<'a> {
    arena: &'a Bump,
    objects: Vec<'a, DispatchObject<'a>>,
    inputs: Vec<'a, VMSlot>,
    textures: Vec<'a, BufferRef<'a>>,
}

impl<'a> Dispatcher<'a> {
    pub fn new(arena: &'a Bump) -> Self {
        Self {
            arena,
            objects: Vec::new_in(arena),
            inputs: Vec::new_in(arena),
            textures: Vec::new_in(arena),
        }
    }

    pub fn write_clear(&mut self, bounds: impl Into<Bounds>) {
        self.objects.push(DispatchObject::Clear { bounds: bounds.into() });
    }

    pub fn write_start(&mut self, bounds: impl Into<Bounds>, shader: &'a CompiledShader) {
        self.objects.push(DispatchObject::Draw {
            shader,
            data: self.inputs.len()..0,
            textures: self.textures.len()..0,
            bounds: bounds.into(),
        });
    }

    pub fn write_data(&mut self, data: &[VMSlot]) {
        self.inputs.extend_from_slice(data);
    }

    pub fn write_texture(&mut self, texture: BufferRef<'a>) {
        self.textures.push(texture);
    }

    pub fn write_end(&mut self) -> Result<(), DrawError> {
        if let Some(DispatchObject::Draw {
            data, textures, shader, ..
        }) = self.objects.last_mut()
        {
            data.end = self.inputs.len();
            textures.end = self.textures.len();

            if shader.input_slots() as usize != data.len() {
                return Err(DrawError::InvalidQuadData);
            }

            if shader.texture_slots() as usize != textures.len() {
                return Err(DrawError::InvalidQuadData);
            }
        } else {
            return Err(DrawError::MalformedStream);
        }

        Ok(())
    }

    pub fn dispatch(self, pool: &mut ThreadPool, simd: SimdDispatcher, buffer: BufferMut<'a>) {
        // prepare data
        let input_buffer = self.inputs.into_bump_slice();
        let texture_buffer = self.textures.into_bump_slice();

        // run the "static" parts of the object shaders
        // (i.e. the parts that don't depend on the current pixel)
        // and tile them into buckets
        let mut memory = VMMemory::new(256, self.arena);
        let jobs = self.objects.iter().map(|object| {
            match object {
                DispatchObject::Draw {
                    shader,
                    data,
                    textures,
                    bounds,
                } => {
                    let inputs = &input_buffer[data.clone()];
                    let textures = &texture_buffer[textures.clone()];

                    // SAFETY: the program is guaranteed to be valid
                    // because [`CompiledShader::compile`] is expected to return a valid program
                    // data is guaranteed to be valid because we checked it in [`write_end`]
                    let result = unsafe {
                        VMContext {
                            program: shader.static_program(),
                            inputs: &inputs,
                            textures: &textures,
                            pos_x: 0.0,
                            pos_y: 0.0,
                            res_x: buffer.width() as f32,
                            res_y: buffer.height() as f32,
                            quad_t: bounds.top as f32,
                            quad_l: bounds.left as f32,
                            quad_b: bounds.bottom as f32,
                            quad_r: bounds.right as f32,
                        }
                        .run::<VMSlot>(&mut memory)
                    };

                    let inputs = &*self.arena.alloc_slice_fill_iter(result.iter().copied());
                    DispatchOperation::Draw {
                        shader,
                        inputs,
                        textures,
                        bounds: *bounds,
                    }
                }

                DispatchObject::Clear { bounds } => DispatchOperation::Clear { bounds: *bounds },
            }
        });

        // tile objects into separate buckets
        let tiles = {
            let mut tiles = SparseMap::new(
                buffer.width().div_ceil(TILE_SIZE) as u32,
                buffer.height().div_ceil(TILE_SIZE) as u32,
                self.arena,
            );

            for job in jobs {
                let bounds = match job {
                    DispatchOperation::Draw { bounds, .. } => bounds,
                    DispatchOperation::Clear { bounds } => bounds,
                };

                let x0 = bounds.left / TILE_SIZE as u32;
                let y0 = bounds.top / TILE_SIZE as u32;
                let x1 = bounds.right.div_ceil(TILE_SIZE as u32).min(tiles.width());
                let y1 = bounds.bottom.div_ceil(TILE_SIZE as u32).min(tiles.height());

                let job = &*self.arena.alloc(job);
                for y in y0..y1 {
                    for x in x0..x1 {
                        tiles.push(x, y, job);
                    }
                }
            }

            tiles
        };

        // filter empty tiles out and make a list of groups
        let groups = Vec::from_iter_in(
            tiles.into_iter().map(|(x, y, objects)| {
                &*self.arena.alloc(DispatchGroup {
                    x: x * TILE_SIZE as u32,
                    y: y * TILE_SIZE as u32,
                    ops: objects.into_bump_slice(),
                })
            }),
            self.arena,
        )
        .into_bump_slice();

        // allocate memory for workers
        let workers = self
            .arena
            .alloc_slice_fill_iter((0..pool.num_workers()).map(|_| DispatchWorker {
                r: VMTile16::zeroed(),
                g: VMTile16::zeroed(),
                b: VMTile16::zeroed(),
                a: VMTile16::zeroed(),
                memory: VMMemory::new(256 * 64, self.arena),
            }));

        // dispatch groups

        pool.run_arrays(workers, groups, |worker, group| {
            simd.dispatch(
                #[inline(always)]
                || {
                    // SAFETY: the buffer is guaranteed to be valid because
                    // it's alive for the duration of the outer scope,
                    // and we access each region only once
                    // (i.e. threads have no intersecting read-write regions)
                    let (buffer, width, height) = unsafe {
                        let mut buffer = std::ptr::read::<BufferMut<'_>>(&buffer as *const _);

                        (
                            buffer.subregion_mut(
                                group.x as usize,
                                group.y as usize,
                                TILE_SIZE as usize,
                                TILE_SIZE as usize,
                            ),
                            buffer.width(),
                            buffer.height(),
                        )
                    };

                    worker.reset_tile();

                    // draw the objects in sequence
                    for job in group.ops.iter().copied() {
                        let bounds = job
                            .bounds()
                            .offset(-(group.x as i32), -(group.y as i32))
                            .intersect(Bounds {
                                top: 0,
                                left: 0,
                                bottom: TILE_SIZE as u32,
                                right: TILE_SIZE as u32,
                            });

                        match job {
                            DispatchOperation::Clear { .. } => {
                                worker.clear_region(
                                    bounds.left as usize,
                                    bounds.top as usize,
                                    bounds.right as usize,
                                    bounds.bottom as usize,
                                );
                            }

                            DispatchOperation::Draw {
                                shader,
                                inputs,
                                textures,
                                ..
                            } => unsafe {
                                worker.draw_region(
                                    VMContext {
                                        program: shader.dynamic_program(),
                                        inputs,
                                        textures,
                                        pos_x: group.x as f32 + 0.5,
                                        pos_y: group.y as f32 + 0.5,
                                        res_x: width as f32,
                                        res_y: height as f32,
                                        quad_t: bounds.top as f32,
                                        quad_l: bounds.left as f32,
                                        quad_b: bounds.bottom as f32,
                                        quad_r: bounds.right as f32,
                                    },
                                    bounds,
                                );
                            },
                        }
                    }

                    worker.finish_tile(buffer);
                },
            );
        });
    }
}

enum DispatchOperation<'a> {
    Draw {
        shader: &'a CompiledShader,
        textures: &'a [BufferRef<'a>],
        inputs: &'a [VMSlot],
        bounds: Bounds,
    },

    Clear {
        bounds: Bounds,
    },
}

impl<'a> DispatchOperation<'a> {
    fn bounds(&self) -> Bounds {
        match self {
            Self::Draw { bounds, .. } => *bounds,
            Self::Clear { bounds } => *bounds,
        }
    }
}

struct DispatchGroup<'a> {
    x: u32,
    y: u32,
    ops: &'a [&'a DispatchOperation<'a>],
}

struct DispatchWorker<'a> {
    r: VMTile16,
    g: VMTile16,
    b: VMTile16,
    a: VMTile16,
    memory: VMMemory<'a>,
}

impl<'a> DispatchWorker<'a> {
    #[inline(always)]
    fn clear_region(&mut self, x0: usize, y0: usize, x1: usize, y1: usize) {
        for j in y0..y1 {
            self.a.as_f32_mut()[j * TILE_SIZE..][..TILE_SIZE][x0..x1].fill(0.0);
        }
    }

    #[inline(always)]
    unsafe fn draw_region(&mut self, context: VMContext, bounds: Bounds) {
        if bounds.contains([0, 0, 16, 16]) {
            unsafe {
                self.draw_region_subtile::<VMTile16>(context, 0, 0, 0, 0, 16, 16);
            }
        } else {
            for i in 0..4u32 {
                let (x0, y0) = ((i % 2) * 8, (i / 2) * 8);
                if bounds.contains([x0, y0, x0 + 8, y0 + 8]) {
                    unsafe {
                        self.draw_region_subtile::<VMTile8>(context, x0 as usize, y0 as usize, 0, 0, 8, 8);
                    }
                } else {
                    for i in 0..4u32 {
                        let (x1, y1) = (x0 + (i % 2) * 4, y0 + (i / 2) * 4);
                        let bounds = bounds.intersect([x1, y1, x1 + 4, y1 + 4]);
                        if !bounds.is_empty() {
                            unsafe {
                                self.draw_region_subtile::<VMTile4>(
                                    context,
                                    x1 as usize,
                                    y1 as usize,
                                    (bounds.left - x1) as usize,
                                    (bounds.top - y1) as usize,
                                    bounds.width() as usize,
                                    bounds.height() as usize,
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    #[inline(always)]
    unsafe fn draw_region_subtile<T: VMTile>(
        &mut self,
        mut context: VMContext,

        tile_x: usize,
        tile_y: usize,

        mask_x: usize,
        mask_y: usize,
        mask_w: usize,
        mask_h: usize,
    ) {
        // SAFETY: the program is guaranteed to be valid
        // because [`CompiledShader::compile`] is expected to return a valid program
        // data is guaranteed to be valid because we checked it in [`write_end`]
        let result = unsafe {
            context.pos_x += tile_x as f32;
            context.pos_y += tile_y as f32;
            context.run::<T>(&mut self.memory)
        };

        let r = result.get(0);
        let g = result.get(1);
        let b = result.get(2);
        let a = result.get(3);

        // blend the tile in
        {
            let (a0, r0, g0, b0) = (
                self.a.as_f32_mut(),
                self.r.as_f32_mut(),
                self.g.as_f32_mut(),
                self.b.as_f32_mut(),
            );
            let (a1, r1, g1, b1) = (a.as_f32(), r.as_f32(), g.as_f32(), b.as_f32());

            for j in 0..mask_h {
                for i in 0..mask_w {
                    let src = (j + mask_y) * T::WIDTH + (i + mask_x);
                    let dst = (j + tile_y + mask_y) << 4 | (i + tile_x + mask_x);

                    if dst >= a0.len() || src >= a1.len() {
                        unsafe {
                            unreachable_unchecked();
                        }
                    }

                    let a1 = a1[src].clamp(0.0, 1.0);
                    a0[dst] = (1.0 - a0[dst]) * a1 + a0[dst];
                    r0[dst] = (r1[src] - r0[dst]) * a1 + r0[dst];
                    g0[dst] = (g1[src] - g0[dst]) * a1 + g0[dst];
                    b0[dst] = (b1[src] - b0[dst]) * a1 + b0[dst];
                }
            }
        }
    }

    #[inline(always)]
    fn reset_tile(&mut self) {
        self.r.as_f32_mut().fill(0.0);
        self.g.as_f32_mut().fill(0.0);
        self.b.as_f32_mut().fill(0.0);
        self.a.as_f32_mut().fill(0.0);
    }

    #[inline(always)]
    fn finish_tile(&mut self, mut buffer: BufferMut) {
        #[inline(always)]
        fn convert_color_0_255(x: &mut VMTile16) {
            #[cold]
            fn cold() {}

            let x = x.as_f32_mut();
            for i in 0..x.len() {
                if x[i] == x[i] {
                    x[i] = (x[i] * 255.0 + 0.5).clamp(0.0, 255.0);
                } else {
                    cold();
                    x[i] = 0.0;
                }
            }
        }

        convert_color_0_255(&mut self.r);
        convert_color_0_255(&mut self.g);
        convert_color_0_255(&mut self.b);
        convert_color_0_255(&mut self.a);

        let (a, r, g, b) = (
            self.a.as_f32_mut(),
            self.r.as_f32_mut(),
            self.g.as_f32_mut(),
            self.b.as_f32_mut(),
        );
        for j in 0..TILE_SIZE.min(buffer.height()) {
            for i in 0..TILE_SIZE.min(buffer.width()) {
                unsafe {
                    let r = r[j * TILE_SIZE + i].to_int_unchecked::<u8>();
                    let g = g[j * TILE_SIZE + i].to_int_unchecked::<u8>();
                    let b = b[j * TILE_SIZE + i].to_int_unchecked::<u8>();
                    let a = a[j * TILE_SIZE + i].to_int_unchecked::<u8>();
                    buffer[(i, j)] = pack_rgba(r, g, b, a);
                }
            }
        }
    }
}
