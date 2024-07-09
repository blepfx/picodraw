use crate::types::GlType;
use std::cell::RefCell;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::mem::replace;

thread_local! {
    static CURRENT_GRAPH: RefCell<Option<Vec<(ValueSource, ValueType)>>> = RefCell::new(None);
}

pub(crate) fn push_op(value: ValueSource, r#type: ValueType) -> Value {
    CURRENT_GRAPH.with(|graph| {
        let mut graph = graph.borrow_mut();
        let graph = graph
            .as_mut()
            .expect("executing not in a shader graph context");

        graph.push((value, r#type));
        Value((graph.len() - 1) as u32, PhantomData)
    })
}

#[derive(Clone, Debug)]
pub struct ShaderGraph<T> {
    values: Vec<(ValueSource, ValueType)>,
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

    pub fn get(&self, value: Value) -> (&ValueSource, ValueType) {
        let (src, ty) = self.values.get(value.0 as usize).expect("invalid value");
        (src, *ty)
    }

    pub fn iter<'a>(
        &'a self,
    ) -> impl Iterator<Item = (Value, &'a ValueSource, ValueType)>
           + DoubleEndedIterator
           + ExactSizeIterator
           + 'a {
        self.values
            .iter()
            .enumerate()
            .map(|(i, (source, ty))| (Value(i as u32, PhantomData), source, *ty))
    }

    pub fn result(&self) -> Value {
        self.result.unwrap()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Value(u32, PhantomData<*const ()>);

impl Value {
    pub fn id(&self) -> u32 {
        self.0
    }
}

impl Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "${}", self.0)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ValueSource {
    Input(String),

    Add(Value, Value),
    Sub(Value, Value),
    Mul(Value, Value),
    Div(Value, Value),
    Rem(Value, Value),
    Dot(Value, Value),
    Cross(Value, Value),
    Neg(Value),

    Sin(Value),
    Cos(Value),
    Tan(Value),

    Asin(Value),
    Acos(Value),
    Atan(Value),
    Atan2(Value, Value),

    Sqrt(Value),
    Pow(Value, Value),
    Exp(Value),
    Ln(Value),

    Min(Value, Value),
    Max(Value, Value),
    Clamp(Value, Value, Value),
    Abs(Value),
    Sign(Value),
    Floor(Value),
    Fract(Value),

    Lerp(Value, Value, Value),
    Smoothstep(Value, Value, Value),
    Step(Value, Value),

    LitFloat(f32),
    LitInt(i32),
    LitBool(bool),

    Eq(Value, Value),
    Ne(Value, Value),
    Lt(Value, Value),
    Le(Value, Value),
    Gt(Value, Value),
    Ge(Value, Value),

    And(Value, Value),
    Or(Value, Value),
    Xor(Value, Value),
    Not(Value),

    NewVec2(Value, Value),
    NewVec3(Value, Value, Value),
    NewVec4(Value, Value, Value, Value),
    SplatVec2(Value),
    SplatVec3(Value),
    SplatVec4(Value),

    CastFloat(Value),
    CastInt(Value),

    // CastBool(Value),
    // CastVec2(Value),
    // CastVec3(Value),
    // CastVec4(Value),
    Swizzle1(Value, Swizzle),
    // Swizzle2(Value, [Swizzle; 2]),
    // Swizzle3(Value, [Swizzle; 3]),
    // Swizzle4(Value, [Swizzle; 4]),
    Length(Value),
    Normalize(Value),

    DerivX(Value),
    DerivY(Value),
    DerivWidth(Value),

    TextureSampleLinear(Value, Value),
    TextureSampleNearest(Value, Value),
    TextureSize(Value),
}

impl ValueSource {
    pub fn visit_dependencies(&self, mut v: impl FnMut(Value)) {
        match self {
            ValueSource::Input(_) => {}

            ValueSource::LitFloat(_) => {}
            ValueSource::LitInt(_) => {}
            ValueSource::LitBool(_) => {}
            ValueSource::Add(a, b) => {
                v(*a);
                v(*b);
            }
            ValueSource::Sub(a, b) => {
                v(*a);
                v(*b);
            }
            ValueSource::Mul(a, b) => {
                v(*a);
                v(*b);
            }
            ValueSource::Div(a, b) => {
                v(*a);
                v(*b);
            }
            ValueSource::Rem(a, b) => {
                v(*a);
                v(*b);
            }
            ValueSource::Dot(a, b) => {
                v(*a);
                v(*b);
            }
            ValueSource::Cross(a, b) => {
                v(*a);
                v(*b);
            }
            ValueSource::Neg(a) => v(*a),
            ValueSource::Sin(a) => v(*a),
            ValueSource::Cos(a) => v(*a),
            ValueSource::Tan(a) => v(*a),
            ValueSource::Asin(a) => v(*a),
            ValueSource::Acos(a) => v(*a),
            ValueSource::Atan(a) => v(*a),
            ValueSource::Atan2(a, b) => {
                v(*a);
                v(*b);
            }
            ValueSource::Sqrt(a) => v(*a),
            ValueSource::Pow(a, b) => {
                v(*a);
                v(*b);
            }
            ValueSource::Exp(a) => v(*a),
            ValueSource::Ln(a) => v(*a),
            ValueSource::Min(a, b) => {
                v(*a);
                v(*b);
            }
            ValueSource::Max(a, b) => {
                v(*a);
                v(*b);
            }
            ValueSource::Clamp(a, b, c) => {
                v(*a);
                v(*b);
                v(*c);
            }
            ValueSource::Abs(a) => v(*a),
            ValueSource::Sign(a) => v(*a),
            ValueSource::Floor(a) => v(*a),
            ValueSource::Fract(a) => v(*a),
            ValueSource::Lerp(a, b, c) => {
                v(*a);
                v(*b);
                v(*c);
            }
            ValueSource::Smoothstep(a, b, c) => {
                v(*a);
                v(*b);
                v(*c);
            }
            ValueSource::Step(a, b) => {
                v(*a);
                v(*b);
            }
            ValueSource::Eq(a, b) => {
                v(*a);
                v(*b);
            }
            ValueSource::Ne(a, b) => {
                v(*a);
                v(*b);
            }
            ValueSource::Lt(a, b) => {
                v(*a);
                v(*b);
            }
            ValueSource::Le(a, b) => {
                v(*a);
                v(*b);
            }
            ValueSource::Gt(a, b) => {
                v(*a);
                v(*b);
            }
            ValueSource::Ge(a, b) => {
                v(*a);
                v(*b);
            }
            ValueSource::And(a, b) => {
                v(*a);
                v(*b);
            }
            ValueSource::Or(a, b) => {
                v(*a);
                v(*b);
            }
            ValueSource::Xor(a, b) => {
                v(*a);
                v(*b);
            }
            ValueSource::Not(a) => v(*a),
            ValueSource::NewVec2(a, b) => {
                v(*a);
                v(*b);
            }
            ValueSource::NewVec3(a, b, c) => {
                v(*a);
                v(*b);
                v(*c);
            }
            ValueSource::NewVec4(a, b, c, d) => {
                v(*a);
                v(*b);
                v(*c);
                v(*d);
            }
            ValueSource::SplatVec2(a) => v(*a),
            ValueSource::SplatVec3(a) => v(*a),
            ValueSource::SplatVec4(a) => v(*a),
            ValueSource::CastFloat(a) => v(*a),
            ValueSource::CastInt(a) => v(*a),
            ValueSource::Length(a) => v(*a),
            ValueSource::Normalize(a) => v(*a),
            ValueSource::Swizzle1(a, _) => v(*a),
            ValueSource::DerivX(a) => v(*a),
            ValueSource::DerivY(a) => v(*a),
            ValueSource::DerivWidth(a) => v(*a),

            ValueSource::TextureSampleLinear(a, b) => {
                v(*a);
                v(*b);
            }
            ValueSource::TextureSampleNearest(a, b) => {
                v(*a);
                v(*b);
            }
            ValueSource::TextureSize(a) => {
                v(*a);
            }
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
