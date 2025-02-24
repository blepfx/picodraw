use picodraw::{
    CommandBuffer, Context, Graph, Shader, ShaderData,
    opengl::OpenGlBackend,
    shader::{float1, float2, float4, io},
};
use pugl_rs::{Event, OpenGl, OpenGlVersion, World};
use std::time::Duration;
use std::time::Instant;

struct Data {
    gl: OpenGlBackend,
    shader: Shader,
    scroll: f32,
    avg_time: f32,
}

#[derive(ShaderData)]
struct ShaderDataCircle {
    x: f32,
    y: f32,
    radius: f32,
}

fn main() {
    let mut data = None;
    let mut world = World::new_program().unwrap();
    let view = world
        .new_view(OpenGl {
            version: OpenGlVersion::Core(3, 3),
            bits_alpha: 0,
            bits_depth: 0,
            bits_stencil: 0,
            ..Default::default()
        })
        .with_title("picodraw opengl example")
        .with_size(512, 512)
        .with_event_handler(move |view, event| match event {
            Event::Expose { backend, .. } => {
                let start = Instant::now();

                // SAFETY: there's a current OpenGL context because we are inside of the Expose event
                let data = data.get_or_insert_with(|| unsafe {
                    let mut gl = OpenGlBackend::new(&|c| backend.get_proc_address(c)).unwrap();
                    let shader = gl.open().create_shader(Graph::collect(|| {
                        fn sdf_circle(pos: float2, center: float2, radius: float1) -> float1 {
                            ((center - pos).len() - radius).smoothstep(0.707, -0.707)
                        }

                        let circle = ShaderDataCircle::read();
                        let mask =
                            sdf_circle(io::position(), float2((circle.x, circle.y)), circle.radius);
                        io::write_color(float4((1.0, 0.5, 1.0, mask * 0.2)));
                    }));

                    Data {
                        gl,
                        shader,
                        scroll: 0.0,
                        avg_time: 0.0,
                    }
                });

                let mut commands = CommandBuffer::default();
                let mut frame = commands.begin_screen([512, 512]);
                frame.clear([0, 0, 512, 512]);

                for i in 0..10 {
                    let angle = (i as f32 * 0.1 + data.scroll * 0.05) * std::f32::consts::PI * 2.0;
                    let x = 256.0 + angle.cos() * 200.0;
                    let y = 256.0 + angle.sin() * 200.0;

                    frame
                        .begin_quad(data.shader, [0, 0, 512, 512])
                        .write_data(ShaderDataCircle {
                            x,
                            y,
                            radius: 200.0,
                        });
                }

                // SAFETY: there's a current OpenGL context because we are inside of the Expose event
                unsafe {
                    data.gl.open().draw(&commands);
                }

                let t = start.elapsed();
                data.avg_time = t.as_secs_f32() * 0.02 + data.avg_time * 0.98;
                data.scroll += 1.0 / 60.0;

                println!(
                    "{:?}ms avg, {:?}ms real, {:?}fps",
                    data.avg_time * 1000.0,
                    t.as_secs_f32() * 1000.0,
                    1.0 / data.avg_time
                );
            }

            Event::Scroll { dx, dy, .. } => {
                if let Some(data) = data.as_mut() {
                    data.scroll += (dx + dy) as f32 * 0.25;
                }
            }

            Event::Close => {
                std::process::exit(0);
            }

            Event::Update => {
                view.obscure_view();
            }

            _ => {}
        })
        .realize()
        .unwrap();

    view.show();

    loop {
        world.update(Some(Duration::from_millis(16))).unwrap();
    }
}
