use crate::{
    simd::dispatch,
    vm::{CompiledShader, PIXEL_COUNT, TILE_SIZE, VMInterpreter, VMProgram, VMSlot, VMTile},
};
use bumpalo::{Bump, collections::Vec};
use picodraw_core::Bounds;
use std::{iter::from_fn, ops::Range};

pub enum BufferType {
    Rgba8Row,
    Argb8Row,
    Abrg8Row,
    Brga8Row,
    Rgba8Column,
    Argb8Column,
    Abrg8Column,
    Brga8Column,
}

struct DispatchObject<'a> {
    shader: &'a CompiledShader,
    data: Range<usize>,
    bounds: Bounds,
}

pub struct DispatchBuffer<'a> {
    pub buffer: &'a mut [u32],
    pub width: u32,
    pub height: u32,
    pub bounds: Bounds,
}

pub struct Dispatcher<'a> {
    arena: &'a Bump,
    objects: Vec<'a, DispatchObject<'a>>,
    data: Vec<'a, VMSlot>,
}

impl<'a> Dispatcher<'a> {
    pub fn new(arena: &'a Bump) -> Self {
        Self {
            arena,
            objects: Vec::new_in(arena),
            data: Vec::new_in(arena),
        }
    }

    pub fn write_start(&mut self, bounds: impl Into<Bounds>, shader: &'a CompiledShader) {
        self.objects.push(DispatchObject {
            shader,
            data: self.data.len()..0,
            bounds: bounds.into(),
        });
    }

    pub fn write_data(&mut self, data: &[VMSlot]) {
        self.data.extend_from_slice(data);
    }

    pub fn write_end(&mut self) {
        self.objects
            .last_mut()
            .expect("write_end without corresponding write_start")
            .data
            .end = self.data.len();
    }

    pub fn dispatch(self, buffer: DispatchBuffer) {
        // prepare data
        let data = self.data.into_bump_slice();
        let width = buffer.width;
        let height = buffer.height;
        let bounds = buffer.bounds;
        let buffer_ptr = buffer.buffer.as_mut_ptr() as usize;
        drop(buffer);

        // tile objects into separate buckets
        let tiles_width = (bounds.width().div_ceil(TILE_SIZE as u32)) as usize;
        let tiles_height = (bounds.height().div_ceil(TILE_SIZE as u32)) as usize;
        let tiles = {
            let mut tiles = Vec::from_iter_in(
                from_fn(|| Some(Vec::new_in(self.arena))).take(tiles_width * tiles_height),
                self.arena,
            );

            for object in self.objects.iter() {
                let bounds = object
                    .bounds
                    .offset(-(bounds.left as i32), -(bounds.top as i32));
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

                DispatchJob {
                    x,
                    y,
                    objects: objects.into_bump_slice(),
                }
            });

        // dispatch jobs
        dispatch_jobs(self.arena, jobs, |job, worker| {
            dispatch(
                #[inline(always)]
                || {
                    // clear the buffer
                    worker.r.fill(0.0);
                    worker.g.fill(0.0);
                    worker.b.fill(0.0);
                    worker.a.fill(0.0);

                    // draw the objects in sequence
                    for object in job.objects.iter() {
                        // SAFETY: the program is guaranteed to be valid
                        // because [`CompiledShader::compile`] is expected to return a valid program
                        unsafe {
                            worker.interpreter.execute(VMProgram {
                                ops: object.shader.opcodes(),
                                data: &data[object.data.clone()],
                                tile_x: job.x as f32,
                                tile_y: job.y as f32,
                                quad_t: object.bounds.top as f32,
                                quad_l: object.bounds.left as f32,
                                quad_b: object.bounds.bottom as f32,
                                quad_r: object.bounds.right as f32,
                                res_x: width as f32,
                                res_y: height as f32,
                            });
                        }

                        let bounds = object.bounds.offset(-(job.x as i32), -(job.y as i32));
                        let r = worker
                            .interpreter
                            .register(object.shader.output_register(0))
                            .as_f32();
                        let g = worker
                            .interpreter
                            .register(object.shader.output_register(1))
                            .as_f32();
                        let b = worker
                            .interpreter
                            .register(object.shader.output_register(2))
                            .as_f32();
                        let a = worker
                            .interpreter
                            .register(object.shader.output_register(3))
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
    interpreter: VMInterpreter<'a, VMTile>,
}

#[inline(always)]
#[cfg(not(feature = "parallel"))]
fn dispatch_jobs<'a>(
    arena: &'a Bump,
    jobs: impl Iterator<Item = DispatchJob<'a>>,
    run: impl Fn(&DispatchJob, &mut DispatchWorker) + Send + Sync,
) {
    let mut worker = DispatchWorker {
        r: arena.alloc([0.0; PIXEL_COUNT]),
        g: arena.alloc([0.0; PIXEL_COUNT]),
        b: arena.alloc([0.0; PIXEL_COUNT]),
        a: arena.alloc([0.0; PIXEL_COUNT]),
        interpreter: VMInterpreter::new(arena),
    };

    for job in jobs {
        run(&job, &mut worker);
    }
}

#[inline(always)]
#[cfg(feature = "parallel")]
fn dispatch_jobs<'a>(
    arena: &'a Bump,
    jobs: impl Iterator<Item = DispatchJob<'a>>,
    run: impl Fn(&DispatchJob, &mut DispatchWorker) + Send + Sync,
) {
    use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
    use std::sync::Mutex;

    let workers = &*arena.alloc_slice_fill_iter((0..rayon::current_num_threads()).map(|_| {
        Mutex::new(DispatchWorker {
            r: arena.alloc([0.0; PIXEL_COUNT]),
            g: arena.alloc([0.0; PIXEL_COUNT]),
            b: arena.alloc([0.0; PIXEL_COUNT]),
            a: arena.alloc([0.0; PIXEL_COUNT]),
            interpreter: VMInterpreter::new(arena),
        })
    }));

    let jobs = Vec::from_iter_in(jobs, arena);

    jobs.par_iter().for_each(|job| {
        let worker = &mut *workers[rayon::current_thread_index().unwrap()]
            .lock()
            .unwrap();
        run(job, worker);
    });
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

pub unsafe fn finish_tile(
    dst: *mut u32,
    width: u32,
    height: u32,
    r: &[f32; PIXEL_COUNT],
    g: &[f32; PIXEL_COUNT],
    b: &[f32; PIXEL_COUNT],
    a: &[f32; PIXEL_COUNT],
    x: u32,
    y: u32,
) {
    for j in 0..(TILE_SIZE as u32).min(height - y) {
        for i in 0..(TILE_SIZE as u32).min(width - x) {
            let tx = x + i;
            let ty = y + j;

            let r = r[(j * TILE_SIZE as u32 + i) as usize];
            let g = g[(j * TILE_SIZE as u32 + i) as usize];
            let b = b[(j * TILE_SIZE as u32 + i) as usize];
            let a = a[(j * TILE_SIZE as u32 + i) as usize];

            unsafe {
                *dst.add((ty * width + tx) as usize) = ((a * 255.0) as u32) << 24
                    | ((r * 255.0) as u32) << 16
                    | ((g * 255.0) as u32) << 8
                    | ((b * 255.0) as u32);
            }
        }
    }
}
