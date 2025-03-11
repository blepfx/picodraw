use std::{
    fmt::{Debug, LowerHex, UpperHex},
    hash::{Hash, Hasher},
};

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct OpAddr(u32);

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum OpValue {
    Position,
    Resolution,
    QuadStart,
    QuadEnd,

    Input(OpInput),
    Literal(OpLiteral),

    Add(OpAddr, OpAddr),
    Sub(OpAddr, OpAddr),
    Mul(OpAddr, OpAddr),
    Div(OpAddr, OpAddr),
    Rem(OpAddr, OpAddr),
    Dot(OpAddr, OpAddr),
    Cross(OpAddr, OpAddr),
    Neg(OpAddr),

    Sin(OpAddr),
    Cos(OpAddr),
    Tan(OpAddr),

    Asin(OpAddr),
    Acos(OpAddr),
    Atan(OpAddr),
    Atan2(OpAddr, OpAddr),

    Sqrt(OpAddr),
    Pow(OpAddr, OpAddr),
    Exp(OpAddr),
    Ln(OpAddr),

    Min(OpAddr, OpAddr),
    Max(OpAddr, OpAddr),
    Clamp(OpAddr, OpAddr, OpAddr),
    Abs(OpAddr),
    Sign(OpAddr),
    Floor(OpAddr),

    Lerp(OpAddr, OpAddr, OpAddr),
    Select(OpAddr, OpAddr, OpAddr),
    Smoothstep(OpAddr, OpAddr, OpAddr),
    Step(OpAddr, OpAddr),

    Eq(OpAddr, OpAddr),
    Ne(OpAddr, OpAddr),
    Lt(OpAddr, OpAddr),
    Le(OpAddr, OpAddr),
    Gt(OpAddr, OpAddr),
    Ge(OpAddr, OpAddr),

    And(OpAddr, OpAddr),
    Or(OpAddr, OpAddr),
    Xor(OpAddr, OpAddr),
    Not(OpAddr),

    Vec2(OpAddr, OpAddr),
    Vec3(OpAddr, OpAddr, OpAddr),
    Vec4(OpAddr, OpAddr, OpAddr, OpAddr),

    Splat2(OpAddr),
    Splat3(OpAddr),
    Splat4(OpAddr),

    CastFloat(OpAddr),
    CastInt(OpAddr),

    ExtractX(OpAddr),
    ExtractY(OpAddr),
    ExtractZ(OpAddr),
    ExtractW(OpAddr),

    Length(OpAddr),
    Normalize(OpAddr),

    DerivX(OpAddr),
    DerivY(OpAddr),
    DerivWidth(OpAddr),

    TextureLinear(OpAddr, OpAddr),
    TextureNearest(OpAddr, OpAddr),
    TextureSize(OpAddr),
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum OpInput {
    TextureStatic,
    TextureRender,

    F32,
    I32,
    I16,
    I8,
    U32,
    U16,
    U8,
}

#[derive(Clone, Copy, Debug)]
pub enum OpLiteral {
    Float(f32),
    Int(i32),
    Bool(bool),
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum OpType {
    F1,
    F2,
    F3,
    F4,
    I1,
    I2,
    I3,
    I4,
    Boolean,
    TextureStatic,
    TextureRender,
}

impl OpAddr {
    pub fn into_raw(self) -> usize {
        self.0 as _
    }

    pub fn from_raw(raw: usize) -> Self {
        Self(raw as _)
    }
}

impl OpType {
    pub fn is_numeric(self) -> bool {
        use OpType::*;
        matches!(self, F1 | F2 | F3 | F4 | I1 | I2 | I3 | I4)
    }

    pub fn is_float(self) -> bool {
        use OpType::*;
        matches!(self, F1 | F2 | F3 | F4)
    }

    pub fn is_int(self) -> bool {
        use OpType::*;
        matches!(self, I1 | I2 | I3 | I4)
    }

    pub fn is_texture(self) -> bool {
        use OpType::*;
        matches!(self, TextureStatic | TextureRender)
    }

    pub fn size(self) -> u32 {
        use OpType::*;
        match self {
            F1 | I1 | Boolean => 1,
            F2 | I2 => 2,
            F3 | I3 => 3,
            F4 | I4 => 4,
            TextureStatic | TextureRender => 1,
        }
    }
}

impl OpInput {
    pub fn value_type(self) -> OpType {
        use OpType::*;

        match self {
            Self::F32 => F1,
            Self::I32 => I1,
            Self::I16 => I1,
            Self::I8 => I1,
            Self::U32 => I1,
            Self::U16 => I1,
            Self::U8 => I1,
            Self::TextureStatic => TextureStatic,
            Self::TextureRender => TextureRender,
        }
    }
}

impl OpValue {
    pub fn type_check(&self, arg: impl Fn(OpAddr) -> Option<OpType>) -> Option<OpType> {
        use OpType::*;
        use OpValue::*;

        Some(match *self {
            Position => F2,
            QuadStart | QuadEnd => F2,
            Resolution => F2,
            Input(OpInput::F32) => F1,
            Input(OpInput::I32) => I1,
            Input(OpInput::I16) => I1,
            Input(OpInput::I8) => I1,
            Input(OpInput::U32) => I1,
            Input(OpInput::U16) => I1,
            Input(OpInput::U8) => I1,
            Input(OpInput::TextureStatic) => TextureStatic,
            Input(OpInput::TextureRender) => TextureRender,

            Literal(OpLiteral::Float(_)) => F1,
            Literal(OpLiteral::Int(_)) => I1,
            Literal(OpLiteral::Bool(_)) => Boolean,

            Add(a, b)
            | Sub(a, b)
            | Mul(a, b)
            | Div(a, b)
            | Rem(a, b)
            | Min(a, b)
            | Max(a, b)
            | Step(a, b) => {
                let l = arg(a)?;
                let r = arg(b)?;
                if l.is_numeric() && l == r {
                    l
                } else {
                    return None;
                }
            }

            Sin(x) | Cos(x) | Tan(x) | Asin(x) | Acos(x) | Atan(x) | Sqrt(x) | Exp(x) | Ln(x)
            | Floor(x) | DerivX(x) | DerivY(x) | DerivWidth(x) | Normalize(x) => {
                let x = arg(x)?;
                if x.is_float() {
                    x
                } else {
                    return None;
                }
            }

            Neg(x) | Abs(x) | Sign(x) => {
                let x = arg(x)?;
                if x.is_numeric() {
                    x
                } else {
                    return None;
                }
            }

            Atan2(x, y) | Pow(x, y) => {
                let l = arg(x)?;
                let r = arg(y)?;
                if l.is_float() && l == r {
                    l
                } else {
                    return None;
                }
            }

            Dot(x, y) => {
                let l = arg(x)?;
                let r = arg(y)?;
                if l.is_float() && l == r {
                    F1
                } else {
                    return None;
                }
            }

            Cross(x, y) => {
                let l = arg(x)?;
                let r = arg(y)?;
                if l == F3 && r == F3 {
                    F3
                } else {
                    return None;
                }
            }

            Clamp(x, y, z) | Lerp(x, y, z) | Smoothstep(x, y, z) => {
                let l = arg(x)?;
                let r = arg(y)?;
                let t = arg(z)?;
                if l.is_float() && l == r && l == t {
                    l
                } else {
                    return None;
                }
            }

            Select(x, y, z) => {
                let c = arg(x)?;
                let l = arg(y)?;
                let r = arg(z)?;
                if c == Boolean && l == r {
                    l
                } else {
                    return None;
                }
            }

            Eq(x, y) | Ne(x, y) | Lt(x, y) | Le(x, y) | Gt(x, y) | Ge(x, y) => {
                let l = arg(x)?;
                let r = arg(y)?;
                if l.is_numeric() && l == r {
                    Boolean
                } else {
                    return None;
                }
            }

            And(x, y) | Or(x, y) | Xor(x, y) => {
                let l = arg(x)?;
                let r = arg(y)?;
                if (l == Boolean || l.is_numeric()) && l == r {
                    l
                } else {
                    return None;
                }
            }

            Not(x) => match arg(x)? {
                x if x == Boolean || x.is_numeric() => x,
                _ => return None,
            },

            Vec2(x, y) => {
                let x = arg(x)?;
                let y = arg(y)?;
                if x.is_numeric() && x == y {
                    match x {
                        F1 => F2,
                        I1 => I2,
                        _ => return None,
                    }
                } else {
                    return None;
                }
            }

            Vec3(x, y, z) => {
                let x = arg(x)?;
                let y = arg(y)?;
                let z = arg(z)?;
                if x.is_numeric() && x == y && x == z {
                    match x {
                        F1 => F3,
                        I1 => I3,
                        _ => return None,
                    }
                } else {
                    return None;
                }
            }

            Vec4(x, y, z, w) => {
                let x = arg(x)?;
                let y = arg(y)?;
                let z = arg(z)?;
                let w = arg(w)?;
                if x.is_numeric() && x == y && x == z && x == w {
                    match x {
                        F1 => F4,
                        I1 => I4,
                        _ => return None,
                    }
                } else {
                    return None;
                }
            }

            Splat2(x) => match arg(x)? {
                F1 => F2,
                I1 => I2,
                _ => return None,
            },

            Splat3(x) => match arg(x)? {
                F1 => F3,
                I1 => I3,
                _ => return None,
            },

            Splat4(x) => match arg(x)? {
                F1 => F4,
                I1 => I4,
                _ => return None,
            },

            CastFloat(x) => match arg(x)? {
                I1 => F1,
                I2 => F2,
                I3 => F3,
                I4 => F4,
                _ => return None,
            },

            CastInt(x) => match arg(x)? {
                F1 => I1,
                F2 => I2,
                F3 => I3,
                F4 => I4,
                _ => return None,
            },

            ExtractX(x) => match arg(x)? {
                x if x.size() < 1 => return None,
                x if x.is_float() => F1,
                x if x.is_int() => I1,
                _ => return None,
            },

            ExtractY(x) => match arg(x)? {
                x if x.size() < 2 => return None,
                x if x.is_float() => F1,
                x if x.is_int() => I1,
                _ => return None,
            },

            ExtractZ(x) => match arg(x)? {
                x if x.size() < 3 => return None,
                x if x.is_float() => F1,
                x if x.is_int() => I1,
                _ => return None,
            },

            ExtractW(x) => match arg(x)? {
                x if x.size() < 4 => return None,
                x if x.is_float() => F1,
                x if x.is_int() => I1,
                _ => return None,
            },

            Length(x) => match arg(x)? {
                F1 | F2 | F3 | F4 => F1,
                _ => return None,
            },

            TextureLinear(x, y) | TextureNearest(x, y) => {
                let x = arg(x)?;
                let y = arg(y)?;
                if x.is_texture() && y == F2 {
                    F4
                } else {
                    return None;
                }
            }

            TextureSize(x) => match arg(x)? {
                TextureStatic | TextureRender => I2,
                _ => return None,
            },
        })
    }

    pub fn index_dependency(&self, idx: usize) -> Option<OpAddr> {
        use OpValue::*;

        match *self {
            Position | Resolution | QuadStart | QuadEnd | Input(_) | Literal(_) => None,

            Sin(x) | Cos(x) | Tan(x) | Asin(x) | Acos(x) | Atan(x) | Sqrt(x) | Exp(x) | Ln(x)
            | Not(x) | Neg(x) | Abs(x) | Sign(x) | Floor(x) | DerivX(x) | DerivY(x)
            | DerivWidth(x) | Normalize(x) | ExtractX(x) | ExtractY(x) | ExtractZ(x)
            | ExtractW(x) | Length(x) | Splat2(x) | Splat3(x) | Splat4(x) | CastFloat(x)
            | CastInt(x) | TextureSize(x) => {
                if idx == 0 {
                    Some(x)
                } else {
                    None
                }
            }

            Add(a, b)
            | Sub(a, b)
            | Mul(a, b)
            | Div(a, b)
            | Rem(a, b)
            | Min(a, b)
            | Max(a, b)
            | Step(a, b)
            | Dot(a, b)
            | Cross(a, b)
            | Atan2(a, b)
            | Pow(a, b)
            | And(a, b)
            | Or(a, b)
            | Xor(a, b)
            | Eq(a, b)
            | Ne(a, b)
            | Lt(a, b)
            | Le(a, b)
            | Gt(a, b)
            | Ge(a, b)
            | Vec2(a, b)
            | TextureLinear(a, b)
            | TextureNearest(a, b) => {
                if idx == 0 {
                    Some(a)
                } else if idx == 1 {
                    Some(b)
                } else {
                    None
                }
            }

            Vec3(x, y, z)
            | Lerp(x, y, z)
            | Clamp(x, y, z)
            | Select(x, y, z)
            | Smoothstep(x, y, z) => {
                if idx == 0 {
                    Some(x)
                } else if idx == 1 {
                    Some(y)
                } else if idx == 2 {
                    Some(z)
                } else {
                    None
                }
            }

            Vec4(x, y, z, w) => {
                if idx == 0 {
                    Some(x)
                } else if idx == 1 {
                    Some(y)
                } else if idx == 2 {
                    Some(z)
                } else if idx == 3 {
                    Some(w)
                } else {
                    None
                }
            }
        }
    }

    pub fn iter_dependencies(&self) -> impl Iterator<Item = OpAddr> + '_ {
        let mut i = 0;
        std::iter::from_fn(move || {
            let dep = self.index_dependency(i)?;
            i += 1;
            Some(dep)
        })
    }
}

impl Eq for OpLiteral {}

impl PartialEq for OpLiteral {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Float(a), Self::Float(b)) => {
                (a.is_nan() && b.is_nan()) || a.to_bits() == b.to_bits()
            }
            (Self::Int(a), Self::Int(b)) => a == b,
            (Self::Bool(a), Self::Bool(b)) => a == b,
            _ => false,
        }
    }
}

impl Hash for OpLiteral {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Float(x) if x.is_nan() => state.write_u32(0x7fc00000),
            Self::Float(x) => x.to_bits().hash(state),
            Self::Int(x) => x.hash(state),
            Self::Bool(x) => x.hash(state),
        }
    }
}

impl Debug for OpAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "${:0>4x}", self.0)
    }
}

impl Debug for OpType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use OpType::*;

        match self {
            F1 => write!(f, "F1"),
            F2 => write!(f, "F2"),
            F3 => write!(f, "F3"),
            F4 => write!(f, "F4"),
            I1 => write!(f, "I1"),
            I2 => write!(f, "I2"),
            I3 => write!(f, "I3"),
            I4 => write!(f, "I4"),
            Boolean => write!(f, "B1"),
            TextureStatic => write!(f, "TX"),
            TextureRender => write!(f, "TR"),
        }
    }
}

impl LowerHex for OpAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

impl UpperHex for OpAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:X}", self.0)
    }
}
