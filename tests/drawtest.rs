use std::sync::Arc;

use image::{DynamicImage, GenericImageView, Rgba, open};
use picodraw::{
    CommandBuffer, Context, Graph, ImageData, ImageFormat, RenderTexture, ShaderData, ShaderDataWriter, Texture,
    shader::*,
};

const CANVAS_SIZE: u32 = 512;

/// a basic test that draws a single full screen solid colored quad
/// - tests that dispatching even works
#[test]
fn fill_purple() {
    run("fill-purple", |context| {
        let shader = context.create_shader(Graph::collect(|| float4((1.0, 0.0, 1.0, 1.0))));

        let mut commands = CommandBuffer::new();
        commands
            .begin_screen([CANVAS_SIZE, CANVAS_SIZE])
            .begin_quad(shader, [0, 0, CANVAS_SIZE, CANVAS_SIZE]);
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

            float4(checker).lerp(float4((0.0, 0.0, 0.0, 1.0)), float4((1.0, 0.0, 0.0, 1.0)))
        }));

        let mut commands = CommandBuffer::new();
        commands
            .begin_screen([CANVAS_SIZE, CANVAS_SIZE])
            .begin_quad(shader, [0, 0, CANVAS_SIZE, CANVAS_SIZE]);
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
            let texture = float4(checker).lerp(float4((1.0, 1.0, 1.0, 1.0)), float4((1.0, 0.0, 0.0, 1.0)));
            let mask = sdf_circle(io::position(), float2((x, y)), float1(128.0));
            let color = texture * mask;

            float4(mask * color)
        }));

        let shader_boxblur = context.create_shader(Graph::collect(|| {
            let buffer = io::read::<RenderTexture>();

            let mut result = float4(0.0);
            for i in -5..=5 {
                for j in -5..=5 {
                    result = result + buffer.sample_nearest(io::position() + float2((i, j)));
                }
            }

            result / (11 * 11) as f32
        }));

        let buffer = context.create_texture_render();
        let mut commands = CommandBuffer::new();

        commands
            .begin_buffer(buffer, [CANVAS_SIZE, CANVAS_SIZE])
            .begin_quad(shader_circle, [300, 0, CANVAS_SIZE, CANVAS_SIZE])
            .write_data([300.0, 200.0]);

        commands
            .begin_screen([CANVAS_SIZE, CANVAS_SIZE])
            .begin_quad(shader_boxblur, [0, 0, CANVAS_SIZE, CANVAS_SIZE])
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
        let msdf = open("./tests/drawtest/msdf.webp").unwrap();
        let mut data = vec![0u8; (4 * msdf.width() * msdf.height()) as usize];
        for i in 0..msdf.width() {
            for j in 0..msdf.height() {
                let Rgba([r, g, b, _]) = msdf.get_pixel(i, j);
                data[((i + j * msdf.width()) * 4 + 0) as usize] = r;
                data[((i + j * msdf.width()) * 4 + 1) as usize] = g;
                data[((i + j * msdf.width()) * 4 + 2) as usize] = b;
                data[((i + j * msdf.width()) * 4 + 3) as usize] = 255;
            }
        }

        (msdf.width(), msdf.height(), data)
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
            float4(mask)
        }));

        let mut commands = CommandBuffer::new();
        let mut frame = commands.begin_screen([CANVAS_SIZE, CANVAS_SIZE]);

        let mut x = 10.0;
        let mut scale = 0.5;
        for _ in 0..=10 {
            frame
                .begin_quad(shader, [0, 0, CANVAS_SIZE, CANVAS_SIZE])
                .write_data(texture)
                .write_data((x, 12.0 + scale * 10.0))
                .write_data(scale);

            x += 18.0 * scale;
            scale *= 1.325;
        }

        frame
            .begin_quad(shader, [0, 0, CANVAS_SIZE, CANVAS_SIZE])
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
            float4((r, g, b, 1.0))
        }));

        for _ in 0..2 {
            let mut commands = CommandBuffer::new();
            let mut frame = commands.begin_screen([CANVAS_SIZE, CANVAS_SIZE]);
            frame.clear([0, 0, CANVAS_SIZE, CANVAS_SIZE]);

            for i in 0..CANVAS_SIZE {
                for j in 0..CANVAS_SIZE {
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
/// - if custom `ShaderData` implementation is honored, including implementations that use resolution/bounds data at serialize time
/// - if `#[shader(ignore)]` and `#[shader(_)]` attributes work correctly
/// - if `ShaderData` implementation can be derived for tuple structs and unit structs
/// - if `ShaderData` proc macro supports custom encoders
///
#[test]
#[cfg(feature = "derive")]
fn serialize_test() {
    fn sdf_circle(pos: float2, center: float2, radius: float1, invert: boolean) -> float1 {
        let mask = ((center - pos).len() - radius).smoothstep(-0.707, 0.707);
        invert.select(mask, 1.0 - mask)
    }

    #[derive(ShaderData)]
    struct PassthroughEncoder<T>(T);
    struct FracResolutionEncoder(f32, f32);
    struct FracQuadBoundsEncoder(f32, f32);

    impl ShaderData for FracResolutionEncoder {
        type Data = float2;
        fn read() -> Self::Data {
            let resolution = io::resolution();
            float2((io::read::<f32>() * resolution.x(), io::read::<f32>() * resolution.y()))
        }
        fn write(&self, writer: &mut dyn ShaderDataWriter) {
            writer.write_f32(self.0 / writer.resolution().width as f32);
            writer.write_f32(self.1 / writer.resolution().height as f32);
        }
    }

    impl ShaderData for FracQuadBoundsEncoder {
        type Data = float2;
        fn read() -> Self::Data {
            let (start, end) = io::bounds();
            float2((io::read::<f32>(), io::read::<f32>())).lerp(start, end)
        }
        fn write(&self, writer: &mut dyn ShaderDataWriter) {
            let bounds = writer.quad_bounds();
            let x0 = (self.0 - bounds.left as f32) / bounds.width() as f32;
            let y0 = (self.1 - bounds.top as f32) / bounds.height() as f32;
            x0.write(writer);
            y0.write(writer);
        }
    }

    impl From<f32> for PassthroughEncoder<f32> {
        fn from(value: f32) -> Self {
            PassthroughEncoder(value)
        }
    }

    impl From<(f32, f32)> for FracResolutionEncoder {
        fn from(value: (f32, f32)) -> Self {
            FracResolutionEncoder(value.0, value.1)
        }
    }

    impl From<(f32, f32)> for FracQuadBoundsEncoder {
        fn from(value: (f32, f32)) -> Self {
            FracQuadBoundsEncoder(value.0, value.1)
        }
    }

    #[derive(ShaderData)]
    struct Circle {
        #[shader(FracResolutionEncoder)]
        point: (f32, f32),

        #[shader(PassthroughEncoder::<f32>)]
        scale: f32,

        invert: bool,

        #[shader(ignore)]
        _ignored: f32,
    }

    #[derive(ShaderData)]
    struct Point(#[shader(FracQuadBoundsEncoder)] (f32, f32));

    #[derive(ShaderData)]
    struct Nothing;

    #[derive(ShaderData)]
    struct Primitives {
        i8: i8,
        i16: i16,
        i32: i32,
        u8: u8,
        u16: u16,
        u32: u32,
        f32: f32,
        bool: bool,
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
        fn write(&self, writer: &mut dyn ShaderDataWriter) {
            writer.write_i32(self.bounds[0] as i32);
            writer.write_i32(self.bounds[1] as i32);
            writer.write_i32(self.bounds[2] as i32);
            writer.write_i32(self.bounds[3] as i32);
            writer.write_i32(self.color as i32);
        }
    }

    run("serialize-test", move |context| {
        let shader = context.create_shader(Graph::collect(|| {
            let data_circle = io::read::<Circle>();
            let data_box = io::read::<Box>();
            let data_point = io::read::<Point>();
            let data_primitives = io::read::<Primitives>();
            let data_extra = io::read::<&f64>();
            let _ = io::read::<Nothing>();
            let _ = io::read::<()>(); //wow

            let position = io::position();
            let mask_box = data_box.color
                * position.x().step(data_box.bounds.x())
                * position.y().step(data_box.bounds.y())
                * (1.0 - position.x().step(data_box.bounds.z()))
                * (1.0 - position.y().step(data_box.bounds.w()));

            let mask_circle = 0.5 * sdf_circle(position, data_circle.point, data_circle.scale.0, data_circle.invert);

            let mask_point = sdf_circle(position, data_point.0, float1(5.0), boolean(false));

            let background = {
                let nan_soup = float1(f32::INFINITY) + float1(f32::NEG_INFINITY) + float1(f32::NAN);
                let literal_soup = float1(1.0)
                    + float1(0.0)
                    + float1(-1.0)
                    + float1(0.5)
                    + float1(int1(1) + int1(0) + int1(-1) + int1(2) + int1(-2) + int1(3) + int1(-3))
                    + boolean(true).select(float1(0.25), 0.0)
                    + boolean(false).select(float1(0.0), 0.5);

                let pos = io::position() * 0.001;
                let a = pos.x() * data_primitives.f32
                    + pos.y() * float1(data_primitives.i8)
                    + float1(data_primitives.i16)
                    + float1(data_primitives.i32)
                    + float1(data_primitives.u8)
                    + float1(data_primitives.u16)
                    + float1(data_primitives.u32)
                    + data_primitives.bool.select(float1(1.0), 2.0)
                    + nan_soup.eq(nan_soup).select(nan_soup, 0.0)
                    + literal_soup % 0.1
                    + data_extra;

                float4((1.0, 1.0, 1.0, a.sin().abs() * 0.2))
            };

            let bounding_box = {
                let (start, end) = io::bounds();
                let start = start + 2.0;
                let end = end - 2.0;

                let mask = position.x().step(start.x())
                    * position.y().step(start.y())
                    * (1.0 - position.x().step(end.x()))
                    * (1.0 - position.y().step(end.y()));

                1.0 - mask
            };

            float4(bounding_box + background + mask_box + mask_circle + mask_point)
        }));

        let mut commands = CommandBuffer::new();

        // should not be dispatched! we reset the command buffer before the dispatch
        // dispatching this will lead to `malformed write stream` panic
        commands.begin_screen([1, 1]).begin_quad(shader, [0, 0, 1, 1]);
        commands.reset_commands();

        commands
            .begin_screen([CANVAS_SIZE, CANVAS_SIZE])
            .begin_quad(shader, [20, 20, CANVAS_SIZE - 20, CANVAS_SIZE - 20])
            .write_data(Circle {
                point: (256.0, 256.0),
                scale: 150.0,
                invert: false,
                _ignored: 1.0,
            })
            .write_data(Box {
                bounds: [100, 100, 250, 250],
                color: i16::MAX / 2,
            })
            .write_data(Point((350.0, 350.0)))
            .write_data(Primitives {
                i8: 12,
                i16: -327,
                i32: 2147,
                u8: 255,
                u16: 655,
                u32: 4294,
                f32: 3.14159,
                bool: true,
            })
            .write_data(&0.1f64)
            .write_data(Nothing)
            .write_data(());

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
            format: ImageFormat::Gray8,
            data: &[127],
        });

        let texture_1 = context.create_texture_static(ImageData {
            width: 2,
            height: 1,
            format: ImageFormat::RGB8,
            data: &[127, 127, 127, 255, 255, 255],
        });

        let shader_0 = context.create_shader(Graph::collect(|| float4((0.0, 0.0, 0.0, 0.25))));
        let shader_1 = context.create_shader(Graph::collect(|| io::read::<Texture>().sample_linear(io::position())));

        let buffer_0 = context.create_texture_render();

        // frame 0 draw
        let mut commands = CommandBuffer::new();
        commands
            .begin_buffer(buffer_0, [CANVAS_SIZE, CANVAS_SIZE])
            .begin_quad(shader_1, [0, 0, 10, 10])
            .write_data(texture_0);
        context.draw(&commands);

        // frame 1 prepare
        let texture_2 = context.create_texture_static(ImageData {
            width: 2,
            height: 2,
            format: ImageFormat::RGBA8,
            data: &[0, 0, 0, 255, 255, 0, 0, 255, 0, 255, 0, 255, 255, 255, 0, 255],
        });

        let shader_2 = context.create_shader(Graph::collect(|| {
            io::read::<Texture>().sample_linear(io::position()) * 2.0
        }));

        assert!(context.delete_shader(shader_1));
        assert!(!context.delete_shader(shader_1));
        assert!(context.delete_texture_static(texture_0));
        assert!(!context.delete_texture_static(texture_0));

        // frame 1 draw
        let mut commands = CommandBuffer::new();
        let mut frame = commands.begin_buffer(buffer_0, [CANVAS_SIZE, CANVAS_SIZE]);
        frame.clear([5, 5, 10, 10]);
        frame.begin_quad(shader_2, [10, 10, 20, 20]).write_data(texture_1);
        frame.begin_quad(shader_2, [20, 20, 30, 30]).write_data(texture_2);
        frame.begin_quad(shader_0, [0, 0, 20, 20]);
        context.draw(&commands);

        // frame 2 prepare
        assert!(context.delete_texture_static(texture_1));
        assert!(!context.delete_texture_static(texture_1));
        assert!(context.delete_texture_static(texture_2));
        assert!(!context.delete_texture_static(texture_2));

        let shader_4 = context.create_shader(Graph::collect(|| float4((1.0, 1.0, 1.0, 1.0))));
        let shader_3 = context.create_shader(Graph::collect(|| {
            io::read::<RenderTexture>().sample_linear(io::position())
        }));

        // frame 2 drawpath
        let mut commands = CommandBuffer::new();
        let mut frame = commands.begin_screen([CANVAS_SIZE, CANVAS_SIZE]);
        frame.begin_quad(shader_4, [0, 0, CANVAS_SIZE, CANVAS_SIZE]);
        frame
            .begin_quad(shader_3, [0, 0, CANVAS_SIZE, CANVAS_SIZE])
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

#[test]
fn shader_ops() {
    run("shader-ops", |context| {
        let shader = context.create_shader(Graph::collect(|| {
            let funcs: [fn(float1, float1) -> float3; 11] = [
                |x, _| {
                    // trig functions
                    let x = x * 10.0 - 5.0;
                    float3((x.sin(), x.cos(), x.tan()))
                },
                |x, _| {
                    // trig functions
                    let x = x * 10.0 - 5.0;
                    float3((x.asin(), x.acos(), x.atan()))
                },
                |x, _| {
                    // exponentiation and powers
                    let x = x * 10.0 - 5.0;
                    float3((x.exp(), x.sqrt(), x.ln()))
                },
                |x, _| {
                    // exponentiation and powers
                    let x = x * 10.0 - 5.0;
                    float3((x.pow(20.1), x.pow(2.0), x.pow(-0.333)))
                },
                |x, _| {
                    // boolean masks and scalar comparisons
                    let x = x * 10.0 - 5.0;
                    let y = int1(x);

                    let a = x.lt(1.0).select(float1(0.9), 0.1);
                    let b = x.gt(2.0).select(float1(0.8), 0.2);
                    let c = x.ge(3.0).select(float1(0.7), 0.3);
                    let d = x.le(4.0).select(float1(0.6), 0.4);

                    let i = y.lt(-1).select(float1(0.9), 0.1);
                    let j = y.gt(-2).select(float1(0.8), 0.2);
                    let k = y.ge(-3).select(float1(0.7), 0.3);
                    let l = y.le(-4).select(float1(0.6), 0.4);
                    let m = y.eq(-5).select(float1(0.5), 0.5);
                    let n = y.ne(-6).select(float1(0.4), 0.6);

                    float3((a + b + c + d, i + j + k, l + m + n)) * 0.25
                },
                |x, _| {
                    // floor/abs/sign
                    let x = x * 10.0 - 5.0;
                    float3((
                        x.floor(),
                        x.sign() + x.norm(), /* same as sign */
                        x.abs() + x.len(),   /* same as abs */
                    )) * 0.25
                },
                |x, _| {
                    // min/max/clamp/lerp
                    let x = x * 10.0 - 5.0;
                    float3((x.min(1.0) + x.max(2.0), x.clamp(-1.0, 1.0), x.lerp(-0.5, 1.0))) * 0.25 + 0.25
                },
                |j, i| {
                    // integer bitwise operations
                    let x = int1(j * 128.0);
                    let y = int1(i * 64.0);
                    let z = int1((i + j) * 32.0);

                    float3((
                        float1((x | y & z) % 16) / 16.0,
                        float1((x & y ^ z) % 16) / 16.0,
                        float1((x ^ y | !z) % 16) / 16.0,
                    ))
                },
                |x, y| {
                    // screen space derivatives
                    let r = io::resolution();
                    let x = x * 10.0 - 5.0;
                    let a = x.sin();
                    let b = x.cos();
                    let c = (y * 5.0).tan();

                    float3((a.dx() * r.x() / 10.0, b, a.fwidth() * r.x() / 10.0))
                        + float3((c.dx() * r.x() / 10.0, c, c.dy() * r.y() / 10.0))
                },
                |x, y| {
                    // 2d vector operations
                    let x = x * 10.0 - 5.0;
                    let y = y * 10.0;

                    let p = y.atan2(x);
                    let n = float2((x, y)).norm();
                    let l = float2((x, y)).len();

                    float3((p, n.y(), l)) * 0.5 + 0.5
                },
                |x, y| {
                    // 3d vector operations
                    let x = x * 10.0 - 5.0;
                    let y = y * 10.0;

                    let p = float3((x, y, 0.0)).cross(float3((1.0, 1.0, 1.0)));
                    let n = float3((x, y, 0.0)).dot(float3((1.0, 1.0, 1.0)));
                    let l = float3((x, y, 0.0)).len();

                    float3((p.x(), n, l)) * 0.5 + 0.5
                },
            ];

            let pos = io::position() / io::resolution();
            let idx = int1(pos.y() * funcs.len() as f32);

            let mut color = float3(0.0);
            for i in 0..funcs.len() {
                color = color
                    + idx
                        .eq(i as i32)
                        .select(funcs[i](pos.x(), pos.y() % (1.0 / funcs.len() as f32)), 0.0);
            }

            float4((color.x(), color.y(), color.z(), 1.0))
        }));

        let mut commands = CommandBuffer::new();
        commands
            .begin_screen([CANVAS_SIZE, CANVAS_SIZE])
            .begin_quad(shader, [0, 0, CANVAS_SIZE, CANVAS_SIZE]);
        context.draw(&commands);
    });
}

#[allow(unused_variables, dead_code)]
fn run(id: &str, render: impl Fn(&mut dyn Context) + Sync + Send + 'static) {
    fn difference(a: &DynamicImage, b: &DynamicImage) -> f64 {
        let mut sum = 0;
        for i in 0..a.width() {
            for j in 0..a.height() {
                let Rgba([r0, g0, b0, a0]) = a.get_pixel(i, j);
                let Rgba([r1, g1, b1, a1]) = b.get_pixel(i, j);
                sum += r0.abs_diff(r1) as u64;
                sum += g0.abs_diff(g1) as u64;
                sum += b0.abs_diff(b1) as u64;
                sum += a0.abs_diff(a1) as u64;
            }
        }
        sum as f64 / (a.height() * a.width()) as f64
    }

    if cfg!(miri) {
        return;
    }

    std::fs::create_dir_all("./tests/drawtest/failures").ok();
    std::fs::create_dir_all("./tests/drawtest/successes").ok();

    let renderer = Arc::new(render);

    let expected = open(format!("./tests/drawtest/expected/{}.webp", id)).expect("no 'expected' image found");
    let results: Vec<(&'static str, DynamicImage)> = vec![
        #[cfg(feature = "software")]
        ("software", software::render(CANVAS_SIZE, CANVAS_SIZE, renderer.clone())),
        #[cfg(feature = "opengl")]
        ("opengl", opengl::render(CANVAS_SIZE, CANVAS_SIZE, renderer.clone())),
    ];

    for (backend, result) in results {
        let diff = difference(&result, &expected);
        if diff > 3.0 {
            result
                .save(format!("./tests/drawtest/failures/{}-{}.webp", backend, id))
                .unwrap();
            std::fs::remove_file(format!("./tests/drawtest/successes/{}-{}.webp", backend, id)).ok();
            panic!("{} backend: {:.2} difference", backend, diff);
        } else {
            result
                .save(format!("./tests/drawtest/successes/{}-{}.webp", backend, id))
                .unwrap();
            std::fs::remove_file(format!("./tests/drawtest/failures/{}-{}.webp", backend, id)).ok();
        }
    }
}

#[cfg(feature = "opengl")]
mod opengl {
    use super::CANVAS_SIZE;
    use image::{DynamicImage, Rgb, RgbImage};
    use picodraw::{Context, opengl::OpenGlBackend};
    use pugl_rs::{Event, OpenGl, OpenGlVersion, World};
    use std::sync::Arc;
    use std::{
        sync::mpsc::{TryRecvError, sync_channel},
        time::Duration,
    };

    pub fn render(width: u32, height: u32, render: Arc<dyn Fn(&mut dyn Context) + Send + Sync>) -> DynamicImage {
        let (sender, receiver) = sync_channel::<RgbImage>(1);
        let mut gl_backend = None;

        let mut world = World::new_program().unwrap();
        let view = world
            .new_view(OpenGl {
                bits_alpha: 0,
                bits_depth: 0,
                bits_stencil: 0,
                version: OpenGlVersion::Core(3, 3),
                debug: true,
                ..Default::default()
            })
            .with_size(CANVAS_SIZE, CANVAS_SIZE)
            .with_event_handler(move |view, event| match event {
                Event::Expose { backend, .. } => {
                    let mut gl_backend = unsafe {
                        gl_backend
                            .get_or_insert_with(|| OpenGlBackend::new(&|c| backend.get_proc_address(c)).unwrap())
                            .open()
                    };

                    render(&mut gl_backend);

                    {
                        let screenshot: Vec<u32> = gl_backend.screenshot([0, 0, width, height]);
                        let mut image = RgbImage::new(width, height);
                        for i in 0..width {
                            for j in 0..height {
                                let data = screenshot[(i + j * width) as usize];
                                image.put_pixel(
                                    i,
                                    height - 1 - j,
                                    Rgb([data as u8, (data >> 8) as u8, (data >> 16) as u8]),
                                );
                            }
                        }

                        let _ = sender.send(image);
                    }
                }
                Event::Update => {
                    view.obscure_view();
                }

                Event::Unrealize { .. } => {
                    if let Some(gl_backend) = gl_backend.take() {
                        unsafe {
                            gl_backend.delete();
                        }
                    }
                }

                _ => {}
            })
            .realize()
            .unwrap();

        view.show_passive();

        loop {
            world.update(Some(Duration::ZERO)).unwrap();
            match receiver.try_recv() {
                Ok(image) => return image.into(),
                Err(TryRecvError::Empty) => continue,
                Err(TryRecvError::Disconnected) => panic!("receiver disconnected"),
            }
        }
    }
}

#[cfg(feature = "software")]
mod software {
    use image::{DynamicImage, Rgb, RgbImage};
    use picodraw::{
        Context,
        software::{BufferMut, SoftwareBackend},
    };
    use std::sync::Arc;

    pub fn render(width: u32, height: u32, render: Arc<dyn Fn(&mut dyn Context) + Send + Sync>) -> DynamicImage {
        let mut backend = SoftwareBackend::new();
        let mut buffer = vec![0u32; (width * height) as usize];
        let mut context = backend.begin(BufferMut::from_slice(&mut buffer, width as usize, height as usize));

        render(&mut context);

        let mut image = RgbImage::new(width, height);
        for i in 0..width {
            for j in 0..height {
                let data = buffer[(i + j * width) as usize];
                image.put_pixel(i, j, Rgb([(data >> 16) as u8, (data >> 8) as u8, data as u8]));
            }
        }

        image.into()
    }
}
