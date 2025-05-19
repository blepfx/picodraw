use picodraw::{
    CommandBuffer, Context, Graph, Shader,
    opengl::{Native, OpenGlBackend},
    shader::{float1, float4, io},
};
use pugl_rs::{Event, OpenGl, OpenGlVersion, World};
use std::time::Duration;

struct Data {
    gl: OpenGlBackend<Native>,
    shader: Shader,
    width: u32,
    height: u32,
    avg_time_ms: f32,
}

fn main() {
    let mut data: Option<Data> = None;
    let mut world = World::new_program().unwrap();
    let view = world
        .new_view(OpenGl {
            version: OpenGlVersion::Core(3, 0),
            bits_alpha: 0,
            bits_depth: 0,
            bits_stencil: 0,
            ..Default::default()
        })
        .with_title("picodraw opengl example")
        .with_size(512, 512)
        .with_resizable(true)
        .with_event_handler(move |view, event| match event {
            Event::Configure { rect, .. } => {
                if let Some(data) = data.as_mut() {
                    data.width = rect.w;
                    data.height = rect.h;
                }
            }

            Event::Expose { backend, .. } => {
                // SAFETY: there's a current OpenGL context because we are inside of the Expose event
                let data = data.get_or_insert_with(|| unsafe {
                    let mut gl = OpenGlBackend::new(|c| backend.get_proc_address(c) as *const _).unwrap();
                    let shader = gl.open().create_shader(Graph::collect(|| {
                        let color = io::read::<[u8; 4]>();
                        let _extra = io::read::<[u32; 7]>();

                        float4((
                            float1(color[0]) / 255.0,
                            float1(color[1]) / 255.0,
                            float1(color[2]) / 255.0,
                            float1(color[3]) / 255.0,
                        ))
                    }));

                    Data {
                        gl,
                        shader,
                        width: 512,
                        height: 512,
                        avg_time_ms: 0.0,
                    }
                });

                let mut commands = CommandBuffer::default();
                let mut frame = commands.begin_screen([data.width, data.height]);
                frame.clear([0, 0, data.width, data.height]);

                for i in 0..data.width {
                    for j in 0..data.height {
                        frame
                            .begin_quad(data.shader, [i, j, i + 1, j + 1])
                            .write_data(if (i + j) % 2 == 0 {
                                [255, 0, 0, 255u8]
                            } else {
                                [0, 0, 255, 255u8]
                            })
                            .write_data([0u32; 7]);
                    }
                }

                // SAFETY: there's a current OpenGL context because we are inside of the Expose event
                unsafe {
                    let mut gl = data.gl.open();
                    gl.draw(&commands);

                    let stats = gl.stats();
                    let gpu_time_ms = stats.gpu_time.unwrap_or_default().as_secs_f32() * 1000.0;
                    data.avg_time_ms = data.avg_time_ms * 0.99 + gpu_time_ms * 0.01;

                    println!(
                        "avg time: {:.2}ms, time: {:.2}ms, bytes sent: {}, drawcalls: {}",
                        data.avg_time_ms, gpu_time_ms, stats.bytes_sent, stats.draw_calls,
                    );
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
