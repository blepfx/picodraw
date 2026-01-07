use image::{DynamicImage, GenericImage, GenericImageView, Rgba, open};
use picodraw::{Context, dynamic::DynContext};
use std::{
    fs::{create_dir_all, remove_file},
    sync::Arc,
    time::{Duration, Instant},
};
use yansi::Paint;

pub const MAX_CANVAS_SIZE: u32 = 512;

const MAX_P90_ERROR: f64 = 0.05;
const MAX_P95_ERROR: f64 = 1.20;
const MAX_P99_ERROR: f64 = 2.00;

#[track_caller]
pub fn run(test: &str, width: u32, height: u32, render: impl Fn(&mut dyn DynContext) + Sync + Send + 'static) {
    if cfg!(miri) {
        #[cfg(feature = "software")]
        software::render(width, height, Arc::new(render));
        return;
    }

    let renderer = Arc::new(render);
    let expected = open(format!("./tests/drawtest/expected/{}.webp", test)).ok();

    #[allow(clippy::type_complexity)]
    let results: Vec<(&'static str, fn(u32, u32, RenderJob) -> DynamicImage)> = vec![
        #[cfg(feature = "software")]
        ("software", software::render),
        #[cfg(feature = "opengl")]
        ("opengl", opengl::render),
    ];

    let outcomes = results
        .into_iter()
        .map(|(backend, render)| {
            let image = render(width, height, renderer.clone());

            let expected = match expected.as_ref() {
                Some(x) => x,
                None => {
                    return Outcome::NotFound { test, backend, image };
                }
            };

            if expected.width() != image.width() || expected.height() != image.height() {
                return Outcome::Resolution {
                    test,
                    backend,
                    expected: (expected.width(), expected.height()),
                    rendered: (image.width(), image.height()),
                    image,
                };
            }

            let (p50, p95, p99) = measure_difference(expected, &image);
            if p50 > MAX_P90_ERROR || p95 > MAX_P95_ERROR || p99 > MAX_P99_ERROR {
                let diff = blend_difference(expected, &image);

                return Outcome::Difference {
                    test,
                    backend,
                    p50,
                    p95,
                    p99,
                    image,
                    diff,
                };
            }

            Outcome::Passed { test, backend, image }
        })
        .collect::<Vec<_>>();

    let mut messages = vec![];
    for outcome in outcomes {
        match outcome {
            Outcome::Passed { test, backend, image } => {
                write_image(test, backend, SaveImage::Success(&image));
            }
            Outcome::NotFound { test, backend, image } => {
                write_image(test, backend, SaveImage::Failure(&image));

                messages.push(format!(
                    "{}{} {} {} - {}",
                    "❌ ".mask(),
                    "[FAILED]".yellow().bold(),
                    backend.cyan(),
                    test,
                    "no test image found in ./drawtest/expected/".dim()
                ));

                messages.push(format!(
                    " | {} {}",
                    "result has been saved as".dim(),
                    format!("./tests/drawtest/failures/{}/{}.webp", backend, test).bold()
                ));
            }
            Outcome::Resolution {
                test,
                backend,
                expected,
                rendered,
                image,
            } => {
                write_image(test, backend, SaveImage::Failure(&image));

                messages.push(format!(
                    "{}{} {} {} - {}",
                    "❌ ".mask(),
                    "[FAILED]".yellow().bold(),
                    backend.cyan(),
                    test,
                    "resolution does not match".dim()
                ));

                messages.push(format!(
                    " | expected resolution: {}x{}",
                    expected.0.cyan().bold(),
                    expected.1.cyan().bold(),
                ));

                messages.push(format!(
                    " | rendered resolution: {}x{}",
                    rendered.0.cyan().bold(),
                    rendered.1.cyan().bold(),
                ));

                messages.push(format!(
                    " | {} {}",
                    "result has been saved as".dim(),
                    format!("./tests/drawtest/failures/{}/{}.webp", backend, test).bold()
                ));
            }
            Outcome::Difference {
                test,
                backend,
                p50: p90,
                p95,
                p99,
                image,
                diff,
            } => {
                write_image(test, backend, SaveImage::Difference(&image, &diff));

                messages.push(format!(
                    "{}{} {} {} - {}",
                    "❌ ".mask(),
                    "[FAILED]".red().bold(),
                    backend.cyan(),
                    test,
                    "rendered image differs from what is expected".dim()
                ));

                messages.push(format!(" | [{}, {}, {}]", "90th", "95th", "99th"));

                macro_rules! perc {
                    ($p:ident, $m:ident) => {
                        if $p > $m {
                            format!("{}", format!("{:4}", $p).bold().red())
                        } else {
                            format!("{}", format!("{:4}", $p).dim())
                        }
                    };
                }

                messages.push(format!(
                    " | [{}, {}, {}]",
                    perc!(p90, MAX_P99_ERROR),
                    perc!(p95, MAX_P95_ERROR),
                    perc!(p99, MAX_P99_ERROR),
                ));

                messages.push(format!(
                    " | {} {}",
                    "result has been saved as".dim(),
                    format!("./tests/drawtest/failures/{}/{}.webp", backend, test).bold()
                ));

                messages.push(format!(
                    " | {} {}",
                    "delta has been saved as".dim(),
                    format!("./tests/drawtest/failures/{}/{}@diff.webp", backend, test).bold()
                ));
            }
        }
    }

    if !messages.is_empty() {
        panic!("\n{}", messages.join("\n"))
    }
}

type RenderJob = Arc<dyn Fn(&mut dyn DynContext) + Send + Sync>;

enum Outcome<'a> {
    Passed {
        test: &'a str,
        backend: &'a str,
        image: DynamicImage,
    },

    NotFound {
        test: &'a str,
        backend: &'a str,
        image: DynamicImage,
    },

    Resolution {
        test: &'a str,
        backend: &'a str,

        expected: (u32, u32),
        rendered: (u32, u32),

        image: DynamicImage,
    },

    Difference {
        test: &'a str,
        backend: &'a str,

        p50: f64,
        p95: f64,
        p99: f64,

        image: DynamicImage,
        diff: DynamicImage,
    },
}

enum SaveImage<'a> {
    Success(&'a DynamicImage),
    Failure(&'a DynamicImage),
    Difference(&'a DynamicImage, &'a DynamicImage),
}

fn write_image(test: &str, backend: &str, image: SaveImage) {
    create_dir_all(format!("./tests/drawtest/successes/{}/", backend)).ok();
    create_dir_all(format!("./tests/drawtest/failures/{}/", backend)).ok();

    match image {
        SaveImage::Success(image) => {
            image
                .save(format!("./tests/drawtest/successes/{}/{}.webp", backend, test))
                .unwrap();
            remove_file(format!("./tests/drawtest/failures/{}/{}.webp", backend, test)).ok();
            remove_file(format!("./tests/drawtest/failures/{}/{}@diff.webp", backend, test)).ok();
        }
        SaveImage::Failure(image) => {
            image
                .save(format!("./tests/drawtest/failures/{}/{}.webp", backend, test))
                .unwrap();
            remove_file(format!("./tests/drawtest/successes/{}/{}.webp", backend, test)).ok();
            remove_file(format!("./tests/drawtest/failures/{}/{}@diff.webp", backend, test)).ok();
        }
        SaveImage::Difference(image, diff) => {
            image
                .save(format!("./tests/drawtest/failures/{}/{}.webp", backend, test))
                .unwrap();
            diff.save(format!("./tests/drawtest/failures/{}/{}@diff.webp", backend, test))
                .unwrap();
            remove_file(format!("./tests/drawtest/successes/{}/{}.webp", backend, test)).ok();
        }
    }
}

fn measure_difference(a: &DynamicImage, b: &DynamicImage) -> (f64, f64, f64) {
    let mut samples = Vec::with_capacity((a.width() * a.height()) as usize);

    for i in 0..a.width() {
        for j in 0..a.height() {
            let Rgba([r0, g0, b0, a0]) = a.get_pixel(i, j);
            let Rgba([r1, g1, b1, a1]) = b.get_pixel(i, j);

            let mut sum = 0.0;
            sum += (r0 as f64 * a0 as f64 - r1 as f64 * a1 as f64).abs() / (255.0 * 255.0);
            sum += (g0 as f64 * a0 as f64 - g1 as f64 * a1 as f64).abs() / (255.0 * 255.0);
            sum += (b0 as f64 * a0 as f64 - b1 as f64 * a1 as f64).abs() / (255.0 * 255.0);
            sum += (a0 as f64 - a1 as f64).abs();
            samples.push(sum);
        }
    }

    samples.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let p50 = samples
        .get((samples.len() as f64 * 0.90).floor() as usize)
        .copied()
        .unwrap_or_default();
    let p95 = samples
        .get((samples.len() as f64 * 0.95).floor() as usize)
        .copied()
        .unwrap_or_default();
    let p99 = samples
        .get((samples.len() as f64 * 0.99).floor() as usize)
        .copied()
        .unwrap_or_default();
    (p50, p95, p99)
}

fn blend_difference(a: &DynamicImage, b: &DynamicImage) -> DynamicImage {
    let mut image = DynamicImage::new(a.width(), b.height(), a.color());

    for i in 0..a.width() {
        for j in 0..a.height() {
            let Rgba([r0, g0, b0, _]) = a.get_pixel(i, j);
            let Rgba([r1, g1, b1, _]) = b.get_pixel(i, j);

            image.put_pixel(i, j, Rgba([r0.abs_diff(r1), g0.abs_diff(g1), b0.abs_diff(b1), 255]));
        }
    }

    image
}

#[cfg(feature = "opengl")]
pub mod opengl {
    use super::{MAX_CANVAS_SIZE, RenderJob};
    use image::{DynamicImage, Rgba, RgbaImage};
    use picodraw::{Color, Context, DrawTarget, dynamic::DynContext, opengl};
    use picoview::{Event, GlConfig, GlVersion, WindowBuilder};
    use std::any::Any;
    use std::panic::{AssertUnwindSafe, resume_unwind};
    use std::sync::mpsc::{Sender, SyncSender};
    use std::time::Duration;
    use std::{
        panic::catch_unwind,
        sync::{
            Arc, Condvar, Mutex,
            atomic::{AtomicBool, Ordering},
        },
    };

    static USE_TBO: AtomicBool = AtomicBool::new(true);
    static JOB_QUEUE: Mutex<Vec<Arc<Job>>> = Mutex::new(Vec::new());

    struct Job {
        width: u32,
        height: u32,
        render: RenderJob,
        sender: SyncSender<Result<DynamicImage, Box<dyn Any + Send>>>,
    }

    fn runner_thread() {
        WindowBuilder::new(|window| {
            let mut gl_backend: Option<opengl::Backend<opengl::Native>> = None;
            Box::new(move |event| {
                if let Event::WindowFrame { gl: Some(gl) } = event {
                    let job = match JOB_QUEUE.lock().unwrap().pop() {
                        Some(job) => job,
                        None => {
                            if gl.make_current(true)
                                && let Some(gl_backend) = gl_backend.take()
                            {
                                unsafe {
                                    gl_backend.delete();
                                }
                            }

                            window.close();
                            return;
                        }
                    };

                    let result = catch_unwind(AssertUnwindSafe(|| {
                        if !gl.make_current(true) {
                            panic!("failed to make opengl context current");
                        }

                        let mut gl_backend = unsafe {
                            gl_backend
                                .get_or_insert_with(|| {
                                    opengl::Backend::new(
                                        opengl::Config {
                                            enable_gpu_time_queries: true,
                                            prefer_ubo_over_tbo: USE_TBO.fetch_not(Ordering::SeqCst),
                                            debug_logger: Some(Box::new(|type_, msg| {
                                                if type_ == opengl::DebugMessage::Error
                                                    || type_ == opengl::DebugMessage::UndefinedBehavior
                                                {
                                                    panic!("opengl error: {}", msg);
                                                } else {
                                                    eprintln!("opengl debug: {}", msg);
                                                }
                                            })),
                                        },
                                        |c| gl.get_proc_address(c) as *const _,
                                    )
                                    .unwrap()
                                })
                                .open()
                        };

                        {
                            gl_backend.set_viewport([MAX_CANVAS_SIZE, MAX_CANVAS_SIZE]);
                            gl_backend.draw(DrawTarget::Screen, |encoder| {
                                encoder.clear([0, 0, MAX_CANVAS_SIZE, MAX_CANVAS_SIZE].into(), Color::default());
                            });

                            gl_backend.set_viewport([job.width, job.height]);
                            (job.render)(&mut gl_backend);
                        }

                        let image = {
                            let screenshot = gl_backend.screenshot(None, [0, 0, job.width, job.height]);
                            let mut image = RgbaImage::new(job.width, job.height);
                            for i in 0..job.width {
                                for j in 0..job.height {
                                    let r = screenshot[4 * (i + j * job.width) as usize];
                                    let g = screenshot[4 * (i + j * job.width) as usize + 1];
                                    let b = screenshot[4 * (i + j * job.width) as usize + 2];
                                    let a = screenshot[4 * (i + j * job.width) as usize + 3];
                                    image.put_pixel(i, job.height - 1 - j, Rgba([r, g, b, a]));
                                }
                            }

                            image.into()
                        };

                        gl.swap_buffers();
                        image
                    }));

                    job.sender.send(result).ok();
                }
            })
        })
        .with_size((MAX_CANVAS_SIZE, MAX_CANVAS_SIZE))
        .with_title("picodraw drawtest runner")
        .with_opengl(GlConfig {
            version: GlVersion::Core(4, 6),
            debug: true,
            ..Default::default()
        })
        .open_blocking()
        .unwrap();
    }

    pub fn render(width: u32, height: u32, render: RenderJob) -> DynamicImage {
        let (sender, receiver) = std::sync::mpsc::sync_channel(0);
        let job = Arc::new(Job {
            width,
            height,
            render,
            sender,
        });

        {
            let mut queue = JOB_QUEUE.lock().unwrap();
            if queue.len() < 3 {
                std::thread::spawn(runner_thread);
            }

            queue.push(job.clone());
        }

        match receiver.recv().unwrap() {
            Ok(image) => image,
            Err(err) => resume_unwind(err),
        }
    }
}

#[cfg(feature = "software")]
pub mod software {
    use super::RenderJob;
    use image::{DynamicImage, Rgba, RgbaImage};
    use picodraw::{
        Color,
        software::{BufferMut, SoftwareBackend},
    };

    pub fn render(width: u32, height: u32, render: RenderJob) -> DynamicImage {
        let mut backend = SoftwareBackend::with_max_parallelism();
        let mut buffer = vec![Color::default(); (width * height) as usize];
        let mut context = backend.open(BufferMut::from_slice(&mut buffer, width as usize, height as usize));

        render(&mut context);

        let mut image = RgbaImage::new(width, height);
        for i in 0..width {
            for j in 0..height {
                let data = buffer[(i + j * width) as usize];
                image.put_pixel(i, j, Rgba([data.r, data.g, data.b, data.a]));
            }
        }

        image.into()
    }
}
