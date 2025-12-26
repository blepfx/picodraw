use crate::{TextureChannel, TextureFilter};

pub type OpFloat = u32;
pub type OpInt32 = u32;
pub type OpBool = u32;
pub type OpTex = u32;

/// A shader data. Shaders are represented by a directed acyclic graph of pure
/// (no side effects) instructions that outputs a color (as a `float4`).
#[derive(Clone, Copy, Debug)]
pub struct ShaderData<'a> {
    pub nodes: &'a [ShaderOp],
    pub output: [OpFloat; 4],
}

#[derive(Clone, Copy, Debug)]
pub enum ShaderOp {
    // literals
    FLit(f32),
    ILit(i32),
    BLit(bool),

    // object read
    ReadF32(u32),
    ReadI32(u32),
    ReadU16(u32),
    ReadU8(u32),

    // context info
    PosX,
    PosY,
    ResX,
    ResY,
    QuadB,
    QuadL,
    QuadT,
    QuadR,

    // float arithmetic
    FAdd(OpFloat, OpFloat),
    FSub(OpFloat, OpFloat),
    FMul(OpFloat, OpFloat),
    FDiv(OpFloat, OpFloat),
    FMod(OpFloat, OpFloat),
    FMin(OpFloat, OpFloat),
    FMax(OpFloat, OpFloat),
    FNeg(OpFloat),
    FAbs(OpFloat),

    // int32 arithmetic
    IAdd(OpInt32, OpInt32),
    ISub(OpInt32, OpInt32),
    IMul(OpInt32, OpInt32),
    IDiv(OpInt32, OpInt32),
    IMod(OpInt32, OpInt32),
    IMin(OpInt32, OpInt32),
    IMax(OpInt32, OpInt32),
    INeg(OpInt32),
    IAbs(OpInt32),

    // float math
    Sin(OpFloat),
    Cos(OpFloat),
    Tan(OpFloat),
    Asin(OpFloat),
    Acos(OpFloat),
    Atan(OpFloat),
    Atan2(OpFloat, OpFloat),

    Pow(OpFloat, OpFloat),
    Sqrt(OpFloat),
    Ln(OpFloat),
    Exp(OpFloat),

    Floor(OpFloat),
    Lerp(OpFloat, OpFloat, OpFloat),

    DerivX(OpFloat),
    DerivY(OpFloat),

    // integer bit stuff
    IOr(OpInt32, OpInt32),
    IAnd(OpInt32, OpInt32),
    IXor(OpInt32, OpInt32),
    IShl(OpInt32, OpInt32),
    IShr(OpInt32, OpInt32),
    INot(OpInt32),

    // boolean bit stuff
    BOr(OpBool, OpBool),
    BAnd(OpBool, OpBool),
    BXor(OpBool, OpBool),
    BNot(OpBool),

    // boolean comparison stuff
    IEq(OpInt32, OpInt32),
    INe(OpInt32, OpInt32),
    ILt(OpInt32, OpInt32),
    ILe(OpInt32, OpInt32),
    IGt(OpInt32, OpInt32),
    IGe(OpInt32, OpInt32),

    FEq(OpFloat, OpFloat),
    FNe(OpFloat, OpFloat),
    FLt(OpFloat, OpFloat),
    FLe(OpFloat, OpFloat),
    FGt(OpFloat, OpFloat),
    FGe(OpFloat, OpFloat),

    // extra stuff
    FSelect(OpBool, OpFloat, OpFloat),
    ISelect(OpBool, OpInt32, OpInt32),
    BSelect(OpBool, OpBool, OpBool),

    FCastInt32(OpInt32),
    ICastFloat(OpFloat),

    // texture stuff
    TexSampleF32(OpTex, OpFloat, OpFloat, TextureFilter, TextureChannel),
    TexSampleU8(OpTex, OpFloat, OpFloat, TextureFilter, TextureChannel),
    TexW(OpTex),
    TexH(OpTex),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ShaderOpType {
    Float,
    Int32,
    Bool,
}

impl ShaderOp {
    pub fn output_type(self) -> ShaderOpType {
        use ShaderOp::*;

        match self {
            PosX
            | PosY
            | ResX
            | ResY
            | QuadB
            | QuadL
            | QuadT
            | QuadR
            | ReadF32(_)
            | FLit(_)
            | FAdd(_, _)
            | FSub(_, _)
            | FMul(_, _)
            | FDiv(_, _)
            | FMod(_, _)
            | FMin(_, _)
            | FMax(_, _)
            | FNeg(_)
            | FAbs(_)
            | Sin(_)
            | Cos(_)
            | Tan(_)
            | Asin(_)
            | Acos(_)
            | Atan(_)
            | Atan2(_, _)
            | Pow(_, _)
            | Sqrt(_)
            | Ln(_)
            | Exp(_)
            | Floor(_)
            | Lerp(_, _, _)
            | DerivX(_)
            | DerivY(_)
            | FSelect(_, _, _)
            | FCastInt32(_)
            | TexSampleF32(_, _, _, _, _) => ShaderOpType::Float,

            ReadI32(_)
            | ReadU16(_)
            | ReadU8(_)
            | ILit(_)
            | IAdd(_, _)
            | ISub(_, _)
            | IMul(_, _)
            | IDiv(_, _)
            | IMod(_, _)
            | IMin(_, _)
            | IMax(_, _)
            | INeg(_)
            | IAbs(_)
            | IOr(_, _)
            | IAnd(_, _)
            | IXor(_, _)
            | IShl(_, _)
            | IShr(_, _)
            | INot(_)
            | ISelect(_, _, _)
            | ICastFloat(_)
            | TexSampleU8(_, _, _, _, _)
            | TexH(_)
            | TexW(_) => ShaderOpType::Int32,

            BLit(_)
            | BAnd(_, _)
            | BOr(_, _)
            | BXor(_, _)
            | BNot(_)
            | IEq(_, _)
            | INe(_, _)
            | ILt(_, _)
            | ILe(_, _)
            | IGt(_, _)
            | IGe(_, _)
            | FEq(_, _)
            | FNe(_, _)
            | FLt(_, _)
            | FLe(_, _)
            | FGt(_, _)
            | FGe(_, _)
            | BSelect(_, _, _) => ShaderOpType::Bool,
        }
    }
}
