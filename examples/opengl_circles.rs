use picodraw::{
    Color, Context, DrawTarget, ShaderData, opengl,
    trace::{float1, float2, float4},
};
use picoview::{Event, GlConfig, GlVersion, WindowBuilder};
use std::time::Instant;

struct Data {
    gl: opengl::Backend<opengl::Native>,
    shader: opengl::Shader,
    width: u32,
    height: u32,
    scroll: f32,
    avg_time_ms: f32,

    total_frames: i32,
    total_time: Instant,
}

fn shader_circle() -> float4 {
    fn sdf_circle(pos: float2, center: float2, radius: float1) -> float1 {
        (radius - (center - pos).len() + 0.5).clamp(0.0, 1.0)
    }

    let x = float1::read_f32(0);
    let y = float1::read_f32(4);
    let radius = float1::read_f32(8);
    let alpha = float1::read_f32(12);

    let mask = sdf_circle(float2::position(), float2((x, y)), radius);

    float4((1.0, 0.5, 1.0, mask * alpha))
}

fn main() {
    WindowBuilder::new(|window| {
        let mut data: Option<Data> = None;
        Box::new(move |event| match event {
            Event::WindowFrame { gl: Some(gl) } => {
                if !gl.make_current(true) {
                    return;
                }

                // SAFETY: there's a current OpenGL context because we called `make_current` above
                unsafe {
                    let data = data.get_or_insert_with(|| {
                        let mut gl = opengl::Backend::new(opengl::Config::default(),|c| gl.get_proc_address(c) as *const _).unwrap();
                        let shader = gl
                            .open()
                            .create_shader(&ShaderData::trace(shader_circle))
                            .unwrap();

                        Data {
                            gl,
                            shader,
                            width: 512,
                            height: 512,
                            scroll: 0.0,
                            avg_time_ms: 0.0,
                            total_frames: 0,
                            total_time: Instant::now(),
                        }
                    });

                    let mut gl = data.gl.open();
                    gl.set_viewport([data.width, data.height]);
                    gl.draw(DrawTarget::Screen, |encoder| {
                        encoder.clear([0, 0, data.width, data.height].into(), Color {
                            r: 0,
                            g: 20,
                            b: 0,
                            a: 0,
                        });

                        let n = (data.scroll * 0.2).sin() * 14.0 + 20.0;
                        let alpha = 1.0 / n;

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
                            encoder.draw(&data.shader);
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

                    println!(
                        "#{} ({}ms): avg time: {:.2}ms, time: {:.2}ms (gpu {:.2}ms, cpu {:.2}ms), bytes sent: {}, drawcalls: {}, fillrate: {:.2} Mpixels/s",
                        data.total_frames, data.total_time.elapsed().as_millis(), data.avg_time_ms, total_time_ms, gpu_time_ms, cpu_time_ms, stats.bytes_sent, stats.draw_calls, fill_rate
                    );


                    data.avg_time_ms = data.avg_time_ms * 0.99 + total_time_ms * 0.01;
                    data.total_frames += 1;
                    data.scroll += 1.0 / 60.0;
                }

                gl.swap_buffers();
                gl.make_current(false);
            }

            Event::WindowResize { size, .. } => {
                if let Some(data) = data.as_mut() {
                    data.width = size.width;
                    data.height = size.height;
                }
            }

            Event::WindowClose => {
                window.close();
            }

            Event::MouseScroll { y, .. } => {
                if let Some(data) = data.as_mut() {
                    data.scroll -= y * 0.1;
                }
            }

            _ => {}
        })
    })
    .with_title("picodraw opengl example")
    .with_size((512, 512))
    .with_resizable((0, 0), (u32::MAX, u32::MAX))
    .with_opengl(GlConfig {
        version: GlVersion::Core(4, 6),
        ..Default::default()
    })
    .open_blocking()
    .unwrap();
}
