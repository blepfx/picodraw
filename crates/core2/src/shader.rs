use crate::{TextureChannel, TextureFilter};

pub type OpFloat = u32;
pub type OpInt32 = u32;
pub type OpBool = u32;
pub type OpTex = u32;
pub type OpLabel = u32;

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
    ReadF32(OpInt32),
    ReadI32(OpInt32),
    ReadI16(OpInt32),
    ReadI8(OpInt32),
    ReadU32(OpInt32),
    ReadU16(OpInt32),
    ReadU8(OpInt32),

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

    PowFloat(OpFloat, OpFloat),
    PowInt32(OpFloat, OpInt32),

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
    EqInt32(OpInt32, OpInt32),
    NeInt32(OpInt32, OpInt32),
    LtInt32(OpInt32, OpInt32),
    LeInt32(OpInt32, OpInt32),
    GtInt32(OpInt32, OpInt32),
    GeInt32(OpInt32, OpInt32),

    EqFloat(OpFloat, OpFloat),
    NeFloat(OpFloat, OpFloat),
    LtFloat(OpFloat, OpFloat),
    LeFloat(OpFloat, OpFloat),
    GtFloat(OpFloat, OpFloat),
    GeFloat(OpFloat, OpFloat),

    // extra stuff
    FSelect(OpBool, OpFloat, OpFloat),
    ISelect(OpBool, OpInt32, OpInt32),
    BSelect(OpBool, OpBool, OpBool),

    FCastInt32(OpInt32),
    ICastFloat(OpFloat),

    // texture stuff
    TexSampleFloat(OpTex, OpFloat, OpFloat, TextureFilter, TextureChannel),
    TexSampleInt(OpTex, OpFloat, OpFloat, TextureFilter, TextureChannel),
    TexW(OpTex),
    TexH(OpTex),

    // control flow
    Label,
    Branch(OpBool, OpLabel, OpLabel),
    Jump(OpLabel),

    FPhi(OpFloat, OpFloat),
    IPhi(OpInt32, OpInt32),
    BPhi(OpBool, OpBool),
}
