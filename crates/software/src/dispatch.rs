use crate::{
    buffer::{BufferMut, BufferRef},
    pack_rgba,
    util::{Pod, SimdDispatcher, ThreadPool},
    vm::{CompiledShader, VMContext, VMMemory, VMSlot, VMTile16},
};
use bumpalo::{Bump, collections::Vec};
use picodraw_core::{Bounds, DrawError};
use std::{iter::from_fn, ops::Range};

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
    data: Vec<'a, VMSlot>,
    textures: Vec<'a, BufferRef<'a>>,
}

impl<'a> Dispatcher<'a> {
    pub fn new(arena: &'a Bump) -> Self {
        Self {
            arena,
            objects: Vec::new_in(arena),
            data: Vec::new_in(arena),
            textures: Vec::new_in(arena),
        }
    }

    pub fn write_clear(&mut self, bounds: impl Into<Bounds>) {
        self.objects.push(DispatchObject::Clear { bounds: bounds.into() });
    }

    pub fn write_start(&mut self, bounds: impl Into<Bounds>, shader: &'a CompiledShader) {
        self.objects.push(DispatchObject::Draw {
            shader,
            data: self.data.len()..0,
            textures: self.textures.len()..0,
            bounds: bounds.into(),
        });
    }

    pub fn write_data(&mut self, data: &[VMSlot]) {
        self.data.extend_from_slice(data);
    }

    pub fn write_texture(&mut self, texture: BufferRef<'a>) {
        self.textures.push(texture);
    }

    pub fn write_end(&mut self) -> Result<(), DrawError> {
        if let Some(DispatchObject::Draw {
            data, textures, shader, ..
        }) = self.objects.last_mut()
        {
            data.end = self.data.len();
            textures.end = self.textures.len();

            if shader.input_slots() as usize != data.len() {
                return Err(DrawError::MalformedStream);
            }

            if shader.texture_slots() as usize != textures.len() {
                return Err(DrawError::MalformedStream);
            }
        } else {
            return Err(DrawError::MalformedStream);
        }

        Ok(())
    }

    pub fn dispatch(self, pool: &mut ThreadPool, simd: SimdDispatcher, buffer: BufferMut<'a>) {
        // prepare data
        let data_buffer = self.data.into_bump_slice();
        let texture_buffer = self.textures.into_bump_slice();

        // run the "static" parts of the object shaders
        // (i.e. the parts that don't depend on the current pixel)
        let mut memory = VMMemory::new(256, self.arena);
        let jobs = self.objects.iter().map(|object| {
            match object {
                DispatchObject::Draw {
                    shader,
                    data,
                    textures,
                    bounds,
                } => {
                    let data = &data_buffer[data.clone()];
                    let textures = &texture_buffer[textures.clone()];

                    // SAFETY: the program is guaranteed to be valid
                    // because [`CompiledShader::compile`] is expected to return a valid program
                    // data is guaranteed to be valid because we checked it in [`write_end`]
                    let result = unsafe {
                        VMContext {
                            program: shader.static_program(),
                            inputs: &data,
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

                    let data = &*self.arena.alloc_slice_fill_iter(result.iter().copied());

                    &*self.arena.alloc(DispatchJob { object, data })
                }

                object => &*self.arena.alloc(DispatchJob { object, data: &[] }),
            }
        });

        // tile objects into separate buckets
        let tiles_width = buffer.width().div_ceil(TILE_SIZE);
        let tiles_height = buffer.height().div_ceil(TILE_SIZE);
        let tiles = {
            let mut tiles = Vec::from_iter_in(
                from_fn(|| Some(Vec::new_in(self.arena))).take(tiles_width * tiles_height),
                self.arena,
            );

            for job in jobs {
                let bounds = match job.object {
                    DispatchObject::Draw { bounds, .. } => bounds,
                    DispatchObject::Clear { bounds } => bounds,
                };

                let x0 = bounds.left as usize / TILE_SIZE;
                let y0 = bounds.top as usize / TILE_SIZE;
                let x1 = (bounds.right as usize).div_ceil(TILE_SIZE);
                let y1 = (bounds.bottom as usize).div_ceil(TILE_SIZE);

                for y in y0..y1 {
                    for x in x0..x1 {
                        tiles[y * tiles_width + x].push(job);
                    }
                }
            }

            tiles
        };

        // filter empty tiles out and make a list of groups
        let groups = Vec::from_iter_in(
            tiles
                .into_iter()
                .enumerate()
                .filter(|(_, objects)| objects.len() > 0)
                .map(|(i, objects)| {
                    let x = ((i % tiles_width) * TILE_SIZE) as u32;
                    let y = ((i / tiles_width) * TILE_SIZE) as u32;

                    &*self.arena.alloc(DispatchGroup {
                        x,
                        y,
                        objects: objects.into_bump_slice(),
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
                            buffer.subregion_mut(group.x as usize, group.y as usize, TILE_SIZE, TILE_SIZE),
                            buffer.width(),
                            buffer.height(),
                        )
                    };

                    // clear the local buffer
                    worker.r.as_f32_mut().fill(0.0);
                    worker.g.as_f32_mut().fill(0.0);
                    worker.b.as_f32_mut().fill(0.0);
                    worker.a.as_f32_mut().fill(0.0);

                    // draw the objects in sequence
                    for job in group.objects.iter() {
                        match job.object {
                            DispatchObject::Clear { bounds } => {
                                let bounds = bounds.offset(-(group.x as i32), -(group.y as i32)).intersect(Bounds {
                                    top: 0,
                                    left: 0,
                                    bottom: TILE_SIZE as u32,
                                    right: TILE_SIZE as u32,
                                });

                                for j in bounds.top as usize..bounds.bottom as usize {
                                    worker.a.as_f32_mut()[j * TILE_SIZE..][..TILE_SIZE]
                                        [bounds.left as usize..bounds.right as usize]
                                        .fill(0.0);
                                }
                            }

                            DispatchObject::Draw {
                                shader,
                                textures,
                                bounds,
                                ..
                            } => {
                                // SAFETY: the program is guaranteed to be valid
                                // because [`CompiledShader::compile`] is expected to return a valid program
                                // data is guaranteed to be valid because we checked it in [`write_end`]
                                let result = unsafe {
                                    VMContext {
                                        program: shader.dynamic_program(),
                                        inputs: &job.data,
                                        textures: &texture_buffer[textures.clone()],
                                        pos_x: group.x as f32 + 0.5,
                                        pos_y: group.y as f32 + 0.5,
                                        res_x: width as f32,
                                        res_y: height as f32,
                                        quad_t: bounds.top as f32,
                                        quad_l: bounds.left as f32,
                                        quad_b: bounds.bottom as f32,
                                        quad_r: bounds.right as f32,
                                    }
                                    .run(&mut worker.memory)
                                };

                                let bounds = bounds.offset(-(group.x as i32), -(group.y as i32));
                                let r = result.get(0);
                                let g = result.get(1);
                                let b = result.get(2);
                                let a = result.get(3);

                                blend_tile(
                                    &mut worker.r,
                                    &mut worker.g,
                                    &mut worker.b,
                                    &mut worker.a,
                                    r,
                                    g,
                                    b,
                                    a,
                                    bounds,
                                );
                            }
                        }
                    }

                    finish_tile(buffer, &mut worker.r, &mut worker.g, &mut worker.b, &mut worker.a);
                },
            );
        });
    }
}

struct DispatchJob<'a> {
    object: &'a DispatchObject<'a>,
    data: &'a [VMSlot],
}

struct DispatchGroup<'a> {
    x: u32,
    y: u32,
    objects: &'a [&'a DispatchJob<'a>],
}

struct DispatchWorker<'a> {
    r: VMTile16,
    g: VMTile16,
    b: VMTile16,
    a: VMTile16,
    memory: VMMemory<'a>,
}

#[inline(always)]
fn blend_tile(
    r0: &mut VMTile16,
    g0: &mut VMTile16,
    b0: &mut VMTile16,
    a0: &mut VMTile16,
    r1: &VMTile16,
    g1: &VMTile16,
    b1: &VMTile16,
    a1: &VMTile16,
    bounds: Bounds,
) {
    let (a0, r0, g0, b0) = (a0.as_f32_mut(), r0.as_f32_mut(), g0.as_f32_mut(), b0.as_f32_mut());
    let (a1, r1, g1, b1) = (a1.as_f32(), r1.as_f32(), g1.as_f32(), b1.as_f32());

    for i in 0..r0.len() {
        let mask = {
            let x = i % TILE_SIZE;
            let y = i / TILE_SIZE;
            (x >= bounds.left as usize)
                & (x < bounds.right as usize)
                & (y >= bounds.top as usize)
                & (y < bounds.bottom as usize)
        };

        let a1 = if mask { a1[i].clamp(0.0, 1.0) } else { 0.0 };

        a0[i] = (1.0 - a0[i]) * a1 + a0[i];
        r0[i] = (r1[i] - r0[i]) * a1 + r0[i];
        g0[i] = (g1[i] - g0[i]) * a1 + g0[i];
        b0[i] = (b1[i] - b0[i]) * a1 + b0[i];
    }
}

#[inline(always)]
fn finish_tile(mut buffer: BufferMut, r: &mut VMTile16, g: &mut VMTile16, b: &mut VMTile16, a: &mut VMTile16) {
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

    convert_color_0_255(r);
    convert_color_0_255(g);
    convert_color_0_255(b);
    convert_color_0_255(a);

    let (a, r, g, b) = (a.as_f32_mut(), r.as_f32_mut(), g.as_f32_mut(), b.as_f32_mut());
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
