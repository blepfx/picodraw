use crate::{
    buffer::{BufferMut, BufferRef},
    util::{Pod, SimdDispatcher, SparseMap, ThreadPool},
    vm::{CompiledShader, VMContext, VMMemory, VMSlot, VMTile, VMTile4, VMTile16},
};
use bumpalo::{Bump, collections::Vec};
use picodraw_core::{Bounds, Color};
use std::hint::unreachable_unchecked;

/// The size of a tile in pixels.
pub const TILE_SIZE: usize = 16;

enum DispatchObject<'a> {
    Draw {
        shader: &'a CompiledShader,
        inputs: &'a [VMSlot],
        textures: &'a [BufferRef<'a>],
        bounds: Bounds,
    },

    Fill {
        bounds: Bounds,
        color: Option<Color>,
    },
}

impl DispatchObject<'_> {
    fn bounds(&self) -> &Bounds {
        match self {
            DispatchObject::Draw { bounds, .. } => bounds,
            DispatchObject::Fill { bounds, .. } => bounds,
        }
    }
}

pub struct Dispatcher<'a> {
    arena: &'a Bump,
    objects: Vec<'a, DispatchObject<'a>>,
    current_data: Vec<'a, u8>,
    current_textures: Vec<'a, BufferRef<'a>>,
    current_bounds: Vec<'a, Bounds>,
}

impl<'a> Dispatcher<'a> {
    pub fn new(arena: &'a Bump) -> Self {
        Self {
            arena,
            objects: Vec::new_in(arena),
            current_data: Vec::new_in(arena),
            current_textures: Vec::new_in(arena),
            current_bounds: Vec::new_in(arena),
        }
    }

    pub fn push_clear(&mut self, bounds: impl Into<Bounds>, color: Option<Color>) {
        self.objects.push(DispatchObject::Fill {
            bounds: bounds.into(),
            color,
        });
    }

    pub fn push_object(&mut self, shader: &'a CompiledShader) {
        // TODO: optimize
        let inputs = {
            let mut inputs = Vec::new_in(self.arena);
            for chunk in self.current_data.chunks(4) {
                #[allow(clippy::get_first)]
                inputs.push(VMSlot::from(i32::from_ne_bytes([
                    *chunk.get(0).unwrap_or(&0),
                    *chunk.get(1).unwrap_or(&0),
                    *chunk.get(2).unwrap_or(&0),
                    *chunk.get(3).unwrap_or(&0),
                ])));
            }

            inputs
        };

        let inputs = inputs.into_bump_slice();
        let textures = self.current_textures.clone().into_bump_slice();
        for bounds in self.current_bounds.drain(..) {
            self.objects.push(DispatchObject::Draw {
                shader,
                inputs,
                textures,
                bounds,
            });
        }

        self.current_bounds.clear();
        self.current_data.clear();
        self.current_textures.clear();
    }

    pub fn push_object_rect(&mut self, bounds: Bounds) {
        self.current_bounds.push(bounds);
    }

    pub fn push_object_data(&mut self, data: &[u8]) {
        self.current_data.extend_from_slice(data);
    }

    pub fn push_object_texture(&mut self, texture: BufferRef<'a>) {
        self.current_textures.push(texture);
    }

    pub fn rasterize(self, pool: &mut ThreadPool, simd: SimdDispatcher, buffer: BufferMut<'a>) {
        // run the "static" parts of the object shaders
        // (i.e. the parts that don't depend on the current pixel)
        // and tile them into buckets
        let mut memory = VMMemory::new(256, self.arena);

        let jobs = self.objects.iter().map(|object| {
            match object {
                DispatchObject::Draw {
                    shader,
                    inputs,
                    textures,
                    bounds,
                } => {
                    // SAFETY: the program is guaranteed to be valid
                    // because [`CompiledShader::compile`] is expected to return a valid program
                    let result = unsafe {
                        VMContext {
                            program: shader.static_program(),
                            inputs,
                            textures,
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

                    DispatchObject::Draw {
                        inputs: &*self.arena.alloc_slice_fill_iter(result.iter().copied()),
                        bounds: *bounds,
                        shader,
                        textures,
                    }
                }

                DispatchObject::Fill { bounds, color } => DispatchObject::Fill {
                    bounds: *bounds,
                    color: *color,
                },
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
                let bounds = job.bounds();
                let x0 = bounds.left / TILE_SIZE as u32;
                let y0 = bounds.top / TILE_SIZE as u32;
                let x1 = bounds.right.div_ceil(TILE_SIZE as u32).min(tiles.width());
                let y1 = bounds.bottom.div_ceil(TILE_SIZE as u32).min(tiles.height());

                // If the job is a clear operation, we can optimize by
                // removing any previous jobs in the affected tiles.
                // This is because a clear operation overwrites everything
                // in the region, so previous operations are redundant.
                if let DispatchObject::Fill { bounds, .. } = job {
                    let x0 = bounds.left.div_ceil(TILE_SIZE as u32);
                    let y0 = bounds.top.div_ceil(TILE_SIZE as u32);
                    let x1 = (bounds.right / TILE_SIZE as u32).min(tiles.width());
                    let y1 = (bounds.bottom / TILE_SIZE as u32).min(tiles.height());

                    for y in y0..y1 {
                        for x in x0..x1 {
                            tiles.clear(x, y);
                        }
                    }
                }

                // Allocate the job to get a small pointer
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
            .alloc_slice_fill_iter((0..pool.num_workers()).map(|_| DispatchWorker::new(self.arena)));

        // dispatch groups
        pool.run_arrays(workers, groups, |worker, group| {
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

            simd.dispatch(
                #[inline(always)]
                || {
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
                            DispatchObject::Fill { color: None, .. } => {
                                worker.fill_undefined();
                            }
                            DispatchObject::Fill { color: Some(color), .. } => {
                                worker.fill_region(
                                    bounds.left as usize,
                                    bounds.top as usize,
                                    bounds.right as usize,
                                    bounds.bottom as usize,
                                    *color,
                                );
                            }

                            DispatchObject::Draw {
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
                        };
                    }

                    worker.finish_tile(buffer);
                },
            );
        });
    }
}

struct DispatchGroup<'a> {
    x: u32,
    y: u32,
    ops: &'a [&'a DispatchObject<'a>],
}

struct DispatchWorker<'a> {
    r: VMTile16,
    g: VMTile16,
    b: VMTile16,
    a: VMTile16,
    content: DispatchContent,
    memory: VMMemory<'a>,
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum DispatchContent {
    Zero,    // all transparent, can blend without lerp
    Screen,  // not yet materialized screen content, needs to read from screen first
    Storage, // materialized content, needs full alpha blending
}

impl<'a> DispatchWorker<'a> {
    fn new(arena: &'a Bump) -> Self {
        Self {
            r: VMTile16::zeroed(),
            g: VMTile16::zeroed(),
            b: VMTile16::zeroed(),
            a: VMTile16::zeroed(),
            content: DispatchContent::Zero,
            memory: VMMemory::new(256 * 64, arena),
        }
    }

    fn fill_undefined(&mut self) {
        self.content = DispatchContent::Zero;
    }

    #[inline(always)]
    fn fill_region(&mut self, x0: usize, y0: usize, x1: usize, y1: usize, color: Color) {
        if color.a == 0 {
            self.content = DispatchContent::Zero;

            for j in y0..y1 {
                self.a.as_f32_mut()[j * TILE_SIZE..][..TILE_SIZE][x0..x1].fill(0.0);
            }
        } else {
            self.content = DispatchContent::Storage;

            let r = color.r as f32 / 255.0;
            let g = color.g as f32 / 255.0;
            let b = color.b as f32 / 255.0;
            let a = color.a as f32 / 255.0;

            for j in y0..y1 {
                self.r.as_f32_mut()[j * TILE_SIZE..][..TILE_SIZE][x0..x1].fill(r);
                self.g.as_f32_mut()[j * TILE_SIZE..][..TILE_SIZE][x0..x1].fill(g);
                self.b.as_f32_mut()[j * TILE_SIZE..][..TILE_SIZE][x0..x1].fill(b);
                self.a.as_f32_mut()[j * TILE_SIZE..][..TILE_SIZE][x0..x1].fill(a);
            }
        }
    }

    #[inline(always)]
    unsafe fn draw_region(&mut self, context: VMContext, bounds: Bounds) {
        let width = bounds.width();
        let height = bounds.height();

        // at least 25% occupancy
        if (width > 8 || height > 8) && self.memory.slots::<VMTile16>() >= context.program.used_registers() {
            unsafe {
                self.draw_region_subtile::<VMTile16>(
                    context,
                    0,
                    0,
                    bounds.left as usize,
                    bounds.top as usize,
                    width as usize,
                    height as usize,
                );
            }
        } else {
            for i in (bounds.left..bounds.right).step_by(4) {
                for j in (bounds.top..bounds.bottom).step_by(4) {
                    unsafe {
                        self.draw_region_subtile::<VMTile4>(
                            context,
                            i as usize,
                            j as usize,
                            0,
                            0,
                            (bounds.right - i).min(4) as usize,
                            (bounds.bottom - j).min(4) as usize,
                        );
                    }
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
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

            assert!(mask_x + mask_w <= T::WIDTH);
            assert!(mask_y + mask_h <= T::HEIGHT);

            for j in 0..mask_h {
                for i in 0..mask_w {
                    let src = (j + mask_y) * T::WIDTH + (i + mask_x);
                    let dst = (j + tile_y + mask_y) << 4 | (i + tile_x + mask_x);

                    if dst >= a0.len() || src >= a1.len() {
                        unsafe {
                            unreachable_unchecked();
                        }
                    }

                    let a1 = 1.0 - a1[src].clamp(0.0, 1.0);
                    a0[dst] = (1.0 - a1) + a0[dst] * a1;
                    r0[dst] = r1[src] + r0[dst] * a1;
                    g0[dst] = g1[src] + g0[dst] * a1;
                    b0[dst] = b1[src] + b0[dst] * a1;
                }
            }
        }
    }

    fn reset_tile(&mut self) {
        self.content = DispatchContent::Screen;
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
            for x in x.iter_mut() {
                if !x.is_nan() {
                    *x = (*x * 255.0 + 0.5).clamp(0.0, 255.0);
                } else {
                    cold();
                    *x = 0.0;
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
                    buffer[(i, j)] = Color {
                        r: r[j * TILE_SIZE + i].to_int_unchecked::<u8>(),
                        g: g[j * TILE_SIZE + i].to_int_unchecked::<u8>(),
                        b: b[j * TILE_SIZE + i].to_int_unchecked::<u8>(),
                        a: a[j * TILE_SIZE + i].to_int_unchecked::<u8>(),
                    };
                }
            }
        }
    }
}
