use crate::{
    buffer::{BufferMut, BufferRef},
    util::{ThreadPool, dispatch_simd},
    vm::{CompiledShader, PIXEL_COUNT, TILE_SIZE, VMContext, VMInterpreter, VMSlot},
};
use bumpalo::{Bump, collections::Vec};
use picodraw_core::Bounds;
use std::{iter::from_fn, ops::Range, sync::Mutex};

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

    pub fn write_end(&mut self) {
        if let Some(DispatchObject::Draw {
            data, textures, shader, ..
        }) = self.objects.last_mut()
        {
            data.end = self.data.len();
            textures.end = self.textures.len();

            if shader.data_slots() as usize != data.len() {
                panic!("write_data wrote wrong amount of data for the given shader");
            }

            if shader.texture_slots() as usize != textures.len() {
                panic!("write_texture added wrong amount of textures for the given shader");
            }
        } else {
            panic!("write_end without corresponding write_start");
        }
    }

    pub fn dispatch(self, pool: &mut ThreadPool, buffer: BufferMut<'a>) {
        // prepare data
        let data_buffer = self.data.into_bump_slice();
        let texture_buffer = self.textures.into_bump_slice();
        let (buffer_ptr, width, height, stride) = buffer.into_raw_parts();
        let buffer_ptr = buffer_ptr as usize;

        // tile objects into separate buckets
        let tiles_width = width.div_ceil(TILE_SIZE);
        let tiles_height = height.div_ceil(TILE_SIZE);
        let tiles = {
            let mut tiles = Vec::from_iter_in(
                from_fn(|| Some(Vec::new_in(self.arena))).take(tiles_width * tiles_height),
                self.arena,
            );

            for object in self.objects.iter() {
                let bounds = match object {
                    DispatchObject::Draw { bounds, .. } => bounds,
                    DispatchObject::Clear { bounds } => bounds,
                };

                let x0 = bounds.left as usize / TILE_SIZE;
                let y0 = bounds.top as usize / TILE_SIZE;
                let x1 = (bounds.right as usize).div_ceil(TILE_SIZE);
                let y1 = (bounds.bottom as usize).div_ceil(TILE_SIZE);

                for y in y0..y1 {
                    for x in x0..x1 {
                        tiles[y * tiles_width + x].push(object);
                    }
                }
            }

            tiles
        };

        // filter empty tiles out and make a list of jobs
        let jobs = tiles
            .into_iter()
            .enumerate()
            .filter(|(_, objects)| objects.len() > 0)
            .map(|(i, objects)| {
                let x = ((i % tiles_width) * TILE_SIZE) as u32;
                let y = ((i / tiles_width) * TILE_SIZE) as u32;

                &*self.arena.alloc(DispatchJob {
                    x,
                    y,
                    objects: objects.into_bump_slice(),
                })
            });

        // allocate memory for workers
        let workers = &*self.arena.alloc_slice_fill_iter((0..pool.num_threads()).map(|_| {
            Mutex::new(DispatchWorker {
                r: self.arena.alloc([0.0; PIXEL_COUNT]),
                g: self.arena.alloc([0.0; PIXEL_COUNT]),
                b: self.arena.alloc([0.0; PIXEL_COUNT]),
                a: self.arena.alloc([0.0; PIXEL_COUNT]),
                interpreter: VMInterpreter::new(self.arena),
            })
        }));

        // dispatch jobs
        pool.execute(jobs, |job, index| {
            let worker = &mut *workers[index].lock().unwrap();
            dispatch_simd(
                #[inline(always)]
                || {
                    // SAFETY: the buffer is guaranteed to be valid because
                    // it's alive for the duration of the outer scope,
                    // and we access each region only once
                    // (i.e. threads have no intersecting write regions)
                    let buffer = unsafe {
                        BufferMut::from_raw_parts(buffer_ptr as *mut u32, width, height, stride).subregion_mut(
                            job.x as usize,
                            job.y as usize,
                            TILE_SIZE,
                            TILE_SIZE,
                        )
                    };

                    // clear the buffer
                    worker.r.fill(0.0);
                    worker.g.fill(0.0);
                    worker.b.fill(0.0);
                    worker.a.fill(0.0);

                    // draw the objects in sequence
                    for object in job.objects.iter() {
                        match object {
                            DispatchObject::Clear { bounds } => {
                                let bounds = bounds.offset(-(job.x as i32), -(job.y as i32)).intersect(Bounds {
                                    top: 0,
                                    left: 0,
                                    bottom: TILE_SIZE as u32,
                                    right: TILE_SIZE as u32,
                                });

                                for j in bounds.top as usize..bounds.bottom as usize {
                                    for i in bounds.left as usize..bounds.right as usize {
                                        worker.a[j * TILE_SIZE + i] = 0.0;
                                    }
                                }
                            }

                            DispatchObject::Draw {
                                shader,
                                data,
                                textures,
                                bounds,
                            } => {
                                // SAFETY: the program is guaranteed to be valid
                                // because [`CompiledShader::compile`] is expected to return a valid program
                                // data is guaranteed to be valid because we checked it in [`write_end`]
                                unsafe {
                                    worker.interpreter.execute(VMContext {
                                        ops: shader.opcodes(),
                                        data: &data_buffer[data.clone()],
                                        textures: &texture_buffer[textures.clone()],
                                        pos_x: job.x as f32 + 0.5,
                                        pos_y: job.y as f32 + 0.5,
                                        res_x: width as f32,
                                        res_y: height as f32,
                                        quad_t: bounds.top as f32,
                                        quad_l: bounds.left as f32,
                                        quad_b: bounds.bottom as f32,
                                        quad_r: bounds.right as f32,
                                    });
                                }

                                let bounds = bounds.offset(-(job.x as i32), -(job.y as i32));
                                let r = worker.interpreter.register(shader.output_register(0)).as_f32();
                                let g = worker.interpreter.register(shader.output_register(1)).as_f32();
                                let b = worker.interpreter.register(shader.output_register(2)).as_f32();
                                let a = worker.interpreter.register(shader.output_register(3)).as_f32();

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

                    finish_tile(buffer, worker.r, worker.g, worker.b, worker.a);
                },
            );
        });
    }
}

struct DispatchJob<'a> {
    x: u32,
    y: u32,
    objects: &'a [&'a DispatchObject<'a>],
}

struct DispatchWorker<'a> {
    r: &'a mut [f32; PIXEL_COUNT],
    g: &'a mut [f32; PIXEL_COUNT],
    b: &'a mut [f32; PIXEL_COUNT],
    a: &'a mut [f32; PIXEL_COUNT],
    interpreter: VMInterpreter<'a>,
}

#[inline(always)]
fn blend_tile(
    r0: &mut [f32; PIXEL_COUNT],
    g0: &mut [f32; PIXEL_COUNT],
    b0: &mut [f32; PIXEL_COUNT],
    a0: &mut [f32; PIXEL_COUNT],
    r1: &[f32; PIXEL_COUNT],
    g1: &[f32; PIXEL_COUNT],
    b1: &[f32; PIXEL_COUNT],
    a1: &[f32; PIXEL_COUNT],
    bounds: Bounds,
) {
    for i in 0..PIXEL_COUNT {
        let mask = {
            let x = i % TILE_SIZE;
            let y = i / TILE_SIZE;
            (x >= bounds.left as usize)
                & (x < bounds.right as usize)
                & (y >= bounds.top as usize)
                & (y < bounds.bottom as usize)
        };

        let a1 = if mask { a1[i] } else { 0.0 };
        a0[i] = a0[i] + ((a1 - a0[i]) * a1);
        r0[i] = r0[i] + ((r1[i] - r0[i]) * a1);
        g0[i] = g0[i] + ((g1[i] - g0[i]) * a1);
        b0[i] = b0[i] + ((b1[i] - b0[i]) * a1);
    }
}

#[inline(always)]
fn finish_tile(
    mut buffer: BufferMut,
    r: &mut [f32; PIXEL_COUNT],
    g: &mut [f32; PIXEL_COUNT],
    b: &mut [f32; PIXEL_COUNT],
    a: &mut [f32; PIXEL_COUNT],
) {
    #[inline(always)]
    fn convert_color_0_255(x: &mut [f32; PIXEL_COUNT]) {
        #[cold]
        fn cold() {}

        for i in 0..PIXEL_COUNT {
            if x[i] == x[i] {
                x[i] = x[i].clamp(0.0, 1.0) * 255.0;
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

    for j in 0..TILE_SIZE.min(buffer.height()) {
        for i in 0..TILE_SIZE.min(buffer.width()) {
            unsafe {
                let r = r[j * TILE_SIZE + i].to_int_unchecked::<u8>();
                let g = g[j * TILE_SIZE + i].to_int_unchecked::<u8>();
                let b = b[j * TILE_SIZE + i].to_int_unchecked::<u8>();
                let a = a[j * TILE_SIZE + i].to_int_unchecked::<u8>();
                buffer[(i, j)] = u32::from_le_bytes([b, g, r, a]);
            }
        }
    }
}
