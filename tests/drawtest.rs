use image::{DynamicImage, GenericImageView, Rgba, open};
use picodraw::{
    CommandBuffer, Context, Graph, ImageData, ImageFormat, RenderTexture, ShaderData, ShaderDataWriter, Texture,
    TextureFilter, shader::*,
};
use std::{f32::consts::PI, sync::Arc};

const MAX_CANVAS_SIZE: u32 = 512;

macro_rules! gen_simple {
    ($id:ident, $width:expr, $height:expr, $render:block) => {
        #[test]
        fn $id() {
            run(stringify!($id), $width, $height, |context| {
                let shader = context.create_shader(Graph::collect(|| {
                    let z = $render;
                    float4((z.x(), z.y(), z.z(), 1.0))
                }));

                let mut commands = CommandBuffer::new();
                commands
                    .begin_screen([$width, $height])
                    .begin_quad(shader, [0, 0, $width, $height]);
                context.draw(&commands);
            });
        }
    };
}

macro_rules! gen_serialize {
    ($id:ident, $width:expr, $height:expr, $generate:expr, $render:expr) => {
        #[test]
        fn $id() {
            fn imp<T: ShaderData>(
                context: &mut dyn Context,
                width: u32,
                height: u32,
                value: T,
                render: impl Fn(T::Data) -> float3,
            ) {
                let shader = context.create_shader(Graph::collect(|| {
                    let z = render(io::read::<T>());
                    float4((z.x(), z.y(), z.z(), 1.0))
                }));

                let mut commands = CommandBuffer::new();
                commands
                    .begin_screen([width, height])
                    .begin_quad(shader, [0, 0, width, height])
                    .write_data(&value);
                context.draw(&commands);
            }

            run(stringify!($id), $width, $height, |context| {
                imp(context, $width, $height, $generate, $render);
            });
        }
    };
}

pub mod ser {
    use super::*;

    struct TestStruct {
        x: f32,
        y: u8,
        z: (f32, f32),
    }

    struct TestStructShader {
        x: float1,
        y: float1,
        z: (float1, float1),
    }

    impl ShaderData for TestStruct {
        type Data = TestStructShader;

        fn read() -> Self::Data {
            let x = io::read::<f32>();
            let y = float1(io::read::<u8>()) / 255.0;
            let z = io::read::<(f32, f32)>();
            TestStructShader { x, y, z }
        }

        fn write(&self, writer: &mut dyn ShaderDataWriter) {
            self.x.write(writer);
            self.y.write(writer);
            self.z.write(writer);
        }
    }

    gen_serialize!(ser_u32, 4, 4, 0xCAFEBABEu32, |x| {
        float3((
            float1(x & 255) / 255.0,
            float1((x >> 8) & 255) / 255.0,
            float1((x >> 16) & 255) / 255.0,
        ))
    });

    gen_serialize!(ser_i32, 4, 4, 0xCAFEBEEFu32 as i32, |x| {
        float3((
            float1(x & 255) / 255.0,
            float1((x >> 8) & 255) / 255.0,
            x.le(0).select(float1(1.0), 0.0),
        ))
    });

    gen_serialize!(ser_u16, 4, 4, 0xCAFEu16, |x| {
        float3((float1(x & 255) / 255.0, float1((x >> 8) & 255) / 255.0, 1.0))
    });

    gen_serialize!(ser_i16, 4, 4, 0xBABEu16 as i16, |x| {
        float3((
            float1(x & 255) / 255.0,
            float1((x >> 8) & 255) / 255.0,
            x.le(0).select(float1(1.0), 0.0),
        ))
    });

    gen_serialize!(ser_u8, 4, 4, 0xCAu8, |x| {
        float3((float1(x & 16) / 16.0, float1((x >> 4) & 16) / 16.0, 1.0))
    });

    gen_serialize!(ser_i8, 4, 4, 0xEFu8 as i8, |x| {
        float3((
            float1(x & 16) / 16.0,
            float1((x >> 4) & 16) / 16.0,
            x.le(0).select(float1(1.0), 0.0),
        ))
    });

    gen_serialize!(ser_bool, 4, 4, true, |x| {
        float3((
            x.select(float1(1.0), 0.0),
            x.select(float1(0.5), 0.0),
            x.select(float1(0.25), 0.0),
        ))
    });

    gen_serialize!(ser_f32_pos, 4, 4, 0.3333333f32, |x| { float3((x, 2.0 * x, 3.0 * x)) });
    gen_serialize!(ser_f32_neg, 4, 4, -0.3333333f32, |x| { float3((x, -2.0 * x, 3.0 * x)) });
    gen_serialize!(ser_f32_zero, 4, 4, 0.0, |x| { float3((x, 1.0f32 - x, 0.5f32 - x)) });
    gen_serialize!(ser_f32_inf_pos, 4, 4, f32::INFINITY, |x| { float3((x, -x, x)) });
    gen_serialize!(ser_f32_inf_neg, 4, 4, f32::NEG_INFINITY, |x| { float3((x, -x, x)) });
    gen_serialize!(ser_f32_nan, 4, 4, f32::NAN, |x| {
        float3(x.eq(x).select(float1(0.0), 1.0))
    });

    gen_serialize!(ser_tuple, 4, 4, (0.2, 0.3, 0.5), |x| { float3((x.0, x.1, x.2)) });

    gen_serialize!(
        ser_struct,
        4,
        4,
        TestStruct {
            x: 0.6666666666666,
            y: 0xCAu8,
            z: (0.25, 0.5)
        },
        |x| { float3((x.x, x.y, x.z.0 * 0.5 + x.z.1 * 0.5,)) }
    );
}

#[cfg(feature = "derive")]
pub mod ser_derive {
    use super::*;

    #[derive(ShaderData)]
    struct UnitStruct;

    #[derive(ShaderData)]
    struct TupleStruct(f32, f32);

    #[derive(ShaderData)]
    struct TestStruct {
        a: f32,
        b: f32,
        c: u8,
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
    struct TestStructWithEncoders {
        normal: f32,

        #[shader(PassthroughEncoder::<f32>)]
        passthrough: f32,

        #[shader(FracResolutionEncoder)]
        resolution: (f32, f32),

        #[shader(FracQuadBoundsEncoder)]
        quad_bounds: (f32, f32),
    }

    gen_serialize!(ser_derive_unit_struct, 4, 4, UnitStruct, |_| { float3(1.0) });

    gen_serialize!(ser_derive_tuple_struct, 4, 4, TupleStruct(0.5, 0.2), |x| {
        float3((x.0, x.1, 1.0))
    });

    gen_serialize!(
        ser_derive_struct,
        4,
        4,
        TestStruct {
            a: 0.5,
            b: 0.2,
            c: 0xCA
        },
        |x| { float3((x.a, x.b, float1(x.c) / 255.0)) }
    );

    gen_serialize!(
        ser_derive_struct_with_encoders,
        4,
        4,
        TestStructWithEncoders {
            normal: 0.5,
            passthrough: 0.7,
            resolution: (2.0, 2.6),
            quad_bounds: (1.0, 3.0),
        },
        |x| {
            float3((
                x.normal * x.passthrough.0,
                x.resolution.x() * x.quad_bounds.x(),
                x.resolution.y() * x.quad_bounds.y(),
            ))
        }
    );
}

pub mod ops {
    use super::*;

    gen_simple!(op_nothing, 4, 4, { float3((1.0, 0.0, 1.0)) });

    gen_simple!(op_infinity, 4, 4, {
        let pos_inf = float1(f32::INFINITY);
        let neg_inf = float1(f32::NEG_INFINITY);
        let nan = float1(f32::NAN);

        float3((pos_inf, neg_inf, nan.eq(nan).select(float1(0.0), 1.0)))
    });

    gen_simple!(op_comp_ge, 64, 64, {
        let p = io::position() / io::resolution();

        float3((
            p.x().ge(p.y()).select(float1(1.0), 0.0),
            p.x().ge(0.5).select(float1(1.0), 0.0),
            p.y().ge(0.5).select(float1(1.0), 0.0),
        ))
    });

    gen_simple!(op_comp_le, 64, 64, {
        let p = io::position() / io::resolution();

        float3((
            p.x().le(p.y()).select(float1(1.0), 0.0),
            p.x().le(0.5).select(float1(1.0), 0.0),
            p.y().le(0.5).select(float1(1.0), 0.0),
        ))
    });

    gen_simple!(op_comp_gt, 64, 64, {
        let p = io::position() / io::resolution();

        float3((
            p.x().gt(p.y()).select(float1(1.0), 0.0),
            p.x().gt(0.5).select(float1(1.0), 0.0),
            p.y().gt(0.5).select(float1(1.0), 0.0),
        ))
    });

    gen_simple!(op_comp_lt, 64, 64, {
        let p = io::position() / io::resolution();

        float3((
            p.x().lt(p.y()).select(float1(1.0), 0.0),
            p.x().lt(0.5).select(float1(1.0), 0.0),
            p.y().lt(0.5).select(float1(1.0), 0.0),
        ))
    });

    gen_simple!(op_comp_eq, 64, 64, {
        let p = io::position() / io::resolution();

        float3((
            p.x().eq(p.y()).select(float1(1.0), 0.0),
            p.x().eq(0.5).select(float1(1.0), 0.0),
            p.y().eq(0.5).select(float1(1.0), 0.0),
        ))
    });

    gen_simple!(op_comp_ne, 64, 64, {
        let p = io::position() / io::resolution();

        float3((
            p.x().ne(p.y()).select(float1(1.0), 0.0),
            p.x().ne(0.5).select(float1(1.0), 0.0),
            p.y().ne(0.5).select(float1(1.0), 0.0),
        ))
    });

    gen_simple!(op_comp_ge_int, 64, 64, {
        let p = (io::position() / io::resolution()) * 16.0 - 8.0;

        float3((
            int1(p.x()).ge(int1(p.y())).select(float1(1.0), 0.0),
            int1(p.x()).ge(0).select(float1(1.0), 0.0),
            int1(p.y()).ge(0).select(float1(1.0), 0.0),
        ))
    });

    gen_simple!(op_comp_le_int, 64, 64, {
        let p = (io::position() / io::resolution()) * 16.0 - 8.0;

        float3((
            int1(p.x()).le(int1(p.y())).select(float1(1.0), 0.0),
            int1(p.x()).le(0).select(float1(1.0), 0.0),
            int1(p.y()).le(0).select(float1(1.0), 0.0),
        ))
    });

    gen_simple!(op_comp_gt_int, 64, 64, {
        let p = (io::position() / io::resolution()) * 16.0 - 8.0;

        float3((
            int1(p.x()).gt(int1(p.y())).select(float1(1.0), 0.0),
            int1(p.x()).gt(0).select(float1(1.0), 0.0),
            int1(p.y()).gt(0).select(float1(1.0), 0.0),
        ))
    });

    gen_simple!(op_comp_lt_int, 64, 64, {
        let p = (io::position() / io::resolution()) * 16.0 - 8.0;

        float3((
            int1(p.x()).lt(int1(p.y())).select(float1(1.0), 0.0),
            int1(p.x()).lt(0).select(float1(1.0), 0.0),
            int1(p.y()).lt(0).select(float1(1.0), 0.0),
        ))
    });

    gen_simple!(op_comp_eq_int, 64, 64, {
        let p = (io::position() / io::resolution()) * 16.0 - 8.0;

        float3((
            int1(p.x()).eq(int1(p.y())).select(float1(1.0), 0.0),
            int1(p.x()).eq(0).select(float1(1.0), 0.0),
            int1(p.y()).eq(0).select(float1(1.0), 0.0),
        ))
    });

    gen_simple!(op_comp_ne_int, 64, 64, {
        let p = (io::position() / io::resolution()) * 16.0 - 8.0;

        float3((
            int1(p.x()).ne(int1(p.y())).select(float1(1.0), 0.0),
            int1(p.x()).ne(0).select(float1(1.0), 0.0),
            int1(p.y()).ne(0).select(float1(1.0), 0.0),
        ))
    });

    gen_simple!(op_sin, 64, 8, {
        let x = (io::position() / io::resolution()).x() * 10.0 - 5.0;
        float3(x.sin())
    });

    gen_simple!(op_cos, 64, 8, {
        let x = (io::position() / io::resolution()).x() * 10.0 - 5.0;
        float3(x.cos())
    });

    gen_simple!(op_tan, 64, 8, {
        let x = (io::position() / io::resolution()).x() * 10.0 - 5.0;
        float3(x.tan())
    });

    gen_simple!(op_asin, 64, 8, {
        let x = (io::position() / io::resolution()).x() * 10.0 - 5.0;
        float3(x.asin())
    });

    gen_simple!(op_acos, 64, 8, {
        let x = (io::position() / io::resolution()).x() * 10.0 - 5.0;
        float3(x.acos())
    });

    gen_simple!(op_atan, 64, 8, {
        let x = (io::position() / io::resolution()).x() * 10.0 - 5.0;
        float3(x.atan())
    });

    gen_simple!(op_exp, 128, 8, {
        let x = (io::position() / io::resolution()).x() * 10.0 - 5.0;
        float3(x.exp())
    });

    gen_simple!(op_sqrt, 128, 8, {
        let x = (io::position() / io::resolution()).x() * 10.0 - 5.0;
        float3(x.sqrt())
    });

    gen_simple!(op_ln, 128, 8, {
        let x = (io::position() / io::resolution()).x() * 10.0 - 5.0;
        float3(x.ln())
    });

    gen_simple!(op_pow, 64, 64, {
        let p = (io::position() / io::resolution()) * 4.0;

        float3((p.x().pow(p.y()), p.x().pow(-p.y()), (-p.x()).pow(2.0)))
    });

    gen_simple!(op_cast, 128, 8, {
        let x = (io::position() / io::resolution()).x() * 10.0 - 5.0;
        let y = int1(x);
        float3((
            (float1(y) - x).abs(),
            float1(y % 2) * 0.5 + 0.5,
            (float1(y) + 5.0) / 10.0,
        ))
    });

    gen_simple!(op_floor, 128, 8, {
        let x = (io::position() / io::resolution()).x() * 10.0 - 5.0;
        float3((x.floor() + 5.0) / 10.0)
    });

    gen_simple!(op_abs, 128, 8, {
        let x = (io::position() / io::resolution()).x() * 10.0 - 5.0;
        (float3((x.abs(), x.len(), 0.0)) + 5.0) / 10.0
    });

    gen_simple!(op_sign, 128, 8, {
        let x = (io::position() / io::resolution()).x() * 10.0 - 5.0;
        float3((x.sign(), x.norm(), 0.0))
    });

    gen_simple!(op_step, 128, 8, {
        let x = (io::position() / io::resolution()).x() * 10.0 - 5.0;
        float3((x.step(0.0), x.step(1.0), x.step(-1.0)))
    });

    gen_simple!(op_smoothstep, 128, 8, {
        let x = (io::position() / io::resolution()).x() * 10.0 - 5.0;
        float3((x.smoothstep(0.0, 1.0), x.smoothstep(1.0, -1.0), x.smoothstep(-1.0, 0.0)))
    });

    gen_simple!(op_min, 128, 8, {
        let x = (io::position() / io::resolution()).x() * 10.0 - 5.0;
        (float3((x.min(0.0), x.min(1.0), x.min(-1.0))) + 5.0) / 10.0
    });

    gen_simple!(op_max, 128, 8, {
        let x = (io::position() / io::resolution()).x() * 10.0 - 5.0;
        (float3((x.max(0.0), x.max(1.0), x.max(-1.0))) + 5.0) / 10.0
    });

    gen_simple!(op_clamp, 128, 8, {
        let x = (io::position() / io::resolution()).x() * 10.0 - 5.0;
        (float3((x.clamp(0.0, 1.0), x.clamp(-1.0, 1.0), x.clamp(-1.0, 0.0))) + 5.0) / 10.0
    });

    gen_simple!(op_lerp, 128, 8, {
        let x = (io::position() / io::resolution()).x();
        float3((x.lerp(0.5, 1.0), x.lerp(1.0, 0.0), float1(0.5).lerp(x, 0.5)))
    });

    gen_simple!(op_bit_and, 32, 32, {
        let x = int1(io::position().x());
        let y = int1(io::position().y());

        float3((
            float1((x & y) % 16) / 16.0,
            float1((x & y) % 32) / 32.0,
            float1((x & y) % 64) / 64.0,
        ))
    });

    gen_simple!(op_bit_or, 32, 32, {
        let x = int1(io::position().x());
        let y = int1(io::position().y());

        float3((
            float1((x | y) % 16) / 16.0,
            float1((x | y) % 32) / 32.0,
            float1((x | y) % 64) / 64.0,
        ))
    });

    gen_simple!(op_bit_xor, 32, 32, {
        let x = int1(io::position().x());
        let y = int1(io::position().y());

        float3((
            float1((x ^ y) % 16) / 16.0,
            float1((x ^ y) % 32) / 32.0,
            float1((x ^ y) % 64) / 64.0,
        ))
    });

    gen_simple!(op_bit_not, 32, 32, {
        let x = int1(io::position().x());
        let y = int1(io::position().y());

        float3((
            float1(!x % 32) / 32.0,
            float1(!y % 32) / 32.0,
            float1(!(x + y) % 32) / 32.0,
        ))
    });

    gen_simple!(op_bit_shl, 32, 32, {
        let x = int1(io::position().x());
        let y = int1(io::position().y()) / 4;

        float3((
            float1((x << y) % 16) / 16.0,
            float1((x << y) % 32) / 32.0,
            float1((x << y) % 64) / 64.0,
        ))
    });

    gen_simple!(op_bit_shr, 32, 32, {
        let x = int1(io::position().x());
        let y = int1(io::position().y()) / 4;

        float3((
            float1((x >> y) % 16) / 16.0,
            float1((x >> y) % 32) / 32.0,
            float1((x >> y) % 64) / 64.0,
        ))
    });

    gen_simple!(op_dydx, 64, 64, {
        let p = io::position() / io::resolution();
        let z = (p.x() * 10.0).sin() * (p.y() * 10.0).cos();

        float3((z.dx() + 0.5, z.dy() + 0.5, z.fwidth()))
    });

    gen_simple!(op_atan2, 64, 64, {
        let p = (io::position() / io::resolution()) * 2.0 - 1.0;
        float3(p.x().atan2(p.y()) / PI * 0.5 + 0.5)
    });

    gen_simple!(op_norm2, 64, 64, {
        let p = (io::position() / io::resolution()) * 2.0 - 1.0;
        float3((p.norm().x() * 0.5 + 0.5, p.norm().y() * 0.5 + 0.5, p.len()))
    });

    gen_simple!(op_dot2, 64, 64, {
        let p = (io::position() / io::resolution()) * 2.0 - 1.0;
        let p = float2((p.x(), p.y()));
        float3((
            p.dot((1.0, 1.0)) * 0.5 + 0.5,
            p.dot((0.0, 1.0)) * 0.5 + 0.5,
            p.dot((1.0, 0.0)) * 0.5 + 0.5,
        ))
    });

    gen_simple!(op_cross3, 64, 64, {
        let p = (io::position() / io::resolution()) * 2.0 - 1.0;
        let p = float3((p.x(), p.y(), 0.0));
        p.cross((1.0, 1.0, 1.0)) * 0.5 + 0.5
    });
}

pub mod texture {
    use super::*;

    const TEST_DITHER0: [u8; 16] = [
        0 * 16,
        8 * 16,
        2 * 16,
        10 * 16,
        12 * 16,
        4 * 16,
        14 * 16,
        6 * 16,
        3 * 16,
        11 * 16,
        1 * 16,
        9 * 16,
        15 * 16,
        7 * 16,
        13 * 16,
        5 * 16,
    ];

    #[test]
    fn texture_static_nearest() {
        run("texture_static_nearest", 32, 32, |context| {
            let texture = context.create_texture_static(ImageData {
                width: 4,
                height: 4,
                format: ImageFormat::R8,
                data: &TEST_DITHER0,
            });

            let shader = context.create_shader(Graph::collect(|| {
                let texture = io::read::<Texture>();
                let uv = io::position() / io::resolution();
                texture.sample(uv * float2(texture.size()), TextureFilter::Linear)
            }));

            let mut commands = CommandBuffer::new();
            commands
                .begin_screen([32, 32])
                .begin_quad(shader, [0, 0, 32, 32])
                .write_data(texture);
            context.draw(&commands);
        });
    }

    #[test]
    fn texture_static_linear() {
        run("texture_static_linear", 32, 32, |context| {
            let texture = context.create_texture_static(ImageData {
                width: 4,
                height: 4,
                format: ImageFormat::R8,
                data: &TEST_DITHER0,
            });

            let shader = context.create_shader(Graph::collect(|| {
                let texture = io::read::<Texture>();
                let uv = io::position() / io::resolution();
                texture.sample(uv * float2(texture.size()), TextureFilter::Linear)
            }));

            let mut commands = CommandBuffer::new();
            commands
                .begin_screen([32, 32])
                .begin_quad(shader, [0, 0, 32, 32])
                .write_data(texture);
            context.draw(&commands);
        });
    }

    #[test]
    fn texture_render_nearest() {
        run("texture_render_nearest", 32, 32, |context| {
            let texture = context.create_texture_render();

            let shader_fill = context.create_shader(Graph::collect(|| {
                let a = float4((1.0, 0.5, 0.25, 1.0));
                let b = float4((0.5, 0.25, 1.0, 1.0));
                let p = io::position() / io::resolution();
                let p = p.dot((0.707, 0.707));

                float4(p).lerp(a, b)
            }));

            let shader_negative = context.create_shader(Graph::collect(|| {
                let texture = io::read::<Texture>();
                let z = texture.sample(
                    io::position() / io::resolution() * float2(texture.size()),
                    TextureFilter::Linear,
                );

                float4((1.0 - z.x(), 1.0 - z.y(), 1.0 - z.z(), z.w()))
            }));

            let mut commands = CommandBuffer::new();

            commands
                .begin_buffer(texture, [4, 4])
                .begin_quad(shader_fill, [0, 0, 4, 4]);

            commands
                .begin_screen([32, 32])
                .begin_quad(shader_negative, [0, 0, 20, 32])
                .write_data(texture);

            context.draw(&commands);
        });
    }

    #[test]
    fn texture_render_linear() {
        run("texture_render_linear", 32, 32, |context| {
            let texture = context.create_texture_render();

            let shader_fill = context.create_shader(Graph::collect(|| {
                let a = float4((1.0, 0.5, 0.25, 1.0));
                let b = float4((0.5, 0.25, 1.0, 1.0));
                let p = io::position() / io::resolution();
                let p = p.dot((0.707, 0.707));

                float4(p).lerp(a, b)
            }));

            let shader_negative = context.create_shader(Graph::collect(|| {
                let texture = io::read::<Texture>();
                let z = texture.sample(
                    io::position() / io::resolution() * float2(texture.size()),
                    TextureFilter::Linear,
                );

                float4((1.0 - z.x(), 1.0 - z.y(), 1.0 - z.z(), z.w()))
            }));

            let mut commands = CommandBuffer::new();

            commands
                .begin_buffer(texture, [4, 4])
                .begin_quad(shader_fill, [0, 0, 4, 4]);

            commands
                .begin_screen([32, 32])
                .begin_quad(shader_negative, [0, 0, 20, 32])
                .write_data(texture);

            context.draw(&commands);
        });
    }

    #[test]
    fn texture_load_r8() {
        run("texture_load_r8", 4, 4, |context| {
            let texture = context.create_texture_static(ImageData {
                width: 1,
                height: 1,
                format: ImageFormat::R8,
                data: &[100],
            });

            let shader = context.create_shader(Graph::collect(|| {
                let texture = io::read::<Texture>();
                texture.sample(0.0, TextureFilter::Nearest)
            }));

            let mut commands = CommandBuffer::new();
            commands
                .begin_screen([4, 4])
                .begin_quad(shader, [0, 0, 4, 4])
                .write_data(texture);
            context.draw(&commands);
        });
    }

    #[test]
    fn texture_load_rgb8() {
        run("texture_load_rgb8", 4, 4, |context| {
            let texture = context.create_texture_static(ImageData {
                width: 1,
                height: 1,
                format: ImageFormat::RGB8,
                data: &[100, 50, 200],
            });

            let shader = context.create_shader(Graph::collect(|| {
                let texture = io::read::<Texture>();
                texture.sample(0.0, TextureFilter::Nearest)
            }));

            let mut commands = CommandBuffer::new();
            commands
                .begin_screen([4, 4])
                .begin_quad(shader, [0, 0, 4, 4])
                .write_data(texture);
            context.draw(&commands);
        });
    }

    #[test]
    fn texture_load_rgba8() {
        run("texture_load_rgba8", 4, 4, |context| {
            let texture = context.create_texture_static(ImageData {
                width: 1,
                height: 1,
                format: ImageFormat::RGBA8,
                data: &[100, 50, 200, 150],
            });

            let shader = context.create_shader(Graph::collect(|| {
                let texture = io::read::<Texture>();
                texture.sample(0.0, TextureFilter::Nearest)
            }));

            let mut commands = CommandBuffer::new();
            commands
                .begin_screen([4, 4])
                .begin_quad(shader, [0, 0, 4, 4])
                .write_data(texture);
            context.draw(&commands);
        });
    }
}

pub mod semantics {
    use super::*;

    #[test]
    fn semantics_blend() {
        run("semantics_blend", 8, 8, |context| {
            let shader = context.create_shader(Graph::collect(|| {
                let data = io::read::<[f32; 4]>();
                float4((data[0], data[1], data[2], data[3]))
            }));

            let mut commands = CommandBuffer::new();
            let mut screen = commands.begin_screen([8, 8]);

            screen
                .begin_quad(shader, [1, 1, 5, 5])
                .write_data([1.0, 0.0, 0.0, 0.50]);
            screen
                .begin_quad(shader, [3, 3, 7, 7])
                .write_data([0.0, 1.0, 1.0, 0.25]);
            screen.begin_quad(shader, [0, 0, 8, 8]).write_data([1.0, 1.0, 1.0, 0.1]);

            context.draw(&commands);
        });
    }

    #[test]
    fn semantics_clear() {
        run("semantics_clear", 8, 8, |context| {
            let shader = context.create_shader(Graph::collect(|| {
                let data = io::read::<[f32; 4]>();
                float4((data[0], data[1], data[2], data[3]))
            }));

            let mut commands = CommandBuffer::new();
            let mut screen = commands.begin_screen([8, 8]);

            screen
                .begin_quad(shader, [1, 1, 7, 7])
                .write_data([1.0, 0.0, 0.0, 0.50]);
            screen.clear([4, 4, 8, 8]);

            context.draw(&commands);
        });
    }

    #[test]
    fn semantics_screen_preserve() {
        run("semantics_screen_preserve", 8, 8, |context| {
            let shader = context.create_shader(Graph::collect(|| {
                let data = io::read::<[f32; 4]>();
                float4((data[0], data[1], data[2], data[3]))
            }));

            let mut commands = CommandBuffer::new();
            let mut screen = commands.begin_screen([8, 8]);
            screen
                .begin_quad(shader, [1, 1, 7, 7])
                .write_data([1.0, 0.0, 0.0, 0.50]);
            context.draw(&commands);

            let mut screen = commands.begin_screen([8, 8]);
            screen.clear([4, 4, 8, 8]);
            screen
                .begin_quad(shader, [1, 1, 7, 7])
                .write_data([0.0, 1.0, 1.0, 0.25]);
            context.draw(&commands);
        });
    }
}

pub mod stress {
    use super::*;

    #[test]
    fn stress_fill_rate() {
        run("stress_fill_rate", MAX_CANVAS_SIZE, MAX_CANVAS_SIZE, move |context| {
            let shader = context.create_shader(Graph::collect(|| {
                let i = io::read::<i32>();
                let j = int1(io::position().x()) + int1(io::position().y()) * int1(io::resolution().x());
                float4((1.0, 1.0, 1.0, (j % i).eq(0).select(float1(i).sqrt() / 255.0, 0.0)))
            }));

            for _ in 0..2 {
                let mut commands = CommandBuffer::new();
                let mut frame = commands.begin_screen([MAX_CANVAS_SIZE, MAX_CANVAS_SIZE]);
                frame.clear([0, 0, MAX_CANVAS_SIZE, MAX_CANVAS_SIZE]);

                for i in 2..500 {
                    frame
                        .begin_quad(shader, [0, 0, MAX_CANVAS_SIZE, MAX_CANVAS_SIZE])
                        .write_data(i);
                }

                context.draw(&commands);
            }
        });
    }

    #[test]
    fn stress_quad_count() {
        run("stress_quad_count", MAX_CANVAS_SIZE, MAX_CANVAS_SIZE, move |context| {
            let shader = context.create_shader(Graph::collect(|| {
                let [r, g, b] = io::read::<[f32; 3]>();
                io::read::<[u32; 8]>();
                float4((r, g, b, 1.0))
            }));

            for _ in 0..2 {
                let mut commands = CommandBuffer::new();
                let mut frame = commands.begin_screen([MAX_CANVAS_SIZE, MAX_CANVAS_SIZE]);
                frame.clear([0, 0, MAX_CANVAS_SIZE, MAX_CANVAS_SIZE]);

                for i in 0..MAX_CANVAS_SIZE {
                    for j in 0..MAX_CANVAS_SIZE {
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

    #[test]
    fn stress_shader_complexity() {
        run("stress_shader_complexity", 4, 4, move |context| {
            let shader = context.create_shader(Graph::collect(|| {
                let mut a = float4((1.0, 0.5, 0.25, 1.0));
                for _ in 0..1000 {
                    a = a * float4((0.999, 1.0, 1.001, 1.0));
                }
                a
            }));

            let mut commands = CommandBuffer::new();
            commands.begin_screen([4, 4]).begin_quad(shader, [0, 0, 4, 4]);
            context.draw(&commands);
        });
    }
}

pub mod complex {
    use super::*;

    #[test]
    fn complex_msdf() {
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

        run("complex_msdf", MAX_CANVAS_SIZE, MAX_CANVAS_SIZE, move |context| {
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
                let sample = atlas.sample(pos.clamp(0.0, atlas.size()), TextureFilter::Linear);
                let median = float1::max(
                    float1::min(sample.x(), sample.y()),
                    float1::min(float1::max(sample.x(), sample.y()), sample.z()),
                );

                let mask = (((2.0 * scale).max(1.0) * (median - 0.5)) + 0.5).smoothstep(0.0, 1.0);
                float4(mask)
            }));

            let mut commands = CommandBuffer::new();
            let mut frame = commands.begin_screen([MAX_CANVAS_SIZE, MAX_CANVAS_SIZE]);

            let mut x = 10.0;
            let mut scale = 0.5;
            for _ in 0..=10 {
                frame
                    .begin_quad(shader, [0, 0, MAX_CANVAS_SIZE, MAX_CANVAS_SIZE])
                    .write_data(texture)
                    .write_data((x, 12.0 + scale * 10.0))
                    .write_data(scale);

                x += 18.0 * scale;
                scale *= 1.325;
            }

            frame
                .begin_quad(shader, [0, 0, MAX_CANVAS_SIZE, MAX_CANVAS_SIZE])
                .write_data(texture)
                .write_data((256.0, 320.0))
                .write_data(20.0);

            context.draw(&commands);
        });
    }

    #[test]
    fn complex_boxblur() {
        run("complex_boxblur", MAX_CANVAS_SIZE, MAX_CANVAS_SIZE, |context| {
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
                        result = result + buffer.sample(io::position() + float2((i, j)), TextureFilter::Nearest);
                    }
                }

                result / (11 * 11) as f32
            }));

            let buffer = context.create_texture_render();
            let mut commands = CommandBuffer::new();

            commands
                .begin_buffer(buffer, [MAX_CANVAS_SIZE, MAX_CANVAS_SIZE])
                .begin_quad(shader_circle, [0, 0, MAX_CANVAS_SIZE, MAX_CANVAS_SIZE])
                .write_data([256.0, 256.0]);

            commands
                .begin_screen([MAX_CANVAS_SIZE, MAX_CANVAS_SIZE])
                .begin_quad(shader_boxblur, [0, 0, MAX_CANVAS_SIZE, MAX_CANVAS_SIZE])
                .write_data(buffer);

            context.draw(&commands);
        });
    }
}

#[allow(unused_variables, dead_code)]
fn run(id: &str, width: u32, height: u32, render: impl Fn(&mut dyn Context) + Sync + Send + 'static) {
    fn difference(a: &DynamicImage, b: &DynamicImage) -> f64 {
        let mut sum = 0;
        for i in 0..a.width() {
            for j in 0..a.height() {
                let Rgba([r0, g0, b0, a0]) = a.get_pixel(i, j);
                let Rgba([r1, g1, b1, a1]) = b.get_pixel(i, j);

                let r0 = r0 as u64;
                let g0 = g0 as u64;
                let b0 = b0 as u64;
                let a0 = a0 as u64;
                let r1 = r1 as u64;
                let g1 = g1 as u64;
                let b1 = b1 as u64;
                let a1 = a1 as u64;

                sum += (r0 * a0).abs_diff(r1 * a1) / (a0 * a1).max(1);
                sum += (g0 * a0).abs_diff(g1 * a1) / (a0 * a1).max(1);
                sum += (b0 * a0).abs_diff(b1 * a1) / (a0 * a1).max(1);
                sum += a0.abs_diff(a1);
            }
        }

        sum as f64 / (a.height() * a.width()) as f64
    }

    if cfg!(miri) {
        return;
    }

    let renderer = Arc::new(render);
    let expected = open(format!("./tests/drawtest/expected/{}.webp", id)).ok();
    let results: Vec<(&'static str, DynamicImage)> = vec![
        #[cfg(feature = "software")]
        ("software", software::render(width, height, renderer.clone())),
        #[cfg(feature = "opengl")]
        ("opengl", opengl::render(width, height, renderer.clone())),
    ];

    let mut failures = vec![];
    for (backend, result) in results {
        let failure = match &expected {
            Some(expected) => {
                if expected.width() != result.width() || expected.height() != result.height() {
                    Some(format!(
                        "expected image size {}x{} but got {}x{}",
                        expected.width(),
                        expected.height(),
                        result.width(),
                        result.height()
                    ))
                } else {
                    let diff = difference(&result, &expected);
                    if diff > 6.0 {
                        Some(format!("diff: {}", diff))
                    } else {
                        None
                    }
                }
            }
            None => Some("no expected image".to_string()),
        };

        std::fs::create_dir_all(format!("./tests/drawtest/failures/{}/", backend)).ok();
        std::fs::create_dir_all(format!("./tests/drawtest/successes/{}/", backend)).ok();

        if let Some(failure) = failure {
            failures.push((backend, failure));
            result
                .save(format!("./tests/drawtest/failures/{}/{}.webp", backend, id))
                .unwrap();
            std::fs::remove_file(format!("./tests/drawtest/successes/{}/{}.webp", backend, id)).ok();
        } else {
            result
                .save(format!("./tests/drawtest/successes/{}/{}.webp", backend, id))
                .unwrap();
            std::fs::remove_file(format!("./tests/drawtest/failures/{}/{}.webp", backend, id)).ok();
        }
    }

    if let Some((backend, message)) = failures.first() {
        panic!(
            "Test {} failed on {} ({}). See ./tests/drawtest/failures/{}/{}.webp for the result.",
            id, backend, message, backend, id
        )
    }
}

#[cfg(feature = "opengl")]
mod opengl {
    use super::MAX_CANVAS_SIZE;
    use image::{DynamicImage, Rgba, RgbaImage};
    use picodraw::{CommandBuffer, Context, opengl::OpenGlBackend};
    use pugl_rs::{Event, OpenGl, OpenGlVersion, World};
    use std::sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicBool, Ordering},
    };
    use std::time::Duration;

    static JOB_QUEUE: Mutex<Vec<Arc<Job>>> = Mutex::new(Vec::new());
    struct Job {
        width: u32,
        height: u32,
        render: Arc<dyn Fn(&mut dyn Context) + Send + Sync>,
        result: Mutex<Option<DynamicImage>>,
        condvar: Condvar,
    }

    fn runner_thread() {
        let close = Arc::new(AtomicBool::new(false));
        let mut gl_backend = None;
        let mut world = World::new_program().unwrap();
        let close_send = close.clone();
        let view = world
            .new_view(OpenGl {
                bits_alpha: 8,
                bits_depth: 0,
                bits_stencil: 0,
                version: OpenGlVersion::Core(3, 3),
                debug: true,
                ..Default::default()
            })
            .with_size(MAX_CANVAS_SIZE, MAX_CANVAS_SIZE)
            .with_event_handler(move |view, event| match event {
                Event::Expose { backend, .. } => {
                    let job = match JOB_QUEUE.lock().unwrap().pop() {
                        Some(job) => job,
                        None => {
                            close_send.store(true, Ordering::SeqCst);
                            return;
                        }
                    };

                    let mut gl_backend = unsafe {
                        gl_backend
                            .get_or_insert_with(|| OpenGlBackend::new(&|c| backend.get_proc_address(c)).unwrap())
                            .open()
                    };

                    {
                        let mut commands = CommandBuffer::new();
                        commands.begin_screen([MAX_CANVAS_SIZE, MAX_CANVAS_SIZE]).clear([
                            0,
                            0,
                            MAX_CANVAS_SIZE,
                            MAX_CANVAS_SIZE,
                        ]);
                        gl_backend.draw(&commands);
                    }

                    (job.render)(&mut gl_backend);

                    {
                        let screenshot: Vec<u32> = gl_backend.screenshot([0, 0, job.width, job.height]);
                        let mut image = RgbaImage::new(job.width, job.height);
                        for i in 0..job.width {
                            for j in 0..job.height {
                                let data = screenshot[(i + j * job.width) as usize];
                                image.put_pixel(
                                    i,
                                    job.height - 1 - j,
                                    Rgba([data as u8, (data >> 8) as u8, (data >> 16) as u8, (data >> 24) as u8]),
                                );
                            }
                        }

                        job.result.lock().unwrap().replace(image.into());
                        job.condvar.notify_one();
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

        while !close.load(Ordering::SeqCst) {
            world.update(Some(Duration::ZERO)).unwrap();
        }
    }

    pub fn render(width: u32, height: u32, render: Arc<dyn Fn(&mut dyn Context) + Send + Sync>) -> DynamicImage {
        let job = Arc::new(Job {
            width,
            height,
            render,
            result: Mutex::new(None),
            condvar: Condvar::new(),
        });

        {
            let mut queue = JOB_QUEUE.lock().unwrap();
            if queue.len() < 4 {
                std::thread::spawn(runner_thread);
            }

            queue.push(job.clone());
        }

        let mut result = job.result.lock().unwrap();
        loop {
            if result.is_some() {
                break result.take().unwrap();
            }

            result = job.condvar.wait(result).unwrap();
        }
    }
}

#[cfg(feature = "software")]
mod software {
    use image::{DynamicImage, Rgba, RgbaImage};
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

        let mut image = RgbaImage::new(width, height);
        for i in 0..width {
            for j in 0..height {
                let data = buffer[(i + j * width) as usize];
                let (r, g, b, a) = picodraw::software::unpack_rgba(data);
                image.put_pixel(i, j, Rgba([r, g, b, a]));
            }
        }

        image.into()
    }
}
