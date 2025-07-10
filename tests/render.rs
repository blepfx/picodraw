pub const MAX_CANVAS_SIZE: u32 = 512;

#[cfg(feature = "opengl")]
pub mod opengl {
    use super::MAX_CANVAS_SIZE;
    use image::{DynamicImage, Rgba, RgbaImage};
    use picodraw::{CommandBuffer, Context, opengl::OpenGlBackend};
    use pugl_rs::{Event, OpenGl, OpenGlVersion, World};
    use std::any::Any;
    use std::panic::{AssertUnwindSafe, resume_unwind};
    use std::time::Duration;
    use std::{
        panic::catch_unwind,
        sync::{
            Arc, Condvar, Mutex,
            atomic::{AtomicBool, Ordering},
        },
    };

    static JOB_QUEUE: Mutex<Vec<Arc<Job>>> = Mutex::new(Vec::new());
    struct Job {
        width: u32,
        height: u32,
        render: Arc<dyn Fn(&mut dyn Context) + Send + Sync>,
        result: Mutex<Option<Result<DynamicImage, Box<dyn Any + Send>>>>,
        condvar: Condvar,
    }

    fn runner_thread() {
        let close = Arc::new(AtomicBool::new(false));
        let mut gl_backend = None;
        let mut world = World::new_program().unwrap();
        let close_send = close.clone();
        let view = world
            .new_view(OpenGl {
                bits_alpha: 8,
                bits_depth: 0,
                bits_stencil: 0,
                version: OpenGlVersion::Core(3, 3),
                debug: true,
                ..Default::default()
            })
            .with_size(MAX_CANVAS_SIZE, MAX_CANVAS_SIZE)
            .with_event_handler(move |view, event| match event {
                Event::Expose { backend, .. } => {
                    let job = match JOB_QUEUE.lock().unwrap().pop() {
                        Some(job) => job,
                        None => {
                            close_send.store(true, Ordering::SeqCst);
                            return;
                        }
                    };

                    let result = catch_unwind(AssertUnwindSafe(|| {
                        let mut gl_backend = unsafe {
                            gl_backend
                                .get_or_insert_with(|| {
                                    OpenGlBackend::new(|c| backend.get_proc_address(c) as *const _).unwrap()
                                })
                                .open()
                        };

                        {
                            let mut commands = CommandBuffer::new();
                            commands.begin_screen([MAX_CANVAS_SIZE, MAX_CANVAS_SIZE]).clear([
                                0,
                                0,
                                MAX_CANVAS_SIZE,
                                MAX_CANVAS_SIZE,
                            ]);
                            gl_backend.draw(&commands);
                        }

                        (job.render)(&mut gl_backend);

                        {
                            let screenshot = gl_backend.screenshot(None, [0, 0, job.width, job.height]);
                            let mut image = RgbaImage::new(job.width, job.height);
                            for i in 0..job.width {
                                for j in 0..job.height {
                                    let r = screenshot[0 + 4 * (i + j * job.width) as usize];
                                    let g = screenshot[1 + 4 * (i + j * job.width) as usize];
                                    let b = screenshot[2 + 4 * (i + j * job.width) as usize];
                                    let a = screenshot[3 + 4 * (i + j * job.width) as usize];
                                    image.put_pixel(i, job.height - 1 - j, Rgba([r, g, b, a]));
                                }
                            }

                            image.into()
                        }
                    }));

                    job.result.lock().unwrap().replace(result);
                    job.condvar.notify_one();
                }
                Event::Update => {
                    view.obscure_view();
                }

                Event::Unrealize { .. } => {
                    if let Some(gl_backend) = gl_backend.take() {
                        unsafe {
                            gl_backend.delete();
                        }
                    }
                }

                _ => {}
            })
            .realize()
            .unwrap();

        view.show_passive();

        while !close.load(Ordering::SeqCst) {
            world.update(Some(Duration::ZERO)).unwrap();
        }
    }

    pub fn render(width: u32, height: u32, render: Arc<dyn Fn(&mut dyn Context) + Send + Sync>) -> DynamicImage {
        let job = Arc::new(Job {
            width,
            height,
            render,
            result: Mutex::new(None),
            condvar: Condvar::new(),
        });

        {
            let mut queue = JOB_QUEUE.lock().unwrap();
            if queue.len() < 3 {
                std::thread::spawn(runner_thread);
            }

            queue.push(job.clone());
        }

        let mut result = job.result.lock().unwrap();
        loop {
            match result.take() {
                Some(Ok(image)) => {
                    return image;
                }
                Some(Err(err)) => {
                    resume_unwind(err);
                }
                None => {}
            }

            result = job.condvar.wait(result).unwrap();
        }
    }
}

#[cfg(feature = "software")]
pub mod software {
    use image::{DynamicImage, Rgba, RgbaImage};
    use picodraw::{
        Context,
        software::{BufferMut, SoftwareBackend},
    };
    use std::sync::Arc;

    pub fn render(width: u32, height: u32, render: Arc<dyn Fn(&mut dyn Context) + Send + Sync>) -> DynamicImage {
        let mut backend = SoftwareBackend::new();
        let mut buffer = vec![0u32; (width * height) as usize];
        let mut context = backend.open(BufferMut::from_slice(&mut buffer, width as usize, height as usize));

        render(&mut context);

        let mut image = RgbaImage::new(width, height);
        for i in 0..width {
            for j in 0..height {
                let data = buffer[(i + j * width) as usize];
                let (r, g, b, a) = picodraw::software::unpack_rgba(data);
                image.put_pixel(i, j, Rgba([r, g, b, a]));
            }
        }

        image.into()
    }
}
