use bmp::{Image, open};
use picodraw::{
    CommandBuffer, Context, Graph, ImageData, ImageFormat, RenderTexture, ShaderData, Texture,
    shader::{boolean, float1, float2, float4, io},
};

/// a basic test that draws a single full screen solid colored quad
/// - tests that dispatching even works
#[test]
fn fill_purple() {
    run("fill-purple", |context| {
        let shader = context.create_shader(Graph::collect(|| {
            io::write_color(float4((1.0, 0.0, 1.0, 1.0)));
        }));

        let mut commands = CommandBuffer::default();
        commands
            .begin_screen([512, 512])
            .begin_quad(shader, [0, 0, 512, 512]);
        context.draw(&commands);
    });
}

/// a basic test that draws a single full screen quad with a checkerboard pattern
/// - tests that dispatching works
/// - non trivial pixel position dependent shaders can run
#[test]
fn fill_checker() {
    run("fill-checker", |context| {
        let shader = context.create_shader(Graph::collect(|| {
            let grid = (io::position() / 16.0).floor();
            let checker = (grid.x() + grid.y()) % 2.0;
            io::write_color(
                float4(checker).lerp(float4((0.0, 0.0, 0.0, 1.0)), float4((1.0, 0.0, 0.0, 1.0))),
            );
        }));

        let mut commands = CommandBuffer::default();
        commands
            .begin_screen([512, 512])
            .begin_quad(shader, [0, 0, 512, 512]);
        context.draw(&commands);
    });
}

/// a test that draws a single quad with a checkerboard and a circular mask and applies post processing (box blur) to it
/// tests:
/// - that dispatching works
/// - dispatching multiple shaders
/// - using a framebuffer to implement post processing
/// - using correct (top left origin) coordinate system
/// - that quads are not rendered beyond their bounds
/// - passing data to the shader using `write_data`
#[test]
fn blurry_semicircle() {
    run("blurry-semicircle", |context| {
        fn sdf_circle(pos: float2, center: float2, radius: float1) -> float1 {
            ((center - pos).len() - radius).smoothstep(0.707, -0.707)
        }

        let shader_circle = context.create_shader(Graph::collect(|| {
            let [x, y] = io::read::<[f32; 2]>();

            let grid = (io::position() / 32.0).floor();
            let checker = (grid.x() + grid.y()) % 2.0;
            let texture =
                float4(checker).lerp(float4((1.0, 1.0, 1.0, 1.0)), float4((1.0, 0.0, 0.0, 1.0)));
            let mask = sdf_circle(io::position(), float2((x, y)), float1(128.0));
            let color = texture * mask;

            io::write_color(float4(mask * color));
        }));

        let shader_boxblur = context.create_shader(Graph::collect(|| {
            let buffer = io::read::<RenderTexture>();

            let mut result = float4(0.0);
            for i in -5..=5 {
                for j in -5..=5 {
                    result = result + buffer.sample_nearest(io::position() + float2((i, j)));
                }
            }

            io::write_color(result / (11 * 11) as f32);
        }));

        let buffer = context.create_texture_render();
        let mut commands = CommandBuffer::default();

        commands
            .begin_buffer(buffer, [512, 512])
            .begin_quad(shader_circle, [300, 0, 512, 512])
            .write_data([300.0, 200.0]);

        commands
            .begin_screen([512, 512])
            .begin_quad(shader_boxblur, [0, 0, 512, 512])
            .write_data(buffer);

        context.draw(&commands);
    });
}

/// a test that draws a letter "A" using the MSDF technique
/// tests:
/// - that dispatching works
/// - drawing multiple quads of the same shader instance
/// - passing data to the shader using `write_data`
/// - using correct (top left origin) coordinate system
/// - loading and sampling a static texture
#[test]
fn msdf_text() {
    let (width, height, data) = {
        let msdf = open("./tests/drawtest/msdf.bmp").unwrap();
        let mut data = vec![0u8; (4 * msdf.get_width() * msdf.get_height()) as usize];
        for i in 0..msdf.get_width() {
            for j in 0..msdf.get_height() {
                let p = msdf.get_pixel(i, j);
                data[((i + j * msdf.get_width()) * 4 + 0) as usize] = p.r;
                data[((i + j * msdf.get_width()) * 4 + 1) as usize] = p.g;
                data[((i + j * msdf.get_width()) * 4 + 2) as usize] = p.b;
                data[((i + j * msdf.get_width()) * 4 + 3) as usize] = 255;
            }
        }

        (msdf.get_width(), msdf.get_height(), data)
    };

    run("msdf-text", move |context| {
        let texture = context.create_texture_static(ImageData {
            width,
            height,
            format: ImageFormat::RGBA8,
            data: &data,
        });

        let shader = context.create_shader(Graph::collect(|| {
            let atlas = io::read::<Texture>();
            let (x, y) = io::read::<(f32, f32)>();
            let scale = io::read::<f32>();

            let pos = (io::position() - float2((x, y))) / scale + float2(12.0);
            let sample = atlas.sample_linear(pos.clamp(0.0, atlas.size()));
            let median = float1::max(
                float1::min(sample.x(), sample.y()),
                float1::min(float1::max(sample.x(), sample.y()), sample.z()),
            );

            let mask = (((2.0 * scale).max(1.0) * (median - 0.5)) + 0.5).smoothstep(0.0, 1.0);
            io::write_color(float4(mask));
        }));

        let mut commands = CommandBuffer::default();
        let mut frame = commands.begin_screen([512, 512]);

        let mut x = 10.0;
        let mut scale = 0.5;
        for _ in 0..=10 {
            frame
                .begin_quad(shader, [0, 0, 512, 512])
                .write_data(texture)
                .write_data((x, 12.0 + scale * 10.0))
                .write_data(scale);

            x += 18.0 * scale;
            scale *= 1.325;
        }

        frame
            .begin_quad(shader, [0, 0, 512, 512])
            .write_data(texture)
            .write_data((256.0, 320.0))
            .write_data(20.0);

        context.draw(&commands);
    });
}

/// tests:
/// - that dispatching works
/// - passing data to the shader using `write_data`
/// - drawing `a lot` of quads works properly
///   (for example `opengl` backend has to dispatch multiple draw calls and sync the buffers correctly)
#[test]
fn stress_test() {
    run("stress-test", move |context| {
        let shader = context.create_shader(Graph::collect(|| {
            let [r, g, b] = io::read::<[f32; 3]>();
            io::read::<[u32; 8]>();
            io::write_color(float4((r, g, b, 1.0)));
        }));

        for _ in 0..2 {
            let mut commands = CommandBuffer::default();
            let mut frame = commands.begin_screen([512, 512]);
            frame.clear([0, 0, 512, 512]);

            for i in 0..512 {
                for j in 0..512 {
                    frame
                        .begin_quad(shader, [i, j, i + 1, j + 1])
                        .write_data(if (i + j) % 2 == 0 {
                            [1.0, 1.0, 0.0]
                        } else {
                            [0.0, 0.0, 1.0]
                        })
                        .write_data([0u32; 8]);
                }
            }

            context.draw(&commands);
        }
    });
}

/// tests:
/// - that dispatching works
/// - passing data to the shader using `write_data`
/// - if `ShaderData` derive proc macro implemented correctly
/// - primitive types like u8, i16 and f32 are serialized correctly
/// - literals like 1 and infinity are serialized correctly
/// - if custom `ShaderData` implementation is honored
#[test]
fn serialize_test() {
    fn sdf_circle(pos: float2, center: float2, radius: float1, invert: boolean) -> float1 {
        let mask = ((center - pos).len() - radius).smoothstep(0.707, -0.707);
        invert.select(1.0 - mask, mask)
    }

    #[cfg(feature = "derive")]
    #[derive(ShaderData)]
    struct Circle {
        x: f32,
        y: f32,
        scale: f32,
        invert: bool,
    }

    #[cfg(not(feature = "derive"))]
    {
        struct CircleShader {
            x: float1,
            y: float1,
            scale: float1,
            invert: bool,
        }

        impl ShaderData for Circle {
            type Data = CircleShader;
            fn read() -> Self::Data {
                CircleShader {
                    x: f32::read(),
                    y: f32::read(),
                    scale: f32::read(),
                    invert: bool::read(),
                }
            }
            fn write(&self, writer: &mut dyn io::ShaderDataWriter) {
                f32::write(&self.x, writer);
                f32::write(&self.y, writer);
                f32::write(&self.scale, writer);
                bool::write(&self.invert, writer);
            }
        }
    }

    struct Box {
        bounds: [u8; 4],
        color: i16,
    }

    struct BoxShader {
        bounds: float4,
        color: float1,
    }

    impl ShaderData for Box {
        type Data = BoxShader;
        fn read() -> Self::Data {
            BoxShader {
                bounds: float4((u8::read(), u8::read(), u8::read(), u8::read())),
                color: float1(i16::read()) / (i16::MAX as f32),
            }
        }
        fn write(&self, writer: &mut dyn io::ShaderDataWriter) {
            writer.write_i32(self.bounds[0] as i32);
            writer.write_i32(self.bounds[1] as i32);
            writer.write_i32(self.bounds[2] as i32);
            writer.write_i32(self.bounds[3] as i32);
            writer.write_i32(self.color as i32);
        }
    }

    run("serialize-test", move |context| {
        let shader = context.create_shader(Graph::collect(|| {
            let zero = 1.0 / float1(f32::INFINITY);

            let data_circle = io::read::<Circle>();
            let data_box = io::read::<Box>();
            let position = io::position();

            let mask_box = data_box.color
                * position.x().step(data_box.bounds.x())
                * position.y().step(data_box.bounds.y())
                * (1.0 - position.x().step(data_box.bounds.z()))
                * (1.0 - position.y().step(data_box.bounds.w()));
            let mask_circle = 0.5
                * sdf_circle(
                    position,
                    float2((data_circle.x, data_circle.y)),
                    data_circle.scale,
                    data_circle.invert,
                );

            io::write_color(float4(mask_box + mask_circle + zero));
        }));

        let mut commands = CommandBuffer::default();
        commands
            .begin_screen([512, 512])
            .begin_quad(shader, [0, 0, 512, 512])
            .write_data(Circle {
                x: 256.0,
                y: 256.0,
                scale: 150.0,
                invert: false,
            })
            .write_data(Box {
                bounds: [100, 100, 250, 250],
                color: i16::MAX / 2,
            });

        context.draw(&commands);
    });
}

/// a test that does a bunch of simple resource operations across multiple frames
/// tests:
/// - that dispatching works
/// - loading and sampling a static texture with different texture formats
/// - creating and deleting shader/texture/framebuffer objects across frames
/// - that buffer data is preserved across frames (unless `clear` is called for a select region)
/// - that screen frame data is preserved across frames
#[test]
fn resource_mgmt() {
    run("resource-mgmt", |context| {
        // frame 0 prepare
        let texture_0 = context.create_texture_static(ImageData {
            width: 1,
            height: 1,
            format: ImageFormat::R8,
            data: &[127],
        });

        let texture_1 = context.create_texture_static(ImageData {
            width: 2,
            height: 1,
            format: ImageFormat::RGB8,
            data: &[127, 127, 127, 255, 255, 255],
        });

        let shader_0 = context.create_shader(Graph::collect(|| {
            io::write_color(float4((0.0, 0.0, 0.0, 0.25)));
        }));

        let shader_1 = context.create_shader(Graph::collect(|| {
            io::write_color(io::read::<Texture>().sample_linear(io::position()))
        }));

        let buffer_0 = context.create_texture_render();

        // frame 0 draw
        let mut commands = CommandBuffer::default();
        commands
            .begin_buffer(buffer_0, [512, 512])
            .begin_quad(shader_1, [0, 0, 10, 10])
            .write_data(texture_0);
        context.draw(&commands);

        // frame 1 prepare
        let texture_2 = context.create_texture_static(ImageData {
            width: 2,
            height: 2,
            format: ImageFormat::RGBA8,
            data: &[
                0, 0, 0, 255, 255, 0, 0, 255, 0, 255, 0, 255, 255, 255, 0, 255,
            ],
        });

        let shader_2 = context.create_shader(Graph::collect(|| {
            io::write_color(io::read::<Texture>().sample_linear(io::position()) * 2.0)
        }));

        assert!(context.delete_shader(shader_1));
        assert!(!context.delete_shader(shader_1));
        assert!(context.delete_texture_static(texture_0));
        assert!(!context.delete_texture_static(texture_0));

        // frame 1 draw
        let mut commands = CommandBuffer::default();
        let mut frame = commands.begin_buffer(buffer_0, [512, 512]);
        frame.clear([5, 5, 10, 10]);
        frame
            .begin_quad(shader_2, [10, 10, 20, 20])
            .write_data(texture_1);
        frame
            .begin_quad(shader_2, [20, 20, 30, 30])
            .write_data(texture_2);
        frame.begin_quad(shader_0, [0, 0, 20, 20]);
        context.draw(&commands);

        // frame 2 prepare
        assert!(context.delete_texture_static(texture_1));
        assert!(!context.delete_texture_static(texture_1));
        assert!(context.delete_texture_static(texture_2));
        assert!(!context.delete_texture_static(texture_2));

        let shader_3 = context.create_shader(Graph::collect(|| {
            io::write_color(io::read::<RenderTexture>().sample_linear(io::position()))
        }));

        let shader_4 = context.create_shader(Graph::collect(|| {
            io::write_color(float4((1.0, 1.0, 1.0, 1.0)))
        }));

        // frame 2 drawpath
        let mut commands = CommandBuffer::default();
        let mut frame = commands.begin_screen([512, 512]);
        frame.begin_quad(shader_4, [0, 0, 512, 512]);
        frame
            .begin_quad(shader_3, [0, 0, 512, 512])
            .write_data(buffer_0);
        frame.clear([10, 10, 15, 15]);
        context.draw(&commands);

        // cleanup
        assert!(context.delete_texture_render(buffer_0));
        assert!(!context.delete_texture_render(buffer_0));
        assert!(context.delete_shader(shader_3));
        assert!(!context.delete_shader(shader_3));
        assert!(context.delete_shader(shader_4));
        assert!(!context.delete_shader(shader_4));
    });
}

#[allow(unused_variables, dead_code)]
fn run(id: &str, render: impl Fn(&mut dyn Context) + Send + 'static) {
    fn difference(a: &Image, b: &Image) -> f64 {
        let mut sum = 0;
        for i in 0..a.get_width() {
            for j in 0..b.get_height() {
                let a = a.get_pixel(i, j);
                let b = b.get_pixel(i, j);
                sum += a.r.abs_diff(b.r) as u64;
                sum += a.g.abs_diff(b.g) as u64;
                sum += a.b.abs_diff(b.b) as u64;
            }
        }
        sum as f64 / (a.get_height() * a.get_width()) as f64
    }

    if cfg!(miri) {
        return;
    }

    std::fs::create_dir_all("./tests/drawtest/failures").ok();

    let expected =
        open(format!("./tests/drawtest/expected/{}.bmp", id)).expect("no 'expected' image found");

    #[cfg(feature = "opengl")]
    {
        let result = opengl::render(expected.get_width(), expected.get_height(), render);
        let diff = difference(&expected, &result);
        if diff > 3.0 {
            result
                .save(format!("./tests/drawtest/failures/opengl-{}.bmp", id))
                .unwrap();
            panic!("opengl backend: {:.2} difference", diff);
        } else {
            std::fs::remove_file(format!("./tests/drawtest/failures/opengl-{}.bmp", id)).ok();
        }
    }
}

#[cfg(feature = "opengl")]
mod opengl {
    use bmp::{Image, Pixel};
    use picodraw::{Context, opengl::OpenGlBackend};
    use pugl_rs::{Event, OpenGl, OpenGlVersion, World};
    use std::{
        sync::mpsc::{TryRecvError, sync_channel},
        time::Duration,
    };

    pub fn render(
        width: u32,
        height: u32,
        render: impl Fn(&mut dyn Context) + Send + 'static,
    ) -> Image {
        let (sender, receiver) = sync_channel::<Image>(1);
        let mut gl_backend = None;

        let mut world = World::new_program().unwrap();
        let view = world
            .new_view(OpenGl {
                bits_alpha: 0,
                bits_depth: 0,
                bits_stencil: 0,
                version: OpenGlVersion::Core(3, 3),
                ..Default::default()
            })
            .with_size(512, 512)
            .with_event_handler(move |view, event| match event {
                Event::Expose { backend, .. } => {
                    let mut gl_backend = unsafe {
                        gl_backend
                            .get_or_insert_with(|| {
                                OpenGlBackend::new(&|c| backend.get_proc_address(c)).unwrap()
                            })
                            .open()
                    };

                    render(&mut gl_backend);

                    {
                        let screenshot: Vec<u32> = gl_backend.screenshot([0, 0, width, height]);
                        let mut image = Image::new(width, height);
                        for i in 0..width {
                            for j in 0..height {
                                let data = screenshot[(i + j * width) as usize];
                                image.set_pixel(i, height - 1 - j, Pixel {
                                    r: ((data >> 0) & 0xFF) as u8,
                                    g: ((data >> 8) & 0xFF) as u8,
                                    b: ((data >> 16) & 0xFF) as u8,
                                });
                            }
                        }

                        let _ = sender.send(image);
                    }
                }
                Event::Update => {
                    view.obscure_view();
                }
                _ => {}
            })
            .realize()
            .unwrap();

        view.show_passive();

        loop {
            world.update(Some(Duration::ZERO)).unwrap();
            match receiver.try_recv() {
                Ok(image) => return image,
                Err(TryRecvError::Empty) => continue,
                Err(TryRecvError::Disconnected) => panic!("receiver disconnected"),
            }
        }
    }
}
