use picodraw::{
    Context, DrawTarget, ShaderData,
    opengl::{Native, OpenGlBackend, OpenGlShader},
    trace::{TraceGraph, float4, int1},
};
use pugl_rs::{Event, OpenGl, OpenGlVersion, World};
use std::time::Duration;

struct Data {
    gl: OpenGlBackend<Native>,
    shader: OpenGlShader,
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
                    let shader = gl
                        .open()
                        .create_shader(ShaderData::from(&TraceGraph::new(|| {
                            let r = int1::read_u8(0);
                            let g = int1::read_u8(1);
                            let b = int1::read_u8(2);
                            let a = int1::read_u8(3);

                            float4((r, g, b, a)) / 255.0
                        })))
                        .unwrap();

                    Data {
                        gl,
                        shader,
                        width: 512,
                        height: 512,
                        avg_time_ms: 0.0,
                    }
                });

                // SAFETY: there's a current OpenGL context because we are inside of the Expose event
                unsafe {
                    let mut gl = data.gl.open();
                    gl.set_viewport([data.width, data.height]);
                    gl.draw(DrawTarget::Screen, |encoder| {
                        encoder.clear([0, 0, data.width, data.height].into());

                        for i in 0..data.width {
                            for j in 0..data.height {
                                let p = (i + j) % 2 == 0;

                                encoder.add_data(&[if p { 255 } else { 0 }]); // R
                                encoder.add_data(&[if p { 0 } else { 255 }]); // G
                                encoder.add_data(&[if p { 255 } else { 0 }]); // B
                                encoder.add_data(&[255]); // A
                                encoder.add_data(&[0; 12]); // Extra stuff
                                encoder.add_rect([i, j, i + 1, j + 1].into());
                                encoder.object(&data.shader);
                            }
                        }
                    });

                    let stats = gl.stats();
                    let gpu_time_ms = stats.gpu_time.unwrap_or_default().as_secs_f32() * 1000.0;
                    let cpu_time_ms = stats.cpu_time.unwrap_or_default().as_secs_f32() * 1000.0;
                    let total_time_ms = gpu_time_ms + cpu_time_ms;
                    let fill_rate = if total_time_ms > 0.0 {
                        (stats.total_pixels as f32) / (gpu_time_ms * 1000.0)
                    } else {
                        0.0
                    };

                    data.avg_time_ms = data.avg_time_ms * 0.99 + total_time_ms * 0.01;

                    println!(
                        "avg time: {:.2}ms, time: {:.2}ms (gpu {:.2}ms, cpu {:.2}ms), bytes sent: {}, drawcalls: {}, fillrate: {:.2} Mpixels/s",
                        data.avg_time_ms, total_time_ms, gpu_time_ms, cpu_time_ms, stats.bytes_sent, stats.draw_calls, fill_rate
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
