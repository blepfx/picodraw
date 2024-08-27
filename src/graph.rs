use crate::types::GlType;
use std::cell::RefCell;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::mem::replace;

thread_local! {
    static CURRENT_GRAPH: RefCell<Option<Vec<(Op, ValueType)>>> = RefCell::new(None);
}

pub(crate) fn push_op(value: Op, r#type: ValueType) -> OpAddr {
    CURRENT_GRAPH.with(|graph| {
        let mut graph = graph.borrow_mut();
        let graph = graph
            .as_mut()
            .expect("executing not in a shader graph context");

        graph.push((value, r#type));
        OpAddr((graph.len() - 1) as u32, PhantomData)
    })
}

#[derive(Clone, Debug)]
pub struct ShaderGraph<T> {
    values: Vec<(Op, ValueType)>,
    result: T,
}

impl<T: GlType> ShaderGraph<T> {
    pub fn collect(c: impl FnOnce() -> T) -> Self {
        let prev = CURRENT_GRAPH.with(|engine| replace(&mut *engine.borrow_mut(), Some(vec![])));
        let result = c();
        let values = CURRENT_GRAPH.with(|engine| replace(&mut *engine.borrow_mut(), prev));

        Self {
            values: values.unwrap(),
            result,
        }
    }

    pub fn get(&self, value: OpAddr) -> (&Op, ValueType) {
        let (src, ty) = self.values.get(value.0 as usize).expect("invalid value");
        (src, *ty)
    }

    pub fn iter<'a>(
        &'a self,
    ) -> impl Iterator<Item = (OpAddr, Op, ValueType)> + DoubleEndedIterator + ExactSizeIterator + 'a
    {
        self.values
            .iter()
            .enumerate()
            .map(|(i, (source, ty))| (OpAddr(i as u32, PhantomData), *source, *ty))
    }

    pub fn result(&self) -> OpAddr {
        self.result.unwrap()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct OpAddr(u32, PhantomData<*const ()>);

impl OpAddr {
    pub fn id(&self) -> u32 {
        self.0
    }
}

impl Debug for OpAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "${:x}", self.0)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Op {
    Input(usize),

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
    Fract(OpAddr),

    Lerp(OpAddr, OpAddr, OpAddr),
    Smoothstep(OpAddr, OpAddr, OpAddr),
    Step(OpAddr, OpAddr),
    Select(OpAddr, OpAddr, OpAddr),

    LitFloat(f32),
    LitInt(i32),
    LitBool(bool),

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

    NewVec2(OpAddr, OpAddr),
    NewVec3(OpAddr, OpAddr, OpAddr),
    NewVec4(OpAddr, OpAddr, OpAddr, OpAddr),
    SplatVec2(OpAddr),
    SplatVec3(OpAddr),
    SplatVec4(OpAddr),

    CastFloat(OpAddr),
    CastInt(OpAddr),

    // CastBool(Value),
    // CastVec2(Value),
    // CastVec3(Value),
    // CastVec4(Value),
    Swizzle1(OpAddr, Swizzle),
    // Swizzle2(Value, [Swizzle; 2]),
    // Swizzle3(Value, [Swizzle; 3]),
    // Swizzle4(Value, [Swizzle; 4]),
    Length(OpAddr),
    Normalize(OpAddr),

    DerivX(OpAddr),
    DerivY(OpAddr),
    DerivWidth(OpAddr),

    TextureSampleLinear(OpAddr, OpAddr),
    TextureSampleNearest(OpAddr, OpAddr),
    TextureSize(OpAddr),

    SlotCreate(OpAddr),
    SlotUpdate(OpAddr, OpAddr),

    LoopPush(OpAddr),
    LoopPop,
}

impl Op {
    pub fn visit_dependencies(&self, mut v: impl FnMut(OpAddr)) {
        match self {
            Op::Input(_) => {}

            Op::LitFloat(_) => {}
            Op::LitInt(_) => {}
            Op::LitBool(_) => {}
            Op::Add(a, b) => {
                v(*a);
                v(*b);
            }
            Op::Sub(a, b) => {
                v(*a);
                v(*b);
            }
            Op::Mul(a, b) => {
                v(*a);
                v(*b);
            }
            Op::Div(a, b) => {
                v(*a);
                v(*b);
            }
            Op::Rem(a, b) => {
                v(*a);
                v(*b);
            }
            Op::Dot(a, b) => {
                v(*a);
                v(*b);
            }
            Op::Cross(a, b) => {
                v(*a);
                v(*b);
            }
            Op::Neg(a) => v(*a),
            Op::Sin(a) => v(*a),
            Op::Cos(a) => v(*a),
            Op::Tan(a) => v(*a),
            Op::Asin(a) => v(*a),
            Op::Acos(a) => v(*a),
            Op::Atan(a) => v(*a),
            Op::Atan2(a, b) => {
                v(*a);
                v(*b);
            }
            Op::Sqrt(a) => v(*a),
            Op::Pow(a, b) => {
                v(*a);
                v(*b);
            }
            Op::Exp(a) => v(*a),
            Op::Ln(a) => v(*a),
            Op::Min(a, b) => {
                v(*a);
                v(*b);
            }
            Op::Max(a, b) => {
                v(*a);
                v(*b);
            }
            Op::Clamp(a, b, c) => {
                v(*a);
                v(*b);
                v(*c);
            }
            Op::Abs(a) => v(*a),
            Op::Sign(a) => v(*a),
            Op::Floor(a) => v(*a),
            Op::Fract(a) => v(*a),
            Op::Select(a, b, c) => {
                v(*a);
                v(*b);
                v(*c);
            }
            Op::Lerp(a, b, c) => {
                v(*a);
                v(*b);
                v(*c);
            }
            Op::Smoothstep(a, b, c) => {
                v(*a);
                v(*b);
                v(*c);
            }
            Op::Step(a, b) => {
                v(*a);
                v(*b);
            }
            Op::Eq(a, b) => {
                v(*a);
                v(*b);
            }
            Op::Ne(a, b) => {
                v(*a);
                v(*b);
            }
            Op::Lt(a, b) => {
                v(*a);
                v(*b);
            }
            Op::Le(a, b) => {
                v(*a);
                v(*b);
            }
            Op::Gt(a, b) => {
                v(*a);
                v(*b);
            }
            Op::Ge(a, b) => {
                v(*a);
                v(*b);
            }
            Op::And(a, b) => {
                v(*a);
                v(*b);
            }
            Op::Or(a, b) => {
                v(*a);
                v(*b);
            }
            Op::Xor(a, b) => {
                v(*a);
                v(*b);
            }
            Op::Not(a) => v(*a),
            Op::NewVec2(a, b) => {
                v(*a);
                v(*b);
            }
            Op::NewVec3(a, b, c) => {
                v(*a);
                v(*b);
                v(*c);
            }
            Op::NewVec4(a, b, c, d) => {
                v(*a);
                v(*b);
                v(*c);
                v(*d);
            }
            Op::SplatVec2(a) => v(*a),
            Op::SplatVec3(a) => v(*a),
            Op::SplatVec4(a) => v(*a),
            Op::CastFloat(a) => v(*a),
            Op::CastInt(a) => v(*a),
            Op::Length(a) => v(*a),
            Op::Normalize(a) => v(*a),
            Op::Swizzle1(a, _) => v(*a),
            Op::DerivX(a) => v(*a),
            Op::DerivY(a) => v(*a),
            Op::DerivWidth(a) => v(*a),

            Op::TextureSampleLinear(a, b) => {
                v(*a);
                v(*b);
            }
            Op::TextureSampleNearest(a, b) => {
                v(*a);
                v(*b);
            }
            Op::TextureSize(a) => {
                v(*a);
            }
            Op::SlotCreate(a) => {
                v(*a);
            }
            Op::SlotUpdate(a, b) => {
                v(*a);
                v(*b);
            }
            Op::LoopPush(a) => {
                v(*a);
            }
            Op::LoopPop => {}
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    Float1,
    Float2,
    Float3,
    Float4,

    Int1,
    Int2,
    Int3,
    Int4,

    Bool1,
    Bool2,
    Bool3,
    Bool4,

    Texture,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Swizzle {
    X,
    Y,
    Z,
    W,
}
