use baseview::{
    gl::GlConfig, Event, EventStatus, Size, Window, WindowEvent, WindowHandler, WindowOpenOptions,
    WindowScalePolicy,
};
use picodraw::{
    opengl::OpenGl, Bounds, Float, Float2, Float4, GlFloat, Shader, ShaderContext, ShaderData,
};
use std::time::Instant;

fn main() {
    let options = WindowOpenOptions {
        title: "picodraw".into(),
        size: Size::new(512.0, 512.0),
        scale: WindowScalePolicy::SystemScaleFactor,

        gl_config: Some(GlConfig {
            version: (3, 3),
            alpha_bits: 0,
            depth_bits: 0,
            ..GlConfig::default()
        }),
    };

    Window::open_blocking(options, |x| App::new(x))
}

struct App {
    gl: OpenGl,
    size: Size,
    avg_time: f32,
}

impl App {
    fn new(window: &mut Window) -> Self {
        unsafe {
            let context = window.gl_context().unwrap();
            context.make_current();

            let mut gl = OpenGl::new(&|c| context.get_proc_address(c.to_str().unwrap()));
            gl.render(0, 0, |mut x| {
                x.register::<Circle>();
            });

            context.make_not_current();

            Self {
                avg_time: 0.0,
                gl,
                size: Size::new(512.0, 512.0),
            }
        }
    }
}

impl WindowHandler for App {
    fn on_frame(&mut self, window: &mut Window) {
        let start = Instant::now();

        unsafe {
            let context = window.gl_context().unwrap();
            context.make_current();

            let stats = self.gl.render(
                self.size.width.ceil() as u32,
                self.size.height.ceil() as u32,
                |mut render| {
                    render.draw(
                        &Circle {
                            center: [256.0, 256.0],
                            radius: 100.0,

                            ignored: 0,
                            alpha: 0.5,
                        },
                        Bounds {
                            left: 256.0 - 100.0,
                            top: 256.0 - 100.0,
                            bottom: 256.0 + 100.0,
                            right: 256.0 + 100.0,
                        },
                    );

                    render.draw(
                        &Circle {
                            center: [320.0, 320.0],
                            radius: 50.0,

                            ignored: 0,
                            alpha: 0.5,
                        },
                        Bounds {
                            left: 320.0 - 50.0,
                            top: 320.0 - 50.0,
                            bottom: 320.0 + 50.0,
                            right: 320.0 + 50.0,
                        },
                    );
                },
            );

            println!("{:?}", stats);

            context.swap_buffers();
            context.make_not_current();
        }

        let t = start.elapsed();
        self.avg_time = t.as_secs_f32() * 0.02 + self.avg_time * 0.98;
        println!(
            "{:?}ms, {:?}ms real, {:?}fps",
            self.avg_time * 1000.0,
            t.as_secs_f32() * 1000.0,
            1.0 / self.avg_time
        );
    }

    fn on_event(&mut self, _window: &mut Window, event: Event) -> EventStatus {
        if let Event::Window(WindowEvent::Resized(size)) = event {
            self.size = size.logical_size();
        }

        EventStatus::Ignored
    }
}

#[derive(ShaderData)]
pub struct Circle {
    pub center: [f32; 2],
    pub radius: f32,

    #[shader(ignore)]
    pub ignored: u64,

    #[shader(F16_01)]
    pub alpha: f32,
}

impl Shader for Circle {
    fn draw(shader: ShaderContext<Self::ShaderVars>) -> Float4 {
        let center = Float2::new(shader.center[0], shader.center[1]);
        let mask =
            1.0 - ((center - shader.position).len() - shader.radius).smoothstep(-0.707, 0.707);

        Float4::from(shader.alpha) * mask
    }
}

/// An example custom data encoder implementation for encoding a float as a 16 bit fixed point number between 0 and 1
struct F16_01(pub f32);

impl ShaderData for F16_01 {
    type ShaderVars = Float;

    fn shader_vars(vars: &mut dyn picodraw::ShaderVars) -> Self::ShaderVars {
        Float::from(vars.read_uint16()) / 65535.0
    }

    fn write(&self, writer: &mut dyn picodraw::ShaderDataWriter) {
        writer.write_int((self.0 * 65535.0) as u16 as i32);
    }
}

impl From<f32> for F16_01 {
    fn from(value: f32) -> Self {
        Self(value)
    }
}
