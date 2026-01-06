use picodraw::{
    Color, Context, DrawTarget, ShaderData, opengl,
    trace::{float4, int1},
};
use picoview::{Event, GlConfig, GlVersion, WindowBuilder};

struct Data {
    gl: opengl::Backend<opengl::Native>,
    shader: opengl::Shader,
    width: u32,
    height: u32,
    avg_time_ms: f32,
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
                            .create_shader(&ShaderData::trace(|| {
                                let r = int1::read_u8(0);
                                let g = int1::read_u8(1);
                                let b = int1::read_u8(2);
                                let a = int1::read_u8(3);

                                float4((r, g, b, a)) / 255.0
                            }))
                            .unwrap();

                        Data {
                            gl,
                            shader,
                            width: 512,
                            height: 512,
                            avg_time_ms: 0.0,
                        }
                    });

                    let mut gl = data.gl.open();
                    gl.set_viewport([data.width, data.height]);
                    gl.draw(DrawTarget::Screen, |encoder| {
                        encoder.clear([0, 0, data.width, data.height].into(), Color::default());

                        for i in 0..data.width {
                            for j in 0..data.height {
                                let p = (i + j) % 2 == 0;

                                encoder.add_data(&[if p { 255 } else { 0 }]); // R
                                encoder.add_data(&[if p { 0 } else { 255 }]); // G
                                encoder.add_data(&[if p { 255 } else { 0 }]); // B
                                encoder.add_data(&[255]); // A
                                encoder.add_data(&[0; 12]); // Extra stuff
                                encoder.add_rect([i, j, i + 1, j + 1].into());
                                encoder.draw(&data.shader);
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
