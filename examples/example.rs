use baseview::{
    gl::GlConfig, Event, EventStatus, Size, Window, WindowEvent, WindowHandler, WindowOpenOptions,
    WindowScalePolicy,
};
use picodraw::{opengl::OpenGl, Float2, Float4, GlFloat, Shader, ShaderData};
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

            self.gl.render(
                self.size.width.ceil() as u32,
                self.size.height.ceil() as u32,
                |mut render| {
                    render.draw(&Circle {
                        center: [256.0, 256.0],
                        radius: 100.0,
                    });
                },
            );

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
}

impl Shader for Circle {
    fn id() -> &'static str {
        "circle"
    }

    fn bounds(&self) -> [f32; 4] {
        [0.0, 0.0, 1000.0, 1000.0]
    }

    fn draw(pos: Float2, vars: Self::ShaderVars) -> Float4 {
        let center = Float2::new(vars.center[0], vars.center[1]);
        let mask = 1.0 - ((center - pos).len() - vars.radius).smoothstep(-0.707, 0.707);

        Float4::from(1.0) * mask
    }
}
