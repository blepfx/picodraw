#![allow(unused_variables, unused_imports, dead_code)]

#[path = "./drawtest/runner.rs"]
mod runner;

use image::{GenericImageView, Rgba, open};
use picodraw::{shader::*, *};
use runner::{MAX_CANVAS_SIZE, run};
use std::f32::consts::PI;

macro_rules! gen_simple {
    ($id:ident, $width:expr, $height:expr, $render:block) => {
        #[test]
        fn $id() {
            run(stringify!($id), $width, $height, |context| {
                let shader = context.create_shader(Graph::trace(|| {
                    let z = $render;
                    float4((z.x(), z.y(), z.z(), 1.0))
                }));

                let mut commands = vec![];
                add_quad(&mut commands, shader, [0, 0, $width, $height], ());
                context.draw_screen(&commands).unwrap();
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
                let shader = context.create_shader(Graph::trace(|| {
                    let z = render(io::read::<T>());
                    float4((z.x(), z.y(), z.z(), 1.0))
                }));

                let mut commands = vec![];
                add_quad(&mut commands, shader, [0, 0, width, height], value);
                context.draw_screen(&commands).unwrap();
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

        fn write(&self, mut writer: impl ShaderDataWriter) {
            self.x.write(&mut writer);
            self.y.write(&mut writer);
            self.z.write(&mut writer);
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
            let texture = context.create_texture([4, 4].into(), TextureFormat::R8);

            assert!(context.upload_texture(
                texture,
                TextureData {
                    bounds: [0, 0, 4, 4].into(),
                    format: TextureFormat::R8,
                    data: &TEST_DITHER0,
                },
            ));
            let shader = context.create_shader(Graph::trace(|| {
                let texture = io::read::<TextureId>();
                let uv = io::position() / io::resolution();
                texture.sample(uv * float2(texture.size()), TextureFilter::Nearest)
            }));

            let mut commands = vec![];
            add_quad(&mut commands, shader, [0, 0, 32, 32], texture);
            context.draw_screen(&commands).unwrap();
        });
    }

    #[test]
    fn texture_static_linear() {
        run("texture_static_linear", 32, 32, |context| {
            let texture = context.create_texture([4, 4].into(), TextureFormat::R8);

            assert!(context.upload_texture(
                texture,
                TextureData {
                    bounds: [0, 0, 4, 4].into(),
                    format: TextureFormat::R8,
                    data: &TEST_DITHER0,
                },
            ));

            let shader = context.create_shader(Graph::trace(|| {
                let texture = io::read::<TextureId>();
                let uv = io::position() / io::resolution();
                texture.sample(uv * float2(texture.size()), TextureFilter::Linear)
            }));

            let mut commands = vec![];
            add_quad(&mut commands, shader, [0, 0, 32, 32], texture);
            context.draw_screen(&commands).unwrap();
        });
    }

    #[test]
    fn texture_render_nearest() {
        run("texture_render_nearest", 32, 32, |context| {
            let texture = context.create_texture([4, 4].into(), TextureFormat::RGBA8);

            let shader_fill = context.create_shader(Graph::trace(|| {
                let a = float4((1.0, 0.5, 0.25, 1.0));
                let b = float4((0.5, 0.25, 1.0, 1.0));
                let p = io::position() / io::resolution();
                let p = p.dot((0.707, 0.707));

                float4(p).lerp(a, b)
            }));

            let shader_negative = context.create_shader(Graph::trace(|| {
                let texture = io::read::<TextureId>();
                let z = texture.sample(
                    io::position() / io::resolution() * float2(texture.size()),
                    TextureFilter::Nearest,
                );

                float4((1.0 - z.x(), 1.0 - z.y(), 1.0 - z.z(), z.w()))
            }));

            let mut commands = vec![];
            add_quad(&mut commands, shader_fill, [0, 0, 4, 4], ());
            context.draw_texture(texture, &commands).unwrap();

            let mut commands = vec![];
            add_quad(&mut commands, shader_negative, [0, 0, 20, 32], texture);
            context.draw_screen(&commands).unwrap();
        });
    }

    #[test]
    fn texture_render_linear() {
        run("texture_render_linear", 32, 32, |context| {
            let texture = context.create_texture([4, 4].into(), TextureFormat::RGBA8);

            let shader_fill = context.create_shader(Graph::trace(|| {
                let a = float4((1.0, 0.5, 0.25, 1.0));
                let b = float4((0.5, 0.25, 1.0, 1.0));
                let p = io::position() / io::resolution();
                let p = p.dot((0.707, 0.707));

                float4(p).lerp(a, b)
            }));

            let shader_negative = context.create_shader(Graph::trace(|| {
                let texture = io::read::<TextureId>();
                let z = texture.sample(
                    io::position() / io::resolution() * float2(texture.size()),
                    TextureFilter::Linear,
                );

                float4((1.0 - z.x(), 1.0 - z.y(), 1.0 - z.z(), z.w()))
            }));

            let mut commands = vec![];
            add_quad(&mut commands, shader_fill, [0, 0, 4, 4], ());
            context.draw_texture(texture, &commands).unwrap();

            let mut commands = vec![];
            add_quad(&mut commands, shader_negative, [0, 0, 20, 32], texture);
            context.draw_screen(&commands).unwrap();
        });
    }

    #[test]
    fn texture_load_r8() {
        run("texture_load_r8", 4, 4, |context| {
            let texture = context.create_texture([1, 1].into(), TextureFormat::R8);

            assert!(context.upload_texture(
                texture,
                TextureData {
                    bounds: [0, 0, 1, 1].into(),
                    format: TextureFormat::R8,
                    data: &[100],
                },
            ));

            let shader = context.create_shader(Graph::trace(|| {
                let texture = io::read::<TextureId>();
                texture.sample(0.0, TextureFilter::Nearest)
            }));

            let mut commands = vec![];
            add_quad(&mut commands, shader, [0, 0, 4, 4], texture);
            context.draw_screen(&commands).unwrap();
        });
    }

    #[test]
    fn texture_load_rgb8() {
        run("texture_load_rgb8", 4, 4, |context| {
            let texture = context.create_texture([1, 1].into(), TextureFormat::RGB8);

            assert!(context.upload_texture(
                texture,
                TextureData {
                    bounds: [0, 0, 1, 1].into(),
                    format: TextureFormat::RGB8,
                    data: &[100, 50, 200],
                },
            ));

            let shader = context.create_shader(Graph::trace(|| {
                let texture = io::read::<TextureId>();
                texture.sample(0.0, TextureFilter::Nearest)
            }));

            let mut commands = vec![];
            add_quad(&mut commands, shader, [0, 0, 4, 4], texture);
            context.draw_screen(&commands).unwrap();
        });
    }

    #[test]
    fn texture_load_rgba8() {
        run("texture_load_rgba8", 4, 4, |context| {
            let texture = context.create_texture([1, 1].into(), TextureFormat::RGBA8);

            assert!(context.upload_texture(
                texture,
                TextureData {
                    bounds: [0, 0, 1, 1].into(),
                    format: TextureFormat::RGBA8,
                    data: &[100, 50, 200, 150],
                },
            ));

            let shader = context.create_shader(Graph::trace(|| {
                let texture = io::read::<TextureId>();
                texture.sample(0.0, TextureFilter::Nearest)
            }));

            let mut commands = vec![];
            add_quad(&mut commands, shader, [0, 0, 4, 4], texture);
            context.draw_screen(&commands).unwrap();
        });
    }
}

pub mod semantics {
    use super::*;

    #[test]
    fn semantics_alpha() {
        run("semantics_alpha", 64, 8, |context| {
            let shader = context.create_shader(Graph::trace(|| {
                let x = io::position().x() / io::resolution().x();
                float4((1.0, 1.0, 1.0, x))
            }));

            let mut commands = vec![];
            add_quad(&mut commands, shader, [0, 0, 64, 8], ());
            add_quad(&mut commands, shader, [0, 4, 64, 8], ());
            context.draw_screen(&commands).unwrap();
        });
    }

    #[test]
    fn semantics_blend() {
        run("semantics_blend", 8, 8, |context| {
            let shader = context.create_shader(Graph::trace(|| {
                let data = io::read::<[f32; 4]>();
                float4((data[0], data[1], data[2], data[3]))
            }));

            let mut commands = vec![];
            add_quad(&mut commands, shader, [1, 1, 5, 5], [1.0, 0.0, 0.0, 0.50]);
            add_quad(&mut commands, shader, [3, 3, 7, 7], [0.0, 1.0, 1.0, 0.25]);
            add_quad(&mut commands, shader, [0, 0, 8, 8], [1.0, 1.0, 1.0, 0.1]);
            context.draw_screen(&commands).unwrap();
        });
    }

    #[test]
    fn semantics_clear() {
        run("semantics_clear", 8, 8, |context| {
            let shader = context.create_shader(Graph::trace(|| {
                let data = io::read::<[f32; 4]>();
                float4((data[0], data[1], data[2], data[3]))
            }));

            let mut commands = vec![];
            add_quad(&mut commands, shader, [1, 1, 7, 7], [1.0, 0.0, 0.0, 0.50]);
            add_clear(&mut commands, [4, 4, 8, 8]);
            context.draw_screen(&commands).unwrap();
        });
    }

    #[test]
    fn semantics_screen_preserve() {
        run("semantics_screen_preserve", 8, 8, |context| {
            let shader = context.create_shader(Graph::trace(|| {
                let data = io::read::<[f32; 4]>();
                float4((data[0], data[1], data[2], data[3]))
            }));

            let mut commands = vec![];
            add_quad(&mut commands, shader, [1, 1, 7, 7], [1.0, 0.0, 0.0, 0.50]);
            context.draw_screen(&commands).unwrap();

            let mut commands = vec![];
            add_clear(&mut commands, [4, 4, 8, 8]);
            add_quad(&mut commands, shader, [1, 1, 7, 7], [0.0, 1.0, 1.0, 0.25]);
            context.draw_screen(&commands).unwrap();
        });
    }

    #[test]
    fn semantics_object_multiquad() {
        run("semantics_object_multiquad", 8, 8, |context| {
            let shader = context.create_shader(Graph::trace(|| {
                let data = io::read::<[f32; 4]>();
                float4((data[0], data[1], data[2], data[3]))
            }));

            let commands = vec![
                Command::ObjectBegin(shader),
                Command::ObjectRect([1, 1, 5, 5].into()),
                Command::ObjectRect([3, 3, 7, 7].into()),
                Command::ObjectData(ObjectData::Float(0.0)),
                Command::ObjectData(ObjectData::Float(0.5)),
                Command::ObjectData(ObjectData::Float(1.0)),
                Command::ObjectData(ObjectData::Float(0.5)),
                Command::ObjectEnd,
            ];

            context.draw_screen(&commands).unwrap();
        });
    }
}

pub mod tiling {
    use super::*;

    #[test]
    fn tiling_aligned_8() {
        run("tiling_aligned_8", 16, 16, |context| {
            let shader = context.create_shader(Graph::trace(|| {
                let data = (io::position() % 3.0) / 3.0;
                float4((data.x(), data.y(), data.x() + data.y(), 1.0))
            }));

            let mut commands = vec![];
            add_quad(&mut commands, shader, [0, 8, 8, 16], ());
            context.draw_screen(&commands).unwrap();
        });
    }

    #[test]
    fn tiling_centered_8() {
        run("tiling_centered_8", 16, 16, |context| {
            let shader = context.create_shader(Graph::trace(|| {
                let data = (io::position() % 3.0) / 3.0;
                float4((data.x(), data.y(), data.x() + data.y(), 1.0))
            }));

            let mut commands = vec![];
            add_quad(&mut commands, shader, [4, 4, 12, 12], ());
            context.draw_screen(&commands).unwrap();
        });
    }

    #[test]
    fn tiling_aligned_4() {
        run("tiling_aligned_4", 16, 16, |context| {
            let shader = context.create_shader(Graph::trace(|| {
                let data = (io::position() % 3.0) / 3.0;
                float4((data.x(), data.y(), data.x() + data.y(), 1.0))
            }));

            let mut commands = vec![];
            add_quad(&mut commands, shader, [0, 8, 4, 12], ());
            context.draw_screen(&commands).unwrap();
        });
    }

    #[test]
    fn tiling_centered_4() {
        run("tiling_centered_4", 16, 16, |context| {
            let shader = context.create_shader(Graph::trace(|| {
                let data = (io::position() % 3.0) / 3.0;
                float4((data.x(), data.y(), data.x() + data.y(), 1.0))
            }));

            let mut commands = vec![];
            add_quad(&mut commands, shader, [6, 6, 10, 10], ());
            context.draw_screen(&commands).unwrap();
        });
    }

    #[test]
    fn tiling_aligned_2() {
        run("tiling_aligned_2", 16, 16, |context| {
            let shader = context.create_shader(Graph::trace(|| {
                let data = (io::position() % 3.0) / 3.0;
                float4((data.x(), data.y(), data.x() + data.y(), 1.0))
            }));

            let mut commands = vec![];
            add_quad(&mut commands, shader, [4, 8, 6, 10], ());
            context.draw_screen(&commands).unwrap();
        });
    }

    #[test]
    fn tiling_centered_2() {
        run("tiling_centered_2", 16, 16, |context| {
            let shader = context.create_shader(Graph::trace(|| {
                let data = (io::position() % 3.0) / 3.0;
                float4((data.x(), data.y(), data.x() + data.y(), 1.0))
            }));

            let mut commands = vec![];
            add_quad(&mut commands, shader, [7, 7, 9, 9], ());
            context.draw_screen(&commands).unwrap();
        });
    }
}

#[cfg(not(miri))]
pub mod stress {
    use super::*;

    #[test]
    fn stress_texture_count() {
        run("stress_texture_count", 256, 8, move |context| {
            let textures = (0..=255u8)
                .map(|x| {
                    let texture = context.create_texture([1, 1].into(), TextureFormat::R8);

                    assert!(context.upload_texture(
                        texture,
                        TextureData {
                            bounds: [0, 0, 1, 1].into(),
                            format: TextureFormat::R8,
                            data: &[x],
                        },
                    ));

                    texture
                })
                .collect::<Vec<_>>();

            let shader = context.create_shader(Graph::trace(|| {
                let texture = io::read::<TextureId>();
                texture.sample(float2(0.0), TextureFilter::Linear)
            }));

            let mut commands = vec![];
            for i in 0..=255 {
                add_quad(&mut commands, shader, [i, 0, i + 1, 8], textures[i as usize]);
            }
            context.draw_screen(&commands).unwrap();
        });
    }

    #[test]
    fn stress_fill_rate() {
        run("stress_fill_rate", MAX_CANVAS_SIZE, MAX_CANVAS_SIZE, move |context| {
            let shader = context.create_shader(Graph::trace(|| {
                let i = io::read::<i32>();
                let j = int1(io::position().x()) + int1(io::position().y()) * int1(io::resolution().x());
                float4((1.0, 1.0, 1.0, (j % i).eq(0).select(float1(i).sqrt() / 255.0, 0.0)))
            }));

            for _ in 0..2 {
                let mut commands = vec![];
                add_clear(&mut commands, [0, 0, MAX_CANVAS_SIZE, MAX_CANVAS_SIZE]);

                for i in 2..500 {
                    add_quad(&mut commands, shader, [0, 0, MAX_CANVAS_SIZE, MAX_CANVAS_SIZE], i);
                }

                context.draw_screen(&commands).unwrap();
            }
        });
    }

    #[test]
    fn stress_quad_count() {
        run("stress_quad_count", MAX_CANVAS_SIZE, MAX_CANVAS_SIZE, move |context| {
            let shader = context.create_shader(Graph::trace(|| {
                let [r, g, b] = io::read::<[f32; 3]>();
                io::read::<[u32; 8]>();
                float4((r, g, b, 1.0))
            }));

            for _ in 0..2 {
                let mut commands = vec![];
                add_clear(&mut commands, [0, 0, MAX_CANVAS_SIZE, MAX_CANVAS_SIZE]);

                for i in 0..MAX_CANVAS_SIZE {
                    for j in 0..MAX_CANVAS_SIZE {
                        add_quad(
                            &mut commands,
                            shader,
                            [i, j, i + 1, j + 1],
                            if (i + j) % 2 == 0 {
                                ([1.0, 1.0, 0.0], [0u32; 8])
                            } else {
                                ([0.0, 0.0, 1.0], [0u32; 8])
                            },
                        );
                    }
                }

                context.draw_screen(&commands).unwrap();
            }
        });
    }

    #[test] //TODO: Fix
    fn stress_shader_complexity() {
        run("stress_shader_complexity", 4, 4, move |context| {
            let shader = context.create_shader(Graph::trace(|| {
                let mut a = float4((1.0, 0.5, 0.25, 1.0));
                for _ in 0..1000 {
                    a = a * float4((0.999, 1.0, 1.001, 1.0));
                }
                a
            }));

            let mut commands = vec![];
            add_quad(&mut commands, shader, [0, 0, 4, 4], ());
            context.draw_screen(&commands).unwrap();
        });
    }
}

#[cfg(not(miri))]
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
            let texture = context.create_texture([width, height].into(), TextureFormat::RGBA8);

            assert!(context.upload_texture(
                texture,
                TextureData {
                    bounds: [0, 0, width, height].into(),
                    format: TextureFormat::RGBA8,
                    data: &data,
                },
            ));

            let shader = context.create_shader(Graph::trace(|| {
                let atlas = io::read::<TextureId>();
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

            let mut commands = vec![];

            let mut x = 10.0;
            let mut scale = 0.5;
            for _ in 0..=10 {
                add_quad(
                    &mut commands,
                    shader,
                    [0, 0, MAX_CANVAS_SIZE, MAX_CANVAS_SIZE],
                    (texture, x, 12.0 + scale * 10.0, scale),
                );

                x += 18.0 * scale;
                scale *= 1.325;
            }

            add_quad(
                &mut commands,
                shader,
                [0, 0, MAX_CANVAS_SIZE, MAX_CANVAS_SIZE],
                (texture, 256.0, 320.0, 20.0),
            );

            context.draw_screen(&commands).unwrap();
        });
    }

    #[test]
    fn complex_boxblur() {
        run("complex_boxblur", MAX_CANVAS_SIZE, MAX_CANVAS_SIZE, |context| {
            fn sdf_circle(pos: float2, center: float2, radius: float1) -> float1 {
                ((center - pos).len() - radius).smoothstep(0.707, -0.707)
            }

            let shader_circle = context.create_shader(Graph::trace(|| {
                let [x, y] = io::read::<[f32; 2]>();

                let grid = (io::position() / 32.0).floor();
                let checker = (grid.x() + grid.y()) % 2.0;
                let texture = float4(checker).lerp(float4((1.0, 1.0, 1.0, 1.0)), float4((1.0, 0.0, 0.0, 1.0)));
                let mask = sdf_circle(io::position(), float2((x, y)), float1(128.0));
                let color = texture * mask;

                float4(mask * color)
            }));

            let shader_boxblur = context.create_shader(Graph::trace(|| {
                let buffer = io::read::<TextureId>();

                let mut result = float4(0.0);
                for i in -5..=5 {
                    for j in -5..=5 {
                        result = result + buffer.sample(io::position() + float2((i, j)), TextureFilter::Nearest);
                    }
                }

                result / (11 * 11) as f32
            }));

            let buffer = context.create_texture([MAX_CANVAS_SIZE, MAX_CANVAS_SIZE].into(), TextureFormat::RGBA8);

            let mut commands = vec![];
            add_quad(
                &mut commands,
                shader_circle,
                [0, 0, MAX_CANVAS_SIZE, MAX_CANVAS_SIZE],
                [256.0, 256.0],
            );
            context.draw_texture(buffer, &commands).unwrap();

            let mut commands = vec![];
            add_quad(
                &mut commands,
                shader_boxblur,
                [0, 0, MAX_CANVAS_SIZE, MAX_CANVAS_SIZE],
                buffer,
            );
            context.draw_screen(&commands).unwrap();
        });
    }
}

fn add_quad<T: ShaderData>(mut cmds: &mut Vec<Command>, shader: ShaderId, bounds: impl Into<Bounds>, data: T) {
    cmds.push(Command::ObjectBegin(shader));
    cmds.push(Command::ObjectRect(bounds.into()));
    data.write(&mut cmds);
    cmds.push(Command::ObjectEnd);
}

fn add_clear(cmds: &mut Vec<Command>, bounds: impl Into<Bounds>) {
    cmds.push(Command::Clear(bounds.into()));
}
