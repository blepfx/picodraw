use super::IRProgram;
use crate::vm::{VMOp, compiler::ir::IR};
use bumpalo::{Bump, collections::Vec};
use picodraw_core::{ShaderData, ShaderError, ShaderOp};

pub fn lower_to_ir<'a>(arena: &'a Bump, shader: &ShaderData) -> Result<IRProgram<'a>, ShaderError> {
    let mut mapping = Vec::new_in(arena);
    for (_, op) in shader.iter() {
        mapping.push(lower_single(arena, |idx| mapping[idx as usize], op)?);
    }

    Ok(IRProgram {
        outputs: arena.alloc_slice_fill_iter(shader.output().iter().map(|&idx| mapping[idx as usize])),
    })
}

fn lower_single<'a>(arena: &'a Bump, mapping: impl Fn(u32) -> IR<'a>, op: ShaderOp) -> Result<IR<'a>, ShaderError> {
    Ok(IR::new(
        arena,
        match op {
            ShaderOp::FLit(value) => VMOp::LitF(value, ()),
            ShaderOp::ILit(value) => VMOp::LitI(value, ()),
            ShaderOp::BLit(true) => VMOp::LitI(-1, ()),
            ShaderOp::BLit(false) => VMOp::LitI(0, ()),
            ShaderOp::ReadF32(offset) => VMOp::Read(offset, ()),
            ShaderOp::ReadI32(offset) => VMOp::Read(offset, ()),
            ShaderOp::ReadU16(offset) => VMOp::ReadU16(offset, ()),
            ShaderOp::ReadU8(offset) => VMOp::ReadU8(offset, ()),
            ShaderOp::FAdd(a, b) => VMOp::AddF(mapping(a), mapping(b), ()),
            ShaderOp::FSub(a, b) => VMOp::SubF(mapping(a), mapping(b), ()),
            ShaderOp::FMul(a, b) => VMOp::MulF(mapping(a), mapping(b), ()),
            ShaderOp::FDiv(a, b) => VMOp::DivF(mapping(a), mapping(b), ()),
            ShaderOp::FMod(a, b) => VMOp::ModF(mapping(a), mapping(b), ()),
            ShaderOp::FNeg(a) => VMOp::NegF(mapping(a), ()),
            ShaderOp::IAdd(a, b) => VMOp::AddI(mapping(a), mapping(b), ()),
            ShaderOp::ISub(a, b) => VMOp::SubI(mapping(a), mapping(b), ()),
            ShaderOp::IMul(a, b) => VMOp::MulI(mapping(a), mapping(b), ()),
            ShaderOp::IDiv(a, b) => VMOp::DivI(mapping(a), mapping(b), ()),
            ShaderOp::IMod(a, b) => VMOp::ModI(mapping(a), mapping(b), ()),
            ShaderOp::INeg(a) => VMOp::NegI(mapping(a), ()),
            ShaderOp::Acos(a) => VMOp::AcosF(mapping(a), ()),
            ShaderOp::Asin(a) => VMOp::AsinF(mapping(a), ()),
            ShaderOp::Atan(a) => VMOp::AtanF(mapping(a), ()),
            ShaderOp::Atan2(a, b) => VMOp::Atan2F(mapping(a), mapping(b), ()),
            ShaderOp::Cos(a) => VMOp::CosF(mapping(a), ()),
            ShaderOp::Sin(a) => VMOp::SinF(mapping(a), ()),
            ShaderOp::Tan(a) => VMOp::TanF(mapping(a), ()),
            ShaderOp::Floor(a) => VMOp::FloorF(mapping(a), ()),
            ShaderOp::Sqrt(a) => VMOp::SqrtF(mapping(a), ()),
            ShaderOp::Exp(a) => VMOp::ExpF(mapping(a), ()),
            ShaderOp::Pow(a, b) => VMOp::PowF(mapping(a), mapping(b), ()),
            ShaderOp::Ln(a) => VMOp::LnF(mapping(a), ()),

            ShaderOp::FMin(a, b) => VMOp::MinF(mapping(a), mapping(b), ()),
            ShaderOp::FMax(a, b) => VMOp::MaxF(mapping(a), mapping(b), ()),
            ShaderOp::FAbs(a) => VMOp::AbsF(mapping(a), ()),

            ShaderOp::IMin(a, b) => VMOp::MinI(mapping(a), mapping(b), ()),
            ShaderOp::IMax(a, b) => VMOp::MaxI(mapping(a), mapping(b), ()),
            ShaderOp::IAbs(a) => VMOp::AbsI(mapping(a), ()),

            ShaderOp::IOr(a, b) | ShaderOp::BOr(a, b) => VMOp::OrI(mapping(a), mapping(b), ()),
            ShaderOp::IAnd(a, b) | ShaderOp::BAnd(a, b) => VMOp::AndI(mapping(a), mapping(b), ()),
            ShaderOp::IXor(a, b) | ShaderOp::BXor(a, b) => VMOp::XorI(mapping(a), mapping(b), ()),
            ShaderOp::INot(a) | ShaderOp::BNot(a) => VMOp::NotI(mapping(a), ()),
            ShaderOp::IShl(a, b) => VMOp::ShlI(mapping(a), mapping(b), ()),
            ShaderOp::IShr(a, b) => VMOp::ShrI(mapping(a), mapping(b), ()),

            ShaderOp::DerivX(a) => VMOp::DxF(mapping(a), ()),
            ShaderOp::DerivY(a) => VMOp::DyF(mapping(a), ()),

            ShaderOp::FSelect(a, b, c) | ShaderOp::BSelect(a, b, c) | ShaderOp::ISelect(a, b, c) => {
                VMOp::Select(mapping(a), mapping(b), mapping(c), ())
            }

            ShaderOp::FCastInt32(a) => VMOp::CastAsF(mapping(a), ()),
            ShaderOp::ICastFloat(a) => VMOp::CastAsI(mapping(a), ()),

            ShaderOp::PosX => VMOp::PosX(()),
            ShaderOp::PosY => VMOp::PosY(()),
            ShaderOp::ResX => VMOp::ResX(()),
            ShaderOp::ResY => VMOp::ResY(()),
            ShaderOp::QuadB => VMOp::QuadB(()),
            ShaderOp::QuadT => VMOp::QuadT(()),
            ShaderOp::QuadL => VMOp::QuadL(()),
            ShaderOp::QuadR => VMOp::QuadR(()),

            ShaderOp::TexH(a) | ShaderOp::TexW(a) | ShaderOp::TexSample(a, _, _, _, _) if a > 255 => {
                return Err(ShaderError::TooComplex);
            }

            ShaderOp::TexW(a) => VMOp::TexW(a as u8, ()),
            ShaderOp::TexH(a) => VMOp::TexH(a as u8, ()),
            ShaderOp::TexSample(texture_idx, u, v, texture_filter, texture_channel) => VMOp::Tex(
                texture_idx as u8,
                texture_channel as u8,
                texture_filter,
                mapping(u),
                mapping(v),
                (),
            ),

            // a + (b - a) * t
            ShaderOp::Lerp(t, a, b) => VMOp::AddF(
                mapping(a),
                IR::new(
                    arena,
                    VMOp::MulF(IR::new(arena, VMOp::SubF(mapping(b), mapping(a), ())), mapping(t), ()),
                ),
                (),
            ),

            ShaderOp::IEq(a, b) => VMOp::EqI(IR::new(arena, VMOp::SubI(mapping(a), mapping(b), ())), ()),
            ShaderOp::ILt(a, b) => VMOp::LtI(IR::new(arena, VMOp::SubI(mapping(a), mapping(b), ())), ()),
            ShaderOp::IGt(a, b) => VMOp::GtI(IR::new(arena, VMOp::SubI(mapping(a), mapping(b), ())), ()),

            ShaderOp::INe(a, b) => VMOp::NotI(lower_single(arena, mapping, ShaderOp::IEq(a, b))?, ()),
            ShaderOp::ILe(a, b) => VMOp::NotI(lower_single(arena, mapping, ShaderOp::IGt(a, b))?, ()),
            ShaderOp::IGe(a, b) => VMOp::NotI(lower_single(arena, mapping, ShaderOp::ILt(a, b))?, ()),

            ShaderOp::FEq(a, b) => VMOp::EqF(IR::new(arena, VMOp::SubF(mapping(a), mapping(b), ())), ()),
            ShaderOp::FLt(a, b) => VMOp::LtF(IR::new(arena, VMOp::SubF(mapping(a), mapping(b), ())), ()),
            ShaderOp::FGt(a, b) => VMOp::GtF(IR::new(arena, VMOp::SubF(mapping(a), mapping(b), ())), ()),

            ShaderOp::FNe(a, b) => VMOp::NotI(lower_single(arena, mapping, ShaderOp::FEq(a, b))?, ()),
            ShaderOp::FLe(a, b) => VMOp::NotI(lower_single(arena, mapping, ShaderOp::FGt(a, b))?, ()),
            ShaderOp::FGe(a, b) => VMOp::NotI(lower_single(arena, mapping, ShaderOp::FLt(a, b))?, ()),

            // TODO: optimize?
            ShaderOp::FSign(a) => {
                let input = mapping(a);
                VMOp::Select(
                    IR::new(arena, VMOp::LtF(input, ())),
                    IR::new(arena, VMOp::LitF(-1.0, ())),
                    IR::new(
                        arena,
                        VMOp::Select(
                            IR::new(arena, VMOp::GtF(input, ())),
                            IR::new(arena, VMOp::LitF(1.0, ())),
                            IR::new(arena, VMOp::LitF(0.0, ())),
                            (),
                        ),
                    ),
                    (),
                )
            }
            ShaderOp::ISign(a) => {
                let input = mapping(a);
                VMOp::Select(
                    IR::new(arena, VMOp::LtI(input, ())),
                    IR::new(arena, VMOp::LitI(-1, ())),
                    IR::new(
                        arena,
                        VMOp::Select(
                            IR::new(arena, VMOp::GtI(input, ())),
                            IR::new(arena, VMOp::LitI(1, ())),
                            IR::new(arena, VMOp::LitI(0, ())),
                            (),
                        ),
                    ),
                    (),
                )
            }
        },
    ))
}
