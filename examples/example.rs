use baseview::{
    gl::GlConfig, Event, EventStatus, Size, Window, WindowEvent, WindowHandler, WindowOpenOptions,
    WindowScalePolicy,
};
use picodraw::{
    opengl::{OpenGl, OpenGlConfig},
    Bounds, Float, Float2, Float4, GlFloat, GlLoopVars, Int, Shader, ShaderContext, ShaderData,
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

            let mut gl = OpenGl::new(
                &|c| context.get_proc_address(c.to_str().unwrap()),
                OpenGlConfig { srgb: true },
            );
            gl.render(0, 0, |mut x| {
                x.register::<Circle>();
                x.register::<Pixel>();
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
                    for i in 0..512 {
                        for j in 0..512 {
                            let even = (i + j) % 2 == 0;

                            render.draw(
                                &Pixel {
                                    r: 1.0,
                                    g: if even { 0.0 } else { 1.0 },
                                    b: if even { 1.0 } else { 0.0 },
                                    a: 1.0,
                                    _extra: [0; 12],
                                },
                                Bounds {
                                    top: i,
                                    left: j,
                                    bottom: i + 1,
                                    right: j + 1,
                                },
                            );
                        }
                    }
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
pub struct Unit;

#[derive(ShaderData)]
pub struct Tuple(pub f32);

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

        let i = Int::from(0);
        let f = Float::from(0.0);
        let (_, f) = (i, f).run_loop(|(i, _)| i.lt(10), |(i, f)| (i + 1, f + 0.05));

        Float4::from(shader.alpha)
            * mask
            * f
            * Float::select(Float::from(1.0), Float::from(0.0), true)
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

#[derive(ShaderData)]
struct Pixel {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
    pub _extra: [u8; 12],
}

impl Shader for Pixel {
    fn draw(shader: ShaderContext<Self::ShaderVars>) -> Float4 {
        Float4::new(shader.r, shader.g, shader.b, shader.a)
    }
}
