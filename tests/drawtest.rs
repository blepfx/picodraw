#![allow(unused_variables, unused_imports, dead_code)]

#[path = "./drawtest/runner.rs"]
mod runner;

use image::{GenericImageView, Rgba, open};
use picodraw::{trace::*, *};
use runner::{MAX_CANVAS_SIZE, run};
use std::f32::consts::PI;

pub mod ser {
    use super::*;

    #[test]
    pub fn ser_i32() {
        run("ser_i32", 4, 4, |context| {
            let shader = context
                .create_shader(&ShaderData::trace(|| {
                    let x = int1::read_i32(0);
                    float4((
                        float1(x & 255) / 255.0,
                        float1((x >> 8) & 255) / 255.0,
                        float1((x >> 16) & 255) / 255.0,
                        1.0,
                    ))
                }))
                .unwrap();

            context.draw(DrawTarget::Screen, |encoder| {
                encoder.add_rect([0, 0, 4, 4].into());
                encoder.add_data(&u32::to_ne_bytes(0xCAFEBEEF));
                encoder.draw(&shader);
            });
        });
    }

    #[test]
    pub fn ser_u16() {
        run("ser_u16", 4, 4, |context| {
            let shader = context
                .create_shader(&ShaderData::trace(|| {
                    let x = int1::read_u16(0);
                    float4((float1(x & 255) / 255.0, float1((x >> 8) & 255) / 255.0, 1.0, 1.0))
                }))
                .unwrap();

            context.draw(DrawTarget::Screen, |encoder| {
                encoder.add_rect([0, 0, 4, 4].into());
                encoder.add_data(&u16::to_ne_bytes(0xCAFE));
                encoder.draw(&shader);
            });
        });
    }

    #[test]
    pub fn ser_u8() {
        run("ser_u8", 4, 4, |context| {
            let shader = context
                .create_shader(&ShaderData::trace(|| {
                    let x = int1::read_u8(0);
                    float4((float1(x & 16) / 16.0, float1((x >> 4) & 16) / 16.0, 1.0, 1.0))
                }))
                .unwrap();

            context.draw(DrawTarget::Screen, |encoder| {
                encoder.add_rect([0, 0, 4, 4].into());
                encoder.add_data(&u8::to_ne_bytes(0xCA));
                encoder.draw(&shader);
            });
        });
    }

    #[test]
    pub fn ser_f32_pos() {
        run("ser_f32_pos", 4, 4, |context| {
            let shader = context
                .create_shader(&ShaderData::trace(|| {
                    let x = float1::read_f32(0);
                    float4((x, x * 2.0, x * 3.0, 1.0))
                }))
                .unwrap();

            context.draw(DrawTarget::Screen, |encoder| {
                encoder.add_rect([0, 0, 4, 4].into());
                encoder.add_data(&f32::to_ne_bytes(0.3333333f32));
                encoder.draw(&shader);
            });
        });
    }

    #[test]
    pub fn ser_f32_neg() {
        run("ser_f32_neg", 4, 4, |context| {
            let shader = context
                .create_shader(&ShaderData::trace(|| {
                    let x = float1::read_f32(0);
                    float4((x, -x * 2.0, x * 3.0, 1.0))
                }))
                .unwrap();

            context.draw(DrawTarget::Screen, |encoder| {
                encoder.add_rect([0, 0, 4, 4].into());
                encoder.add_data(&f32::to_ne_bytes(-0.3333333f32));
                encoder.draw(&shader);
            });
        });
    }

    #[test]
    pub fn ser_f32_zero() {
        run("ser_f32_zero", 4, 4, |context| {
            let shader = context
                .create_shader(&ShaderData::trace(|| {
                    let x = float1::read_f32(0);
                    float4((x, 1.0 - x, 0.5 - x, 1.0))
                }))
                .unwrap();

            context.draw(DrawTarget::Screen, |encoder| {
                encoder.add_rect([0, 0, 4, 4].into());
                encoder.add_data(&f32::to_ne_bytes(0.0));
                encoder.draw(&shader);
            });
        });
    }

    #[test]
    pub fn ser_f32_inf_pos() {
        run("ser_f32_inf_pos", 4, 4, |context| {
            let shader = context
                .create_shader(&ShaderData::trace(|| {
                    let x = float1::read_f32(0);
                    float4((x, -x, x, 1.0))
                }))
                .unwrap();

            context.draw(DrawTarget::Screen, |encoder| {
                encoder.add_rect([0, 0, 4, 4].into());
                encoder.add_data(&f32::to_ne_bytes(f32::INFINITY));
                encoder.draw(&shader);
            });
        });
    }

    #[test]
    pub fn ser_f32_inf_neg() {
        run("ser_f32_inf_neg", 4, 4, |context| {
            let shader = context
                .create_shader(&ShaderData::trace(|| {
                    let x = float1::read_f32(0);
                    float4((x, -x, x, 1.0))
                }))
                .unwrap();

            context.draw(DrawTarget::Screen, |encoder| {
                encoder.add_rect([0, 0, 4, 4].into());
                encoder.add_data(&f32::to_ne_bytes(f32::NEG_INFINITY));
                encoder.draw(&shader);
            });
        });
    }

    #[test]
    pub fn ser_f32_nan() {
        run("ser_f32_nan", 4, 4, |context| {
            let shader = context
                .create_shader(&ShaderData::trace(|| {
                    let x = float1::read_f32(0);
                    float4((
                        x.eq(x).select(float1(0.0), 1.0),
                        x.eq(x).select(float1(0.0), 1.0),
                        x.eq(x).select(float1(0.0), 1.0),
                        1.0,
                    ))
                }))
                .unwrap();

            context.draw(DrawTarget::Screen, |encoder| {
                encoder.add_rect([0, 0, 4, 4].into());
                encoder.add_data(&f32::to_ne_bytes(f32::NAN));
                encoder.draw(&shader);
            });
        });
    }

    #[test]
    pub fn ser_multiple() {
        run("ser_multiple", 4, 4, |context| {
            let shader = context
                .create_shader(&ShaderData::trace(|| {
                    // XXXX YYYY ZZW_
                    let x = float1::read_f32(0);
                    let y = int1::read_i32(4);
                    let z = int1::read_u16(8);
                    let w = int1::read_u8(10);

                    float4((x, y, z, w)) / 255.0
                }))
                .unwrap();

            context.draw(DrawTarget::Screen, |encoder| {
                encoder.add_rect([0, 0, 4, 4].into());
                encoder.add_data(&f32::to_ne_bytes(200.0));
                encoder.add_data(&i32::to_ne_bytes(100));
                encoder.add_data(&u16::to_ne_bytes(50));
                encoder.add_data(&u8::to_ne_bytes(250));
                encoder.draw(&shader);
            });
        });
    }
}

pub mod ops {
    use super::*;

    macro_rules! test {
        ($id:ident, $width:expr, $height:expr, $render:block) => {
            #[test]
            fn $id() {
                run(stringify!($id), $width, $height, |context| {
                    let shader = context
                        .create_shader(&ShaderData::trace(|| {
                            let z = { $render };
                            float4((z.x(), z.y(), z.z(), 1.0))
                        }))
                        .unwrap();

                    context.draw(DrawTarget::Screen, |encoder| {
                        encoder.add_rect([0, 0, $width, $height].into());
                        encoder.draw(&shader);
                    });
                });
            }
        };
    }

    test!(op_nothing, 4, 4, { float3((1.0, 0.0, 1.0)) });

    test!(op_infinity, 4, 4, {
        let pos_inf = float1(f32::INFINITY);
        let neg_inf = float1(f32::NEG_INFINITY);
        let nan = float1(f32::NAN);

        float3((pos_inf, neg_inf, nan.eq(nan).select(float1(0.0), 1.0)))
    });

    test!(op_select, 64, 64, {
        let p = int1(float2::position().x()).rem_euclid(4);

        select! {
            p.eq(0) => float3((1.0, 1.0, 0.0)),
            p.eq(1) => {
                let x = (float2::position().x() / float2::resolution().x()).clamp(0.0, 1.0);
                let y = (float2::position().y() / float2::resolution().y()).clamp(0.0, 1.0);
                float3((0.0, 1.0 - x, y))
            },
            p.eq(2) => {
                let x = (float2::position().x() / float2::resolution().x()).clamp(0.0, 1.0);
                let y = (float2::position().y() / float2::resolution().y()).clamp(0.0, 1.0);
                float3((1.0, x, 1.0 - y))
            },
            else => {
                float3((0.0, 0.0, 1.0))
            }
        }
    });

    test!(op_comp_ge, 64, 64, {
        let p = float2::position() / float2::resolution();

        float3((
            p.x().ge(p.y()).select(float1(1.0), 0.0),
            p.x().ge(0.5).select(float1(1.0), 0.0),
            p.y().ge(0.5).select(float1(1.0), 0.0),
        ))
    });

    test!(op_comp_le, 64, 64, {
        let p = float2::position() / float2::resolution();

        float3((
            p.x().le(p.y()).select(float1(1.0), 0.0),
            p.x().le(0.5).select(float1(1.0), 0.0),
            p.y().le(0.5).select(float1(1.0), 0.0),
        ))
    });

    test!(op_comp_gt, 64, 64, {
        let p = float2::position() / float2::resolution();

        float3((
            p.x().gt(p.y()).select(float1(1.0), 0.0),
            p.x().gt(0.5).select(float1(1.0), 0.0),
            p.y().gt(0.5).select(float1(1.0), 0.0),
        ))
    });

    test!(op_comp_lt, 64, 64, {
        let p = float2::position() / float2::resolution();

        float3((
            p.x().lt(p.y()).select(float1(1.0), 0.0),
            p.x().lt(0.5).select(float1(1.0), 0.0),
            p.y().lt(0.5).select(float1(1.0), 0.0),
        ))
    });

    test!(op_comp_eq, 64, 64, {
        let p = float2::position() / float2::resolution();

        float3((
            p.x().eq(p.y()).select(float1(1.0), 0.0),
            p.x().eq(0.5).select(float1(1.0), 0.0),
            p.y().eq(0.5).select(float1(1.0), 0.0),
        ))
    });

    test!(op_comp_ne, 64, 64, {
        let p = float2::position() / float2::resolution();

        float3((
            p.x().ne(p.y()).select(float1(1.0), 0.0),
            p.x().ne(0.5).select(float1(1.0), 0.0),
            p.y().ne(0.5).select(float1(1.0), 0.0),
        ))
    });

    test!(op_int_comp_ge, 64, 64, {
        let p = (float2::position() / float2::resolution()) * 16.0 - 8.0;

        float3((
            int1(p.x()).ge(int1(p.y())).select(float1(1.0), 0.0),
            int1(p.x()).ge(0).select(float1(1.0), 0.0),
            int1(p.y()).ge(0).select(float1(1.0), 0.0),
        ))
    });

    test!(op_int_comp_le, 64, 64, {
        let p = (float2::position() / float2::resolution()) * 16.0 - 8.0;

        float3((
            int1(p.x()).le(int1(p.y())).select(float1(1.0), 0.0),
            int1(p.x()).le(0).select(float1(1.0), 0.0),
            int1(p.y()).le(0).select(float1(1.0), 0.0),
        ))
    });

    test!(op_int_comp_gt, 64, 64, {
        let p = (float2::position() / float2::resolution()) * 16.0 - 8.0;

        float3((
            int1(p.x()).gt(int1(p.y())).select(float1(1.0), 0.0),
            int1(p.x()).gt(0).select(float1(1.0), 0.0),
            int1(p.y()).gt(0).select(float1(1.0), 0.0),
        ))
    });

    test!(op_int_comp_lt, 64, 64, {
        let p = (float2::position() / float2::resolution()) * 16.0 - 8.0;

        float3((
            int1(p.x()).lt(int1(p.y())).select(float1(1.0), 0.0),
            int1(p.x()).lt(0).select(float1(1.0), 0.0),
            int1(p.y()).lt(0).select(float1(1.0), 0.0),
        ))
    });

    test!(op_int_comp_eq, 64, 64, {
        let p = (float2::position() / float2::resolution()) * 16.0 - 8.0;

        float3((
            int1(p.x()).eq(int1(p.y())).select(float1(1.0), 0.0),
            int1(p.x()).eq(0).select(float1(1.0), 0.0),
            int1(p.y()).eq(0).select(float1(1.0), 0.0),
        ))
    });

    test!(op_int_comp_ne, 64, 64, {
        let p = (float2::position() / float2::resolution()) * 16.0 - 8.0;

        float3((
            int1(p.x()).ne(int1(p.y())).select(float1(1.0), 0.0),
            int1(p.x()).ne(0).select(float1(1.0), 0.0),
            int1(p.y()).ne(0).select(float1(1.0), 0.0),
        ))
    });

    test!(op_sin, 64, 8, {
        let x = (float2::position() / float2::resolution()).x() * 10.0 - 5.0;
        float3(x.sin())
    });

    test!(op_cos, 64, 8, {
        let x = (float2::position() / float2::resolution()).x() * 10.0 - 5.0;
        float3(x.cos())
    });

    test!(op_tan, 64, 8, {
        let x = (float2::position() / float2::resolution()).x() * 10.0 - 5.0;
        float3(x.tan())
    });

    test!(op_asin, 64, 8, {
        let x = (float2::position() / float2::resolution()).x() * 10.0 - 5.0;
        float3(x.asin())
    });

    test!(op_acos, 64, 8, {
        let x = (float2::position() / float2::resolution()).x() * 10.0 - 5.0;
        float3(x.acos())
    });

    test!(op_atan, 64, 8, {
        let x = (float2::position() / float2::resolution()).x() * 10.0 - 5.0;
        float3(x.atan())
    });

    test!(op_exp, 128, 8, {
        let x = (float2::position() / float2::resolution()).x() * 10.0 - 5.0;
        float3(x.exp())
    });

    test!(op_sqrt, 128, 8, {
        let x = (float2::position() / float2::resolution()).x() * 10.0 - 5.0;
        float3(x.sqrt())
    });

    test!(op_ln, 128, 8, {
        let x = (float2::position() / float2::resolution()).x() * 10.0 - 5.0;
        float3(x.ln())
    });

    test!(op_pow, 64, 64, {
        let p = (float2::position() / float2::resolution()) * 4.0;

        float3((p.x().powf(p.y()), p.x().powf(-p.y()), (-p.x()).powf(2.0)))
    });

    test!(op_cast, 128, 8, {
        let x = (float2::position() / float2::resolution()).x() * 10.0 - 5.0;
        let y = int1(x);
        float3((
            (float1(y) - x).abs(),
            float1(y.rem_euclid(2)) * 0.5 + 0.5,
            (float1(y) + 5.0) / 10.0,
        ))
    });

    test!(op_floor, 128, 8, {
        let x = (float2::position() / float2::resolution()).x() * 10.0 - 5.0;
        float3((x.floor() + 5.0) / 10.0)
    });

    test!(op_abs, 128, 8, {
        let x = (float2::position() / float2::resolution()) * 10.0 - 5.0;
        (float3((x.x().abs(), x.y().abs(), 0.0)) + 5.0) / 10.0
    });

    test!(op_sign, 8, 8, {
        let x = (float2::position() / float2::resolution()) * 10.0 - 5.0;
        float3((x.x().signum(), x.y().signum(), 0.0))
    });

    test!(op_min, 128, 8, {
        let x = (float2::position() / float2::resolution()).x() * 10.0 - 5.0;
        (float3((x.min(0.0), x.min(1.0), x.min(-1.0))) + 5.0) / 10.0
    });

    test!(op_max, 128, 8, {
        let x = (float2::position() / float2::resolution()).x() * 10.0 - 5.0;
        (float3((x.max(0.0), x.max(1.0), x.max(-1.0))) + 5.0) / 10.0
    });

    test!(op_clamp, 128, 8, {
        let x = (float2::position() / float2::resolution()).x() * 10.0 - 5.0;
        (float3((x.clamp(0.0, 1.0), x.clamp(-1.0, 1.0), x.clamp(-1.0, 0.0))) + 5.0) / 10.0
    });

    test!(op_lerp, 128, 8, {
        let x = (float2::position() / float2::resolution()).x();
        float3((
            x.lerp(float1(0.5), 1.0),
            x.lerp(float1(1.0), 0.0),
            float1(0.5).lerp(x, 0.5),
        ))
    });

    test!(op_int_sign, 8, 8, {
        let x = float2::position() - 4.5;
        float3((int1(x.x()).signum(), int1(x.y()).signum(), int1(0).signum())) * 0.5 + 0.5
    });

    test!(op_int_abs, 8, 8, {
        let x = float2::position() - 4.5;
        float3((float1(int1(x.x()).abs()), float1(int1(x.y()).abs()), int1(0).abs())) / 4.5
    });

    test!(op_int_and, 32, 32, {
        let x = int1(float2::position().x());
        let y = int1(float2::position().y());

        float3((
            float1((x & y).rem_euclid(16)) / 16.0,
            float1((x & y).rem_euclid(32)) / 32.0,
            float1((x & y).rem_euclid(64)) / 64.0,
        ))
    });

    test!(op_int_or, 32, 32, {
        let x = int1(float2::position().x());
        let y = int1(float2::position().y());

        float3((
            float1((x | y).rem_euclid(16)) / 16.0,
            float1((x | y).rem_euclid(32)) / 32.0,
            float1((x | y).rem_euclid(64)) / 64.0,
        ))
    });

    test!(op_int_xor, 32, 32, {
        let x = int1(float2::position().x());
        let y = int1(float2::position().y());

        float3((
            float1((x ^ y).rem_euclid(16)) / 16.0,
            float1((x ^ y).rem_euclid(32)) / 32.0,
            float1((x ^ y).rem_euclid(64)) / 64.0,
        ))
    });

    test!(op_int_not, 32, 32, {
        let x = int1(float2::position().x());
        let y = int1(float2::position().y());

        float3((
            float1((!x).rem_euclid(32)) / 32.0,
            float1((!y).rem_euclid(32)) / 32.0,
            float1((!(x + y)).rem_euclid(32)) / 32.0,
        ))
    });

    test!(op_int_shl, 32, 32, {
        let x = int1(float2::position().x());
        let y = int1(float2::position().y()) / 4;

        float3((
            float1((x << y).rem_euclid(16)) / 16.0,
            float1((x << y).rem_euclid(32)) / 32.0,
            float1((x << y).rem_euclid(64)) / 64.0,
        ))
    });

    test!(op_int_shr, 32, 32, {
        let x = int1(float2::position().x());
        let y = int1(float2::position().y()) / 4;

        float3((
            float1((x >> y).rem_euclid(16)) / 16.0,
            float1((x >> y).rem_euclid(32)) / 32.0,
            float1((x >> y).rem_euclid(64)) / 64.0,
        ))
    });

    test!(op_bool_or, 32, 32, {
        let x = int1(float2::position().x()).rem_euclid(2).eq(0);
        let y = int1(float2::position().y()).rem_euclid(2).eq(0);
        let z = int1(float2::position().x()).rem_euclid(3).eq(0);
        let w = int1(float2::position().y()).rem_euclid(3).eq(0);

        float3((
            (x | y).select(float1(1.0), 0.0),
            (z | w).select(float1(1.0), 0.0),
            (x | z).select(float1(1.0), 0.0),
        ))
    });

    test!(op_bool_and, 32, 32, {
        let x = int1(float2::position().x()).rem_euclid(2).eq(0);
        let y = int1(float2::position().y()).rem_euclid(2).eq(0);
        let z = int1(float2::position().x()).rem_euclid(3).eq(0);
        let w = int1(float2::position().y()).rem_euclid(3).eq(0);

        float3((
            (x & y).select(float1(1.0), 0.0),
            (z & w).select(float1(1.0), 0.0),
            (x & z).select(float1(1.0), 0.0),
        ))
    });

    test!(op_bool_xor, 32, 32, {
        let x = int1(float2::position().x()).rem_euclid(2).eq(0);
        let y = int1(float2::position().y()).rem_euclid(2).eq(0);
        let z = int1(float2::position().x()).rem_euclid(3).eq(0);
        let w = int1(float2::position().y()).rem_euclid(3).eq(0);

        float3((
            (x ^ y).select(float1(1.0), 0.0),
            (z ^ w).select(float1(1.0), 0.0),
            (x ^ z).select(float1(1.0), 0.0),
        ))
    });

    test!(op_bool_not, 32, 32, {
        let x = int1(float2::position().x()).rem_euclid(2).eq(0);
        let y = int1(float2::position().y()).rem_euclid(2).eq(0);

        float3((
            (!x).select(float1(1.0), 0.0),
            (!y & boolean(true)).select(float1(1.0), 0.0),
            (!x & boolean(false)).select(float1(1.0), 0.0),
        ))
    });

    test!(op_bool_select, 32, 32, {
        let x = int1(float2::position().x()).rem_euclid(2).eq(0);
        let y = int1(float2::position().y()).rem_euclid(2).eq(0);
        let z = int1(float2::position().x()).rem_euclid(3).eq(0);
        let w = int1(float2::position().y()).rem_euclid(3).eq(0);

        float3((
            x.select(z, w).select(float1(1.0), 0.0),
            y.select(x, z).select(float1(0.0), 1.0),
            z.select(w, y).select(float1(1.0), 0.0),
        ))
    });

    test!(op_dydx, 64, 64, {
        let p = float2::position() / float2::resolution();
        let z = (p.x() * 10.0).sin() * (p.y() * 10.0).cos();

        float3((z.dx() + 0.5, z.dy() + 0.5, p.dx().len()))
    });

    test!(op_atan2, 64, 64, {
        let p = (float2::position() / float2::resolution()) * 2.0 - 1.0;
        float3(p.x().atan2(p.y()) / PI * 0.5 + 0.5)
    });

    test!(op_norm2, 64, 64, {
        let p = (float2::position() / float2::resolution()) * 2.0 - 1.0;
        float3((p.norm().x() * 0.5 + 0.5, p.norm().y() * 0.5 + 0.5, p.len()))
    });

    test!(op_dot2, 64, 64, {
        let p = (float2::position() / float2::resolution()) * 2.0 - 1.0;
        let p = float2((p.x(), p.y()));
        float3((
            p.dot((1.0, 1.0)) * 0.5 + 0.5,
            p.dot((0.0, 1.0)) * 0.5 + 0.5,
            p.dot((1.0, 0.0)) * 0.5 + 0.5,
        ))
    });

    test!(op_cross3, 64, 64, {
        let p = (float2::position() / float2::resolution()) * 2.0 - 1.0;
        let p = float3((p.x(), p.y(), 0.0));
        p.cross((1.0, 1.0, 1.0)) * 0.5 + 0.5
    });

    test!(op_norm3, 64, 64, {
        let p = (float2::position() / float2::resolution()) * 2.0 - 1.0;
        let p = float3((p.x(), p.y(), 1.0));

        float3((p.norm().x() * 0.5 + 0.5, p.norm().y() * 0.5 + 0.5, p.len()))
    });

    test!(op_dot3, 64, 64, {
        let p = (float2::position() / float2::resolution()) * 2.0 - 1.0;
        let p = float3((p.x(), p.y(), 1.0));

        float3((
            p.dot((1.0, 1.0, 1.0)) * 0.5 + 0.5,
            p.dot((0.0, 1.0, 1.0)) * 0.5 + 0.5,
            p.dot((1.0, 0.0, 1.0)) * 0.5 + 0.5,
        ))
    });
}

pub mod tex {
    use super::*;

    const TEST_DITHER0: [u8; 16] = [
        0,
        8 * 16,
        2 * 16,
        10 * 16,
        12 * 16,
        4 * 16,
        14 * 16,
        6 * 16,
        3 * 16,
        11 * 16,
        16,
        9 * 16,
        15 * 16,
        7 * 16,
        13 * 16,
        5 * 16,
    ];

    #[test]
    fn texture_static_nearest() {
        run("texture_static_nearest", 32, 32, |context| {
            let mut texture = context.create_texture([4, 4].into(), TextureFormat::R8).unwrap();
            context
                .upload_texture(
                    &mut texture,
                    TextureData {
                        bounds: [0, 0, 4, 4].into(),
                        format: TextureFormat::R8,
                        data: &TEST_DITHER0,
                    },
                )
                .unwrap();

            let shader = context
                .create_shader(&ShaderData::trace(|| {
                    let texture = texture2d::read(0);
                    let uv = float2::position() / float2::resolution();
                    texture.sample(uv * float2((texture.width(), texture.height())), TextureFilter::Nearest)
                }))
                .unwrap();

            context.draw(DrawTarget::Screen, |encoder| {
                encoder.add_rect([0, 0, 32, 32].into());
                encoder.add_texture(&texture);
                encoder.draw(&shader);
            });

            context.delete_texture(texture);
        });
    }

    #[test]
    fn texture_static_linear() {
        run("texture_static_linear", 32, 32, |context| {
            let mut texture = context.create_texture([4, 4].into(), TextureFormat::R8).unwrap();
            context
                .upload_texture(
                    &mut texture,
                    TextureData {
                        bounds: [0, 0, 4, 4].into(),
                        format: TextureFormat::R8,
                        data: &TEST_DITHER0,
                    },
                )
                .unwrap();

            let shader = context
                .create_shader(&ShaderData::trace(|| {
                    let texture = texture2d::read(0);
                    let uv = float2::position() / float2::resolution();
                    texture.sample(uv * float2((texture.width(), texture.height())), TextureFilter::Linear)
                }))
                .unwrap();

            context.draw(DrawTarget::Screen, |encoder| {
                encoder.add_rect([0, 0, 32, 32].into());
                encoder.add_texture(&texture);
                encoder.draw(&shader);
            });

            context.delete_texture(texture);
        });
    }

    #[test]
    fn texture_render_nearest() {
        run("texture_render_nearest", 32, 32, |context| {
            let mut texture = context.create_texture([4, 4].into(), TextureFormat::RGBA8).unwrap();

            let shader_fill = context
                .create_shader(&ShaderData::trace(|| {
                    let a = float4((1.0, 0.5, 0.25, 1.0));
                    let b = float4((0.5, 0.25, 1.0, 1.0));
                    let p = float2::position() / float2::resolution();
                    p.dot((0.707, 0.707)).lerp(a, b)
                }))
                .unwrap();

            let shader_negative = context
                .create_shader(&ShaderData::trace(|| {
                    let texture = texture2d::read(0);
                    let z = texture.sample(
                        float2::position() / float2::resolution() * float2((texture.width(), texture.height())),
                        TextureFilter::Nearest,
                    );

                    float4((1.0 - z.x(), 1.0 - z.y(), 1.0 - z.z(), z.w()))
                }))
                .unwrap();

            context.draw(DrawTarget::Texture(&mut texture), |encoder| {
                encoder.add_rect([0, 0, 4, 4].into());
                encoder.draw(&shader_fill);
            });

            context.draw(DrawTarget::Screen, |encoder| {
                encoder.add_rect([0, 0, 20, 32].into());
                encoder.add_texture(&texture);
                encoder.draw(&shader_negative);
            });

            context.delete_texture(texture);
        });
    }

    #[test]
    fn texture_render_linear() {
        run("texture_render_linear", 32, 32, |context| {
            let mut texture = context.create_texture([4, 4].into(), TextureFormat::RGBA8).unwrap();

            let shader_fill = context
                .create_shader(&ShaderData::trace(|| {
                    let a = float4((1.0, 0.5, 0.25, 1.0));
                    let b = float4((0.5, 0.25, 1.0, 1.0));
                    let p = float2::position() / float2::resolution();
                    p.dot((0.707, 0.707)).lerp(a, b)
                }))
                .unwrap();

            let shader_negative = context
                .create_shader(&ShaderData::trace(|| {
                    let texture = texture2d::read(0);
                    let z = texture.sample(
                        float2::position() / float2::resolution() * float2((texture.width(), texture.height())),
                        TextureFilter::Linear,
                    );

                    float4((1.0 - z.x(), 1.0 - z.y(), 1.0 - z.z(), z.w()))
                }))
                .unwrap();

            context.draw(DrawTarget::Texture(&mut texture), |encoder| {
                encoder.add_rect([0, 0, 4, 4].into());
                encoder.draw(&shader_fill);
            });

            context.draw(DrawTarget::Screen, |encoder| {
                encoder.add_rect([0, 0, 20, 32].into());
                encoder.add_texture(&texture);
                encoder.draw(&shader_negative);
            });

            context.delete_texture(texture);
        });
    }

    #[test]
    fn texture_load_r8() {
        run("texture_load_r8", 4, 4, |context| {
            let mut texture = context.create_texture([1, 1].into(), TextureFormat::R8).unwrap();
            context
                .upload_texture(
                    &mut texture,
                    TextureData {
                        bounds: [0, 0, 1, 1].into(),
                        format: TextureFormat::R8,
                        data: &[100],
                    },
                )
                .unwrap();

            let shader = context
                .create_shader(&ShaderData::trace(|| {
                    texture2d::read(0).sample(0.0, TextureFilter::Nearest)
                }))
                .unwrap();

            context.draw(DrawTarget::Screen, |encoder| {
                encoder.add_rect([0, 0, 4, 4].into());
                encoder.add_texture(&texture);
                encoder.draw(&shader);
            });

            context.delete_texture(texture);
        });
    }

    #[test]
    fn texture_load_rgb8() {
        run("texture_load_rgb8", 4, 4, |context| {
            let mut texture = context.create_texture([1, 1].into(), TextureFormat::RGB8).unwrap();
            context
                .upload_texture(
                    &mut texture,
                    TextureData {
                        bounds: [0, 0, 1, 1].into(),
                        format: TextureFormat::RGB8,
                        data: &[100, 50, 200],
                    },
                )
                .unwrap();

            let shader = context
                .create_shader(&ShaderData::trace(|| {
                    texture2d::read(0).sample(0.0, TextureFilter::Nearest)
                }))
                .unwrap();

            context.draw(DrawTarget::Screen, |encoder| {
                encoder.add_rect([0, 0, 4, 4].into());
                encoder.add_texture(&texture);
                encoder.draw(&shader);
            });

            context.delete_texture(texture);
        });
    }

    #[test]
    fn texture_load_rgba8() {
        run("texture_load_rgba8", 4, 4, |context| {
            let mut texture = context.create_texture([1, 1].into(), TextureFormat::RGBA8).unwrap();
            context
                .upload_texture(
                    &mut texture,
                    TextureData {
                        bounds: [0, 0, 1, 1].into(),
                        format: TextureFormat::RGBA8,
                        data: &[100, 50, 200, 150],
                    },
                )
                .unwrap();

            let shader = context
                .create_shader(&ShaderData::trace(|| {
                    texture2d::read(0).sample(0.0, TextureFilter::Nearest)
                }))
                .unwrap();

            context.draw(DrawTarget::Screen, |encoder| {
                encoder.add_rect([0, 0, 4, 4].into());
                encoder.add_texture(&texture);
                encoder.draw(&shader);
            });

            context.delete_texture(texture);
        });
    }
}

pub mod semantics {
    use super::*;

    #[test]
    fn semantics_alpha() {
        run("semantics_alpha", 64, 8, |context| {
            let shader = context
                .create_shader(&ShaderData::trace(|| {
                    let x = float2::position().x() / float2::resolution().x();
                    float4((1.0, 1.0, 1.0, x))
                }))
                .unwrap();

            context.draw(DrawTarget::Screen, |encoder| {
                encoder.add_rect([0, 0, 64, 8].into());
                encoder.draw(&shader);

                encoder.add_rect([0, 4, 64, 8].into());
                encoder.draw(&shader);
            });
        });
    }

    #[test]
    fn semantics_blend() {
        run("semantics_blend", 8, 8, |context| {
            let shader = context
                .create_shader(&ShaderData::trace(|| {
                    let r = float1::read_f32(0);
                    let g = float1::read_f32(4);
                    let b = float1::read_f32(8);
                    let a = float1::read_f32(12);
                    float4((r, g, b, a))
                }))
                .unwrap();

            context.draw(DrawTarget::Screen, |encoder| {
                encoder.add_rect([1, 1, 5, 5].into());
                encoder.add_data(&f32::to_ne_bytes(0.5));
                encoder.add_data(&f32::to_ne_bytes(0.0));
                encoder.add_data(&f32::to_ne_bytes(0.0));
                encoder.add_data(&f32::to_ne_bytes(0.5));
                encoder.draw(&shader);

                encoder.add_rect([3, 3, 7, 7].into());
                encoder.add_data(&f32::to_ne_bytes(0.0));
                encoder.add_data(&f32::to_ne_bytes(0.25));
                encoder.add_data(&f32::to_ne_bytes(0.25));
                encoder.add_data(&f32::to_ne_bytes(0.25));
                encoder.draw(&shader);

                encoder.add_rect([0, 0, 8, 8].into());
                encoder.add_data(&f32::to_ne_bytes(0.1));
                encoder.add_data(&f32::to_ne_bytes(0.1));
                encoder.add_data(&f32::to_ne_bytes(0.1));
                encoder.add_data(&f32::to_ne_bytes(0.1));
                encoder.draw(&shader);
            });
        });
    }

    #[test]
    fn semantics_empty_draw() {
        run("semantics_empty_draw", 8, 8, |context| {
            let shader = context
                .create_shader(&ShaderData::trace(|| {
                    let t = texture2d::read(0);
                    let r = float1::read_f32(0);
                    let g = float1::read_f32(4);
                    let b = float1::read_f32(8);
                    let a = float1::read_f32(12);
                    float4((r, g, b, a)) + t.sample(0.0, TextureFilter::Nearest)
                }))
                .unwrap();

            context.draw(DrawTarget::Screen, |encoder| {
                encoder.draw(&shader);
            });
        });
    }

    #[test]
    fn semantics_missing_data() {
        run("semantics_missing_data", 8, 8, |context| {
            let shader = context
                .create_shader(&ShaderData::trace(|| {
                    let t = texture2d::read(0);
                    let r = float1::read_f32(0);
                    let g = float1::read_f32(4);
                    let b = float1::read_f32(8);
                    let a = float1::read_f32(12);
                    float4((r, g, b, a)) + t.sample(0.0, TextureFilter::Nearest)
                }))
                .unwrap();

            // reading missing data or sampling missing texture results in implementation-defined values
            // but it should not crash/result in undefined behavior (selecting between defined and impl-defined is still defined)
            context.draw(DrawTarget::Screen, |encoder| {
                encoder.add_rect([1, 1, 7, 7].into());
                encoder.draw(&shader);
                encoder.fill([1, 1, 7, 7].into(), Color::default());
            });
        });
    }

    #[test]
    fn semantics_clear() {
        run("semantics_clear", 8, 8, |context| {
            context.draw(DrawTarget::Screen, |encoder| {
                encoder.fill(
                    [1, 1, 7, 7].into(),
                    Color {
                        r: 128,
                        g: 64,
                        b: 32,
                        a: 255,
                    },
                );

                encoder.fill(
                    [4, 4, 8, 8].into(),
                    Color {
                        r: 32,
                        g: 64,
                        b: 128,
                        a: 128,
                    },
                );

                encoder.fill([0, 0, 4, 4].into(), Color::default());
            });
        });
    }

    #[test]
    fn semantics_screen_preserve() {
        run("semantics_screen_preserve", 8, 8, |context| {
            let shader = context
                .create_shader(&ShaderData::trace(|| {
                    let r = float1::read_f32(0);
                    let g = float1::read_f32(4);
                    let b = float1::read_f32(8);
                    let a = float1::read_f32(12);
                    float4((r, g, b, a))
                }))
                .unwrap();

            context.draw(DrawTarget::Screen, |encoder| {
                encoder.add_rect([1, 1, 7, 7].into());
                encoder.add_data(&f32::to_ne_bytes(0.5));
                encoder.add_data(&f32::to_ne_bytes(0.0));
                encoder.add_data(&f32::to_ne_bytes(0.0));
                encoder.add_data(&f32::to_ne_bytes(0.5));
                encoder.draw(&shader);
            });

            context.draw(DrawTarget::Screen, |encoder| {
                encoder.fill([4, 4, 8, 8].into(), Color::default());

                encoder.add_rect([1, 1, 7, 7].into());
                encoder.add_data(&f32::to_ne_bytes(0.0));
                encoder.add_data(&f32::to_ne_bytes(0.25));
                encoder.add_data(&f32::to_ne_bytes(0.25));
                encoder.add_data(&f32::to_ne_bytes(0.25));
                encoder.draw(&shader);
            });
        });
    }

    #[test]
    fn semantics_object_multiquad() {
        run("semantics_object_multiquad", 8, 8, |context| {
            let shader = context
                .create_shader(&ShaderData::trace(|| {
                    let r = float1::read_f32(0);
                    let g = float1::read_f32(4);
                    let b = float1::read_f32(8);
                    let a = float1::read_f32(12);
                    float4((r, g, b, a))
                }))
                .unwrap();

            context.draw(DrawTarget::Screen, |encoder| {
                encoder.add_rect([1, 1, 5, 5].into());
                encoder.add_rect([3, 3, 7, 7].into());
                encoder.add_data(&f32::to_ne_bytes(0.0));
                encoder.add_data(&f32::to_ne_bytes(0.5));
                encoder.add_data(&f32::to_ne_bytes(1.0));
                encoder.add_data(&f32::to_ne_bytes(0.5));
                encoder.draw(&shader);
            });
        });
    }

    #[test]
    fn semantics_quad_bounds() {
        run("semantics_quad_bounds", 16, 16, |context| {
            let shader = context
                .create_shader(&ShaderData::trace(|| {
                    let start = float2::quad_start();
                    let end = float2::quad_end();
                    let uv = (float2::position() - start) / (end - start);

                    float4((uv.x(), uv.y(), uv.x() + uv.y(), 1.0))
                }))
                .unwrap();

            context.draw(DrawTarget::Screen, |encoder| {
                encoder.add_rect([8, 0, 16, 8].into());
                encoder.draw(&shader);

                encoder.add_rect([8, 8, 16, 16].into());
                encoder.draw(&shader);
            });
        });
    }
}

pub mod tiling {
    use super::*;

    #[test]
    fn tiling_aligned_8() {
        run("tiling_aligned_8", 16, 16, |context| {
            let shader = context
                .create_shader(&ShaderData::trace(|| {
                    let data = float2::position().rem_euclid(3.0) / 3.0;
                    float4((data.x(), data.y(), data.x() + data.y(), 1.0))
                }))
                .unwrap();

            context.draw(DrawTarget::Screen, |encoder| {
                encoder.add_rect([0, 8, 8, 16].into());
                encoder.draw(&shader);
            });
        });
    }

    #[test]
    fn tiling_centered_8() {
        run("tiling_centered_8", 16, 16, |context| {
            let shader = context
                .create_shader(&ShaderData::trace(|| {
                    let data = float2::position().rem_euclid(3.0) / 3.0;
                    float4((data.x(), data.y(), data.x() + data.y(), 1.0))
                }))
                .unwrap();

            context.draw(DrawTarget::Screen, |encoder| {
                encoder.add_rect([4, 4, 12, 12].into());
                encoder.draw(&shader);
            });
        });
    }

    #[test]
    fn tiling_aligned_4() {
        run("tiling_aligned_4", 16, 16, |context| {
            let shader = context
                .create_shader(&ShaderData::trace(|| {
                    let data = float2::position().rem_euclid(3.0) / 3.0;
                    float4((data.x(), data.y(), data.x() + data.y(), 1.0))
                }))
                .unwrap();

            context.draw(DrawTarget::Screen, |encoder| {
                encoder.add_rect([0, 8, 4, 12].into());
                encoder.draw(&shader);
            });
        });
    }

    #[test]
    fn tiling_centered_4() {
        run("tiling_centered_4", 16, 16, |context| {
            let shader = context
                .create_shader(&ShaderData::trace(|| {
                    let data = float2::position().rem_euclid(3.0) / 3.0;
                    float4((data.x(), data.y(), data.x() + data.y(), 1.0))
                }))
                .unwrap();

            context.draw(DrawTarget::Screen, |encoder| {
                encoder.add_rect([6, 6, 10, 10].into());
                encoder.draw(&shader);
            });
        });
    }

    #[test]
    fn tiling_aligned_2() {
        run("tiling_aligned_2", 16, 16, |context| {
            let shader = context
                .create_shader(&ShaderData::trace(|| {
                    let data = float2::position().rem_euclid(3.0) / 3.0;
                    float4((data.x(), data.y(), data.x() + data.y(), 1.0))
                }))
                .unwrap();

            context.draw(DrawTarget::Screen, |encoder| {
                encoder.add_rect([4, 8, 6, 10].into());
                encoder.draw(&shader);
            });
        });
    }

    #[test]
    fn tiling_centered_2() {
        run("tiling_centered_2", 16, 16, |context| {
            let shader = context
                .create_shader(&ShaderData::trace(|| {
                    let data = float2::position().rem_euclid(3.0) / 3.0;
                    float4((data.x(), data.y(), data.x() + data.y(), 1.0))
                }))
                .unwrap();

            context.draw(DrawTarget::Screen, |encoder| {
                encoder.add_rect([7, 7, 9, 9].into());
                encoder.draw(&shader);
            });
        });
    }
}

pub mod stress {
    use super::*;

    #[test]
    fn stress_texture_count() {
        run("stress_texture_count", 256, 8, move |context| {
            let textures = (0..=255u8)
                .map(|x| {
                    let mut tex = context.create_texture([1, 1].into(), TextureFormat::R8).unwrap();

                    context
                        .upload_texture(
                            &mut tex,
                            TextureData {
                                bounds: [0, 0, 1, 1].into(),
                                format: TextureFormat::R8,
                                data: &[x],
                            },
                        )
                        .unwrap();

                    tex
                })
                .collect::<Vec<_>>();

            let shader = context
                .create_shader(&ShaderData::trace(|| {
                    let tex = texture2d::read(0);
                    tex.sample(float2(0.0), TextureFilter::Linear)
                }))
                .unwrap();

            context.draw(DrawTarget::Screen, |encoder| {
                for i in 0..=255i32 {
                    encoder.add_rect([i, 0, i + 1, 8].into());
                    encoder.add_texture(&textures[i as usize]);
                    encoder.draw(&shader);
                }
            });
        });
    }

    #[test]
    #[cfg(not(miri))]
    #[cfg(false)]
    fn stress_fill_rate() {
        run("stress_fill_rate", MAX_CANVAS_SIZE, MAX_CANVAS_SIZE, move |context| {
            let shader = context.create_shader(Graph::trace(|| {
                let i = io::read::<i32>();
                let j = int1(float2::position().x()) + int1(float2::position().y()) * int1(float2::resolution().x());
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
        if cfg!(miri) {
            return;
        }

        run("stress_quad_count", MAX_CANVAS_SIZE, MAX_CANVAS_SIZE, move |context| {
            let shader = context
                .create_shader(&ShaderData::trace(|| {
                    float4((
                        float1(int1::read_u8(0)) / 255.0,
                        float1(int1::read_u8(1)) / 255.0,
                        float1(int1::read_u8(2)) / 255.0,
                        1.0,
                    ))
                }))
                .unwrap();

            for _ in 0..2 {
                context.draw(DrawTarget::Screen, |encoder| {
                    encoder.fill([0, 0, MAX_CANVAS_SIZE, MAX_CANVAS_SIZE].into(), Color::default());

                    for i in 0..MAX_CANVAS_SIZE {
                        for j in 0..MAX_CANVAS_SIZE {
                            encoder.add_rect([i, j, i + 1, j + 1].into());
                            if (i + j) % 2 == 0 {
                                encoder.add_data(&[255u8, 255u8, 0u8]);
                            } else {
                                encoder.add_data(&[0u8, 0u8, 255u8]);
                            }
                            encoder.draw(&shader);
                        }
                    }
                });
            }
        });
    }

    #[test]
    #[cfg(not(miri))] //TODO: Fix
    #[cfg(false)]
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

pub mod complex {
    use super::*;

    #[test]
    fn complex_sdf_round_rect() {
        if cfg!(miri) {
            return;
        }

        run("complex_sdf_round_rect", MAX_CANVAS_SIZE, MAX_CANVAS_SIZE, |context| {
            // https://iquilezles.org/articles/distfunctions2d/
            fn shader_rect() -> float4 {
                let center = float2((float1::read_f32(0), float1::read_f32(4)));
                let angle = float1::read_f32(8);
                let extents = float2((float1::read_f32(12), float1::read_f32(16)));
                let radius = float4((
                    float1::read_f32(20),
                    float1::read_f32(24),
                    float1::read_f32(28),
                    float1::read_f32(32),
                ));
                let color = float4((
                    float1::read_f32(36),
                    float1::read_f32(40),
                    float1::read_f32(44),
                    float1::read_f32(48),
                ));

                let p = float2::position() - center;
                let p = float2((
                    p.x() * angle.cos() - p.y() * angle.sin(),
                    p.x() * angle.sin() + p.y() * angle.cos(),
                ));

                let r = p.x().gt(0.0).select(
                    p.y().gt(0.0).select(radius.x(), radius.y()),
                    p.y().gt(0.0).select(radius.z(), radius.w()),
                );

                let q = p.abs() - extents + r;
                let d = q.x().max(q.y()).min(0.0) + q.max(0.0).len() - r;

                let mask = (0.5 - d * 0.707).clamp(0.0, 1.0);
                float4((color.x(), color.y(), color.z(), color.w() * mask))
            }

            let shader_rect = context.create_shader(&ShaderData::trace(shader_rect)).unwrap();

            context.draw(DrawTarget::Screen, |encoder| {
                encoder.add_rect([0, 0, MAX_CANVAS_SIZE, MAX_CANVAS_SIZE].into());
                encoder.add_data(&f32::to_ne_bytes(256.0)); // center.x
                encoder.add_data(&f32::to_ne_bytes(256.0)); // center.y
                encoder.add_data(&f32::to_ne_bytes(0.5)); // angle
                encoder.add_data(&f32::to_ne_bytes(100.0)); // extents.x
                encoder.add_data(&f32::to_ne_bytes(50.0)); // extents.y
                encoder.add_data(&f32::to_ne_bytes(10.0)); // radius.top_left
                encoder.add_data(&f32::to_ne_bytes(20.0)); // radius.top_right
                encoder.add_data(&f32::to_ne_bytes(30.0)); // radius.bottom_right
                encoder.add_data(&f32::to_ne_bytes(10.0)); // radius.bottom_left
                encoder.add_data(&f32::to_ne_bytes(1.0)); // color.r
                encoder.add_data(&f32::to_ne_bytes(0.0)); // color.g
                encoder.add_data(&f32::to_ne_bytes(1.0)); // color.b
                encoder.add_data(&f32::to_ne_bytes(0.5)); // color.a
                encoder.draw(&shader_rect);

                encoder.add_rect([0, 0, MAX_CANVAS_SIZE, MAX_CANVAS_SIZE].into());
                encoder.add_data(&f32::to_ne_bytes(300.0)); // center.x
                encoder.add_data(&f32::to_ne_bytes(200.0)); // center.y
                encoder.add_data(&f32::to_ne_bytes(-0.1)); // angle
                encoder.add_data(&f32::to_ne_bytes(60.0)); // extents.x
                encoder.add_data(&f32::to_ne_bytes(90.0)); // extents.y
                encoder.add_data(&f32::to_ne_bytes(10.0)); // radius.top_left
                encoder.add_data(&f32::to_ne_bytes(10.0)); // radius.top_right
                encoder.add_data(&f32::to_ne_bytes(20.0)); // radius.bottom_right
                encoder.add_data(&f32::to_ne_bytes(20.0)); // radius.bottom_left
                encoder.add_data(&f32::to_ne_bytes(0.0)); // color.r
                encoder.add_data(&f32::to_ne_bytes(1.0)); // color.g
                encoder.add_data(&f32::to_ne_bytes(1.0)); // color.b
                encoder.add_data(&f32::to_ne_bytes(0.5)); // color.a
                encoder.draw(&shader_rect);
            });
        });
    }

    #[test]
    fn complex_msdf() {
        if cfg!(miri) {
            return;
        }

        let (width, height, data) = {
            let msdf = open("./tests/drawtest/msdf.webp").unwrap();
            let mut data = vec![0u8; (4 * msdf.width() * msdf.height()) as usize];
            for i in 0..msdf.width() {
                for j in 0..msdf.height() {
                    let Rgba([r, g, b, _]) = msdf.get_pixel(i, j);
                    data[((i + j * msdf.width()) * 4) as usize] = r;
                    data[((i + j * msdf.width()) * 4 + 1) as usize] = g;
                    data[((i + j * msdf.width()) * 4 + 2) as usize] = b;
                    data[((i + j * msdf.width()) * 4 + 3) as usize] = 255;
                }
            }

            (msdf.width(), msdf.height(), data)
        };

        run("complex_msdf", MAX_CANVAS_SIZE, MAX_CANVAS_SIZE, move |context| {
            let mut texture = context
                .create_texture([width, height].into(), TextureFormat::RGBA8)
                .unwrap();

            context
                .upload_texture(
                    &mut texture,
                    TextureData {
                        bounds: [0, 0, width, height].into(),
                        format: TextureFormat::RGBA8,
                        data: &data,
                    },
                )
                .unwrap();

            let shader = context
                .create_shader(&ShaderData::trace(|| {
                    let (x, y) = (float1::read_f32(0), float1::read_f32(4));
                    let scale = float1::read_f32(8);
                    let atlas = texture2d::read(0);

                    let pos = (float2::position() - float2((x, y))) / scale + float2(12.0);
                    let sample = atlas.sample(
                        pos.clamp(0.0, float2((atlas.width(), atlas.height()))),
                        TextureFilter::Linear,
                    );
                    let median = float1::max(
                        float1::min(sample.x(), sample.y()),
                        float1::min(float1::max(sample.x(), sample.y()), sample.z()),
                    );

                    let mask = (((2.0 * scale).max(1.0) * (median - 0.5)) + 0.5).clamp(0.0, 1.0);
                    float4(mask)
                }))
                .unwrap();

            context.draw(DrawTarget::Screen, |encoder| {
                encoder.fill([0, 0, MAX_CANVAS_SIZE, MAX_CANVAS_SIZE].into(), Color::default());

                let mut x = 10.0;
                let mut scale = 0.5;
                for _ in 0..=10 {
                    encoder.add_rect([0, 0, MAX_CANVAS_SIZE, MAX_CANVAS_SIZE].into());
                    encoder.add_data(&f32::to_ne_bytes(x)); // x
                    encoder.add_data(&f32::to_ne_bytes(12.0 + scale * 10.0)); // y
                    encoder.add_data(&f32::to_ne_bytes(scale)); // scale
                    encoder.add_texture(&texture);
                    encoder.draw(&shader);

                    x += 18.0 * scale;
                    scale *= 1.325;
                }

                encoder.add_rect([0, 0, MAX_CANVAS_SIZE, MAX_CANVAS_SIZE].into());
                encoder.add_data(&f32::to_ne_bytes(256.0)); // x
                encoder.add_data(&f32::to_ne_bytes(320.0)); // y
                encoder.add_data(&f32::to_ne_bytes(20.0)); // scale
                encoder.add_texture(&texture);
                encoder.draw(&shader);
            });
        });
    }

    #[test]
    fn complex_boxblur() {
        if cfg!(miri) {
            return;
        }

        run("complex_boxblur", MAX_CANVAS_SIZE, MAX_CANVAS_SIZE, |context| {
            fn sdf_circle(pos: float2, center: float2, radius: float1) -> float1 {
                (0.5 - ((center - pos).len() - radius) / 0.707).clamp(0.0, 1.0)
            }

            let shader_circle = context
                .create_shader(&ShaderData::trace(|| {
                    let x = float1::read_f32(0);
                    let y = float1::read_f32(4);

                    let grid = (float2::position() / 32.0).floor();
                    let checker = (grid.x() + grid.y()).rem_euclid(2.0);
                    let texture = checker.lerp(float4((1.0, 1.0, 1.0, 1.0)), float4((1.0, 0.0, 0.0, 1.0)));
                    let mask = sdf_circle(float2::position(), float2((x, y)), float1(128.0));
                    let color = texture * mask;

                    float4(mask * color)
                }))
                .unwrap();

            let shader_boxblur = context
                .create_shader(&ShaderData::trace(|| {
                    let buffer = texture2d::read(0);

                    let mut result = float4(0.0);
                    for i in -5..=5 {
                        for j in -5..=5 {
                            result =
                                result + buffer.sample(float2::position() + float2((i, j)), TextureFilter::Nearest);
                        }
                    }

                    result / (11 * 11) as f32
                }))
                .unwrap();

            let mut buffer = context
                .create_texture([MAX_CANVAS_SIZE, MAX_CANVAS_SIZE].into(), TextureFormat::RGBA8)
                .unwrap();

            context.draw(DrawTarget::Texture(&mut buffer), |encoder| {
                encoder.fill([0, 0, MAX_CANVAS_SIZE, MAX_CANVAS_SIZE].into(), Color::default());
                encoder.add_rect([0, 0, MAX_CANVAS_SIZE, MAX_CANVAS_SIZE].into());
                encoder.add_data(&f32::to_ne_bytes(256.0)); // circle center x
                encoder.add_data(&f32::to_ne_bytes(256.0)); // circle center y
                encoder.draw(&shader_circle);
            });

            context.draw(DrawTarget::Screen, |encoder| {
                encoder.fill([0, 0, MAX_CANVAS_SIZE, MAX_CANVAS_SIZE].into(), Color::default());
                encoder.add_rect([0, 0, MAX_CANVAS_SIZE, MAX_CANVAS_SIZE].into());
                encoder.add_texture(&buffer);
                encoder.draw(&shader_boxblur);
            });
        });
    }
}
