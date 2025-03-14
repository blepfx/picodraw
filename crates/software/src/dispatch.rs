use crate::{
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
        bounds: Bounds,
    },

    Clear {
        bounds: Bounds,
    },
}

pub struct DispatchBuffer<'a> {
    pub buffer: &'a mut [u32],
    pub width: u32,
    pub height: u32,
    pub bounds: Bounds,
}

pub struct Dispatcher<'a> {
    arena: &'a Bump,
    buffer: DispatchBuffer<'a>,
    objects: Vec<'a, DispatchObject<'a>>,
    data: Vec<'a, VMSlot>,
}

impl<'a> Dispatcher<'a> {
    pub fn new(arena: &'a Bump, buffer: DispatchBuffer<'a>) -> Self {
        Self {
            arena,
            buffer,
            objects: Vec::new_in(arena),
            data: Vec::new_in(arena),
        }
    }

    pub fn write_clear(&mut self, bounds: impl Into<Bounds>) {
        self.objects.push(DispatchObject::Clear {
            bounds: bounds.into(),
        });
    }

    pub fn write_start(&mut self, bounds: impl Into<Bounds>, shader: &'a CompiledShader) {
        self.objects.push(DispatchObject::Draw {
            shader,
            data: self.data.len()..0,
            bounds: bounds.into(),
        });
    }

    pub fn write_data(&mut self, data: &[VMSlot]) {
        self.data.extend_from_slice(data);
    }

    pub fn write_end(&mut self) {
        if let Some(DispatchObject::Draw { data, shader, .. }) = self.objects.last_mut() {
            data.end = self.data.len();
            if shader.data_slots() as usize != data.len() {
                panic!("write_data wrote insufficient amount of data for the given shader");
            }
        } else {
            panic!("write_end without corresponding write_start");
        }
    }

    pub fn dispatch(self, pool: &mut ThreadPool) {
        // prepare data
        let slots = self.data.into_bump_slice();
        let width = self.buffer.width;
        let height = self.buffer.height;
        let bounds = self.buffer.bounds;
        let buffer_ptr = self.buffer.buffer.as_mut_ptr() as usize;
        drop(self.buffer);

        // tile objects into separate buckets
        let tiles_width = (bounds.width().div_ceil(TILE_SIZE as u32)) as usize;
        let tiles_height = (bounds.height().div_ceil(TILE_SIZE as u32)) as usize;
        let tiles = {
            let mut tiles = Vec::from_iter_in(
                from_fn(|| Some(Vec::new_in(self.arena))).take(tiles_width * tiles_height),
                self.arena,
            );

            for object in self.objects.iter() {
                let bounds = match object {
                    DispatchObject::Draw { bounds, .. } => {
                        bounds.offset(-(bounds.left as i32), -(bounds.top as i32))
                    }
                    DispatchObject::Clear { bounds } => {
                        bounds.offset(-(bounds.left as i32), -(bounds.top as i32))
                    }
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
        let workers = &*self
            .arena
            .alloc_slice_fill_iter((0..pool.num_threads()).map(|_| {
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
                    // clear the buffer
                    worker.r.fill(0.0);
                    worker.g.fill(0.0);
                    worker.b.fill(0.0);
                    worker.a.fill(0.0);

                    // draw the objects in sequence
                    for object in job.objects.iter() {
                        match object {
                            DispatchObject::Clear { bounds } => {
                                let bounds = bounds.offset(-(job.x as i32), -(job.y as i32));
                                for j in bounds.top as usize..bounds.bottom as usize {
                                    for i in bounds.left as usize..bounds.right as usize {
                                        worker.a[j * TILE_SIZE + i] = 0.0;
                                    }
                                }
                            }

                            DispatchObject::Draw {
                                shader,
                                data,
                                bounds,
                            } => {
                                // SAFETY: the program is guaranteed to be valid
                                // because [`CompiledShader::compile`] is expected to return a valid program
                                // data is guaranteed to be valid because we checked it in [`write_end`]
                                unsafe {
                                    worker.interpreter.execute(VMContext {
                                        ops: shader.opcodes(),
                                        data: &slots[data.clone()],
                                        tile_x: job.x as f32,
                                        tile_y: job.y as f32,
                                        quad_t: bounds.top as f32,
                                        quad_l: bounds.left as f32,
                                        quad_b: bounds.bottom as f32,
                                        quad_r: bounds.right as f32,
                                        res_x: width as f32,
                                        res_y: height as f32,
                                    });
                                }

                                let bounds = bounds.offset(-(job.x as i32), -(job.y as i32));
                                let r = worker
                                    .interpreter
                                    .register(shader.output_register(0))
                                    .as_f32();
                                let g = worker
                                    .interpreter
                                    .register(shader.output_register(1))
                                    .as_f32();
                                let b = worker
                                    .interpreter
                                    .register(shader.output_register(2))
                                    .as_f32();
                                let a = worker
                                    .interpreter
                                    .register(shader.output_register(3))
                                    .as_f32();

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

                    // copy the worker memory to the global buffer
                    // SAFETY: the buffer is guaranteed to be valid,
                    // and we write to each region only once
                    // (i.e. threads have no intersecting write regions)
                    unsafe {
                        finish_tile(
                            buffer_ptr as *mut u32,
                            width,
                            height,
                            worker.r,
                            worker.g,
                            worker.b,
                            worker.a,
                            job.x,
                            job.y,
                        );
                    }
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
unsafe fn finish_tile(
    dst: *mut u32,
    width: u32,
    height: u32,
    r: &mut [f32; PIXEL_COUNT],
    g: &mut [f32; PIXEL_COUNT],
    b: &mut [f32; PIXEL_COUNT],
    a: &mut [f32; PIXEL_COUNT],
    x: u32,
    y: u32,
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

    for j in 0..(TILE_SIZE as u32).min(height - y) {
        for i in 0..(TILE_SIZE as u32).min(width - x) {
            let tx = x + i;
            let ty = y + j;

            unsafe {
                let r = r[(j * TILE_SIZE as u32 + i) as usize].to_int_unchecked::<u32>();
                let g = g[(j * TILE_SIZE as u32 + i) as usize].to_int_unchecked::<u32>();
                let b = b[(j * TILE_SIZE as u32 + i) as usize].to_int_unchecked::<u32>();
                let a = a[(j * TILE_SIZE as u32 + i) as usize].to_int_unchecked::<u32>();
                *dst.add((ty * width + tx) as usize) = a << 24 | r << 16 | g << 8 | b;
            }
        }
    }
}
