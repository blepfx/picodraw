use picodraw::{
    Context, DrawTarget, ShaderData,
    opengl::{Native, OpenGlBackend, OpenGlShader},
    trace::{TraceGraph, float1, float2, float4},
};
use pugl_rs::{Event, OpenGl, OpenGlVersion, World};
use std::time::Duration;

struct Data {
    gl: OpenGlBackend<Native>,
    shader: OpenGlShader,
    width: u32,
    height: u32,
    scroll: f32,
    avg_time_ms: f32,
}

fn shader_circle() -> float4 {
    fn sdf_circle(pos: float2, center: float2, radius: float1) -> float1 {
        (radius - (center - pos).len() + 0.5).clamp(0.0, 1.0)
    }

    let x = float1::read(0);
    let y = float1::read(4);
    let radius = float1::read(8);
    let alpha = float1::read(12);

    let mask = sdf_circle(float2::position(), float2((x, y)), radius);

    float4((1.0, 0.5, 1.0, mask * alpha))
}

fn main() {
    let mut data: Option<Data> = None;
    let mut world = World::new_program().unwrap();
    let view = world
        .new_view(OpenGl {
            version: OpenGlVersion::Core(3, 3),
            debug: true,
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
                        .create_shader(ShaderData::from(&TraceGraph::new(shader_circle)))
                        .unwrap();

                    Data {
                        gl,
                        shader,
                        width: 512,
                        height: 512,
                        scroll: 0.0,
                        avg_time_ms: 0.0,
                    }
                });

                // SAFETY: there's a current OpenGL context because we are inside of the Expose event
                unsafe {
                    let mut gl = data.gl.open();
                    gl.set_viewport([data.width, data.height]);
                    gl.draw(DrawTarget::Screen, |encoder| {
                        encoder.clear([0, 0, data.width, data.height].into());

                        let n = (data.scroll * 0.2).sin() * 14.0 + 20.0;
                        let alpha = 1.0 / n as f32;

                        for i in 0..n as i32 {
                            let angle = (i as f32 / (n - 1.0) + data.scroll * 0.05) * std::f32::consts::PI * 2.0;
                            let x = data.width as f32 * 0.5 + angle.cos() * 200.0;
                            let y = data.height as f32 * 0.5 + angle.sin() * 200.0;
                            let alpha = if i + 1 == (n as i32) { alpha * n.fract() } else { alpha };

                            encoder.add_data(&f32::to_ne_bytes(x));
                            encoder.add_data(&f32::to_ne_bytes(y));
                            encoder.add_data(&f32::to_ne_bytes(200.0));
                            encoder.add_data(&f32::to_ne_bytes(alpha));
                            encoder.add_rect([0, 0, data.width, data.height].into());
                            encoder.object(&data.shader);
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
