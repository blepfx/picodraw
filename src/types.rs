use crate::graph::{push_op, Swizzle, Value, ValueSource, ValueType};
use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Neg, Not, Rem, Sub};

pub trait GlType: Copy + 'static {
    const TYPE: ValueType;
    fn wrap(value: Value) -> Self;
    fn unwrap(self) -> Value;

    fn input_raw(id: usize) -> Self {
        Self::wrap(push_op(ValueSource::Input(id), Self::TYPE))
    }
}

pub trait GlFloat:
    From<Float>
    + From<f32>
    + GlType
    + Add<Self, Output = Self>
    + Sub<Self, Output = Self>
    + Mul<Self, Output = Self>
    + Div<Self, Output = Self>
    + Rem<Self, Output = Self>
    + Add<f32, Output = Self>
    + Sub<f32, Output = Self>
    + Mul<f32, Output = Self>
    + Div<f32, Output = Self>
    + Rem<f32, Output = Self>
    + Neg<Output = Self>
{
    fn sin(self) -> Self;
    fn cos(self) -> Self;
    fn tan(self) -> Self;

    fn asin(self) -> Self;
    fn acos(self) -> Self;
    fn atan(self) -> Self;

    fn sqrt(self) -> Self;
    fn pow(self, power: impl Into<Self>) -> Self;
    fn exp(self) -> Self;
    fn ln(self) -> Self;

    fn floor(self) -> Self;
    fn fract(self) -> Self;
    fn abs(self) -> Self;
    fn sign(self) -> Self;
    fn min(self, x: impl Into<Self>) -> Self;
    fn max(self, x: impl Into<Self>) -> Self;
    fn clamp(self, min: impl Into<Self>, max: impl Into<Self>) -> Self;

    fn step(self, edge: impl Into<Self>) -> Self;
    fn smoothstep(self, min: impl Into<Self>, max: impl Into<Self>) -> Self;
    fn lerp(self, min: impl Into<Self>, max: impl Into<Self>) -> Self;
    fn select(self, other: impl Into<Self>, cond: impl Into<Bool>) -> Self;

    fn len(self) -> Float;
    fn norm(self) -> Self;
    fn dot(self, rhs: impl Into<Self>) -> Float;

    fn dfdx(self) -> Self;
    fn dfdy(self) -> Self;
    fn fwidth(self) -> Self;
}

#[derive(Clone, Copy)]
pub struct Float(pub(crate) Value);

impl Float {
    pub fn atan2(self, x: impl Into<Self>) -> Self {
        Self(push_op(
            ValueSource::Atan2(self.0, x.into().0),
            ValueType::Float1,
        ))
    }

    pub fn le(self, rhs: impl Into<Self>) -> Bool {
        Bool(push_op(
            ValueSource::Le(self.0, rhs.into().0),
            ValueType::Bool1,
        ))
    }

    pub fn lt(self, rhs: impl Into<Self>) -> Bool {
        Bool(push_op(
            ValueSource::Lt(self.0, rhs.into().0),
            ValueType::Bool1,
        ))
    }

    pub fn ge(self, rhs: impl Into<Self>) -> Bool {
        Bool(push_op(
            ValueSource::Ge(self.0, rhs.into().0),
            ValueType::Bool1,
        ))
    }

    pub fn gt(self, rhs: impl Into<Self>) -> Bool {
        Bool(push_op(
            ValueSource::Gt(self.0, rhs.into().0),
            ValueType::Bool1,
        ))
    }
}

impl From<f32> for Float {
    fn from(value: f32) -> Self {
        Self(push_op(ValueSource::LitFloat(value), ValueType::Float1))
    }
}

impl From<Int> for Float {
    fn from(value: Int) -> Self {
        Self(push_op(ValueSource::CastFloat(value.0), ValueType::Float1))
    }
}

impl From<Bool> for Float {
    fn from(value: Bool) -> Self {
        Self(push_op(ValueSource::CastFloat(value.0), ValueType::Float1))
    }
}

#[derive(Clone, Copy)]
pub struct Int(pub(crate) Value);

impl Int {
    pub fn le(self, rhs: impl Into<Self>) -> Bool {
        Bool(push_op(
            ValueSource::Le(self.0, rhs.into().0),
            ValueType::Bool1,
        ))
    }

    pub fn lt(self, rhs: impl Into<Self>) -> Bool {
        Bool(push_op(
            ValueSource::Lt(self.0, rhs.into().0),
            ValueType::Bool1,
        ))
    }

    pub fn ge(self, rhs: impl Into<Self>) -> Bool {
        Bool(push_op(
            ValueSource::Ge(self.0, rhs.into().0),
            ValueType::Bool1,
        ))
    }

    pub fn gt(self, rhs: impl Into<Self>) -> Bool {
        Bool(push_op(
            ValueSource::Gt(self.0, rhs.into().0),
            ValueType::Bool1,
        ))
    }

    pub fn eq(self, rhs: impl Into<Self>) -> Bool {
        Bool(push_op(
            ValueSource::Eq(self.0, rhs.into().0),
            ValueType::Bool1,
        ))
    }

    pub fn neq(self, rhs: impl Into<Self>) -> Bool {
        Bool(push_op(
            ValueSource::Ne(self.0, rhs.into().0),
            ValueType::Bool1,
        ))
    }
}

impl From<i32> for Int {
    fn from(value: i32) -> Self {
        Self(push_op(ValueSource::LitInt(value), ValueType::Int1))
    }
}

impl From<Bool> for Int {
    fn from(value: Bool) -> Self {
        Self(push_op(ValueSource::CastInt(value.0), ValueType::Int1))
    }
}

impl From<Float> for Int {
    fn from(value: Float) -> Self {
        Self(push_op(ValueSource::CastInt(value.0), ValueType::Int1))
    }
}

#[derive(Clone, Copy)]
pub struct Bool(pub(crate) Value);

#[derive(Clone, Copy)]
pub struct Float2(pub(crate) Value);

impl Float2 {
    pub fn new(x: impl Into<Float>, y: impl Into<Float>) -> Self {
        Self(push_op(
            ValueSource::NewVec2(x.into().0, y.into().0),
            ValueType::Float2,
        ))
    }

    pub fn x(self) -> Float {
        Float(push_op(
            ValueSource::Swizzle1(self.0, Swizzle::X),
            ValueType::Float1,
        ))
    }

    pub fn y(self) -> Float {
        Float(push_op(
            ValueSource::Swizzle1(self.0, Swizzle::Y),
            ValueType::Float1,
        ))
    }
}

impl From<Float> for Float2 {
    fn from(value: Float) -> Self {
        Self(push_op(ValueSource::SplatVec2(value.0), ValueType::Float2))
    }
}

impl From<f32> for Float2 {
    fn from(value: f32) -> Self {
        Self::from(Float::from(value))
    }
}

#[derive(Clone, Copy)]
pub struct Float3(pub(crate) Value);

impl Float3 {
    pub fn new(x: impl Into<Float>, y: impl Into<Float>, z: impl Into<Float>) -> Self {
        Self(push_op(
            ValueSource::NewVec3(x.into().0, y.into().0, z.into().0),
            ValueType::Float3,
        ))
    }

    pub fn x(self) -> Float {
        Float(push_op(
            ValueSource::Swizzle1(self.0, Swizzle::X),
            ValueType::Float1,
        ))
    }

    pub fn y(self) -> Float {
        Float(push_op(
            ValueSource::Swizzle1(self.0, Swizzle::Y),
            ValueType::Float1,
        ))
    }

    pub fn z(self) -> Float {
        Float(push_op(
            ValueSource::Swizzle1(self.0, Swizzle::Z),
            ValueType::Float1,
        ))
    }

    pub fn cross(self, rhs: impl Into<Self>) -> Self {
        Self(push_op(
            ValueSource::Cross(self.0, rhs.into().0),
            ValueType::Float3,
        ))
    }
}

impl From<Float> for Float3 {
    fn from(value: Float) -> Self {
        Self(push_op(ValueSource::SplatVec3(value.0), ValueType::Float3))
    }
}

impl From<f32> for Float3 {
    fn from(value: f32) -> Self {
        Self::from(Float::from(value))
    }
}

#[derive(Clone, Copy)]
pub struct Float4(pub(crate) Value);

impl Float4 {
    pub fn new(
        x: impl Into<Float>,
        y: impl Into<Float>,
        z: impl Into<Float>,
        w: impl Into<Float>,
    ) -> Self {
        Self(push_op(
            ValueSource::NewVec4(x.into().0, y.into().0, z.into().0, w.into().0),
            ValueType::Float4,
        ))
    }

    pub fn x(self) -> Float {
        Float(push_op(
            ValueSource::Swizzle1(self.0, Swizzle::X),
            ValueType::Float1,
        ))
    }

    pub fn y(self) -> Float {
        Float(push_op(
            ValueSource::Swizzle1(self.0, Swizzle::Y),
            ValueType::Float1,
        ))
    }

    pub fn z(self) -> Float {
        Float(push_op(
            ValueSource::Swizzle1(self.0, Swizzle::Z),
            ValueType::Float1,
        ))
    }

    pub fn w(self) -> Float {
        Float(push_op(
            ValueSource::Swizzle1(self.0, Swizzle::W),
            ValueType::Float1,
        ))
    }
}

impl From<Float> for Float4 {
    fn from(value: Float) -> Self {
        Self(push_op(ValueSource::SplatVec4(value.0), ValueType::Float4))
    }
}

impl From<f32> for Float4 {
    fn from(value: f32) -> Self {
        Self::from(Float::from(value))
    }
}

#[derive(Copy, Clone)]
pub struct Texture(Value);

impl GlType for Texture {
    const TYPE: ValueType = ValueType::Texture;

    fn wrap(value: Value) -> Self {
        Self(value)
    }

    fn unwrap(self) -> Value {
        self.0
    }
}

impl Texture {
    pub fn linear(&self, pos: impl Into<Float2>) -> Float4 {
        Float4(push_op(
            ValueSource::TextureSampleLinear(self.0, pos.into().0),
            ValueType::Float4,
        ))
    }

    pub fn nearest(&self, pos: impl Into<Float2>) -> Float4 {
        Float4(push_op(
            ValueSource::TextureSampleNearest(self.0, pos.into().0),
            ValueType::Float4,
        ))
    }

    pub fn size(&self) -> Float2 {
        Float2(push_op(ValueSource::TextureSize(self.0), ValueType::Float2))
    }
}

macro_rules! impl_float {
    ($type:ty, $vtype:ident) => {
        impl GlType for $type {
            const TYPE: ValueType = ValueType::$vtype;
            fn wrap(value: Value) -> Self {
                Self(value)
            }

            fn unwrap(self) -> Value {
                self.0
            }
        }

        impl GlFloat for $type {
            fn sin(self) -> Self {
                Self(push_op(ValueSource::Sin(self.0), ValueType::$vtype))
            }

            fn cos(self) -> Self {
                Self(push_op(ValueSource::Cos(self.0), ValueType::$vtype))
            }

            fn tan(self) -> Self {
                Self(push_op(ValueSource::Tan(self.0), ValueType::$vtype))
            }

            fn asin(self) -> Self {
                Self(push_op(ValueSource::Asin(self.0), ValueType::$vtype))
            }

            fn acos(self) -> Self {
                Self(push_op(ValueSource::Acos(self.0), ValueType::$vtype))
            }

            fn atan(self) -> Self {
                Self(push_op(ValueSource::Atan(self.0), ValueType::$vtype))
            }

            fn sqrt(self) -> Self {
                Self(push_op(ValueSource::Sqrt(self.0), ValueType::$vtype))
            }

            fn pow(self, power: impl Into<Self>) -> Self {
                Self(push_op(
                    ValueSource::Pow(self.0, power.into().0),
                    ValueType::$vtype,
                ))
            }

            fn exp(self) -> Self {
                Self(push_op(ValueSource::Exp(self.0), ValueType::$vtype))
            }

            fn ln(self) -> Self {
                Self(push_op(ValueSource::Ln(self.0), ValueType::$vtype))
            }

            fn floor(self) -> Self {
                Self(push_op(ValueSource::Floor(self.0), ValueType::$vtype))
            }

            fn fract(self) -> Self {
                Self(push_op(ValueSource::Fract(self.0), ValueType::$vtype))
            }

            fn abs(self) -> Self {
                Self(push_op(ValueSource::Abs(self.0), ValueType::$vtype))
            }

            fn sign(self) -> Self {
                Self(push_op(ValueSource::Sign(self.0), ValueType::$vtype))
            }

            fn min(self, x: impl Into<Self>) -> Self {
                Self(push_op(
                    ValueSource::Min(self.0, x.into().0),
                    ValueType::$vtype,
                ))
            }

            fn max(self, x: impl Into<Self>) -> Self {
                Self(push_op(
                    ValueSource::Max(self.0, x.into().0),
                    ValueType::$vtype,
                ))
            }

            fn clamp(self, min: impl Into<Self>, max: impl Into<Self>) -> Self {
                Self(push_op(
                    ValueSource::Clamp(self.0, min.into().0, max.into().0),
                    ValueType::$vtype,
                ))
            }

            fn step(self, edge: impl Into<Self>) -> Self {
                Self(push_op(
                    ValueSource::Step(self.0, edge.into().0),
                    ValueType::$vtype,
                ))
            }

            fn smoothstep(self, min: impl Into<Self>, max: impl Into<Self>) -> Self {
                Self(push_op(
                    ValueSource::Smoothstep(self.0, min.into().0, max.into().0),
                    ValueType::$vtype,
                ))
            }

            fn lerp(self, min: impl Into<Self>, max: impl Into<Self>) -> Self {
                Self(push_op(
                    ValueSource::Lerp(self.0, min.into().0, max.into().0),
                    ValueType::$vtype,
                ))
            }

            fn select(self, other: impl Into<Self>, cond: impl Into<Bool>) -> Self {
                Self(push_op(
                    ValueSource::Select(cond.into().0, self.0, other.into().0),
                    ValueType::$vtype,
                ))
            }

            fn norm(self) -> Self {
                if ValueType::$vtype == ValueType::Float1 {
                    Self(push_op(ValueSource::Sign(self.0), ValueType::Float1))
                } else {
                    Self(push_op(ValueSource::Normalize(self.0), ValueType::$vtype))
                }
            }

            fn len(self) -> Float {
                if ValueType::$vtype == ValueType::Float1 {
                    Float(push_op(ValueSource::Abs(self.0), ValueType::Float1))
                } else {
                    Float(push_op(ValueSource::Length(self.0), ValueType::Float1))
                }
            }

            fn dot(self, rhs: impl Into<Self>) -> Float {
                if ValueType::$vtype == ValueType::Float1 {
                    Float(push_op(
                        ValueSource::Mul(self.0, rhs.into().0),
                        ValueType::Float1,
                    ))
                } else {
                    Float(push_op(
                        ValueSource::Dot(self.0, rhs.into().0),
                        ValueType::Float1,
                    ))
                }
            }

            fn dfdx(self) -> Self {
                Self(push_op(ValueSource::DerivX(self.0), ValueType::Float1))
            }

            fn dfdy(self) -> Self {
                Self(push_op(ValueSource::DerivY(self.0), ValueType::Float1))
            }

            fn fwidth(self) -> Self {
                Self(push_op(ValueSource::DerivWidth(self.0), ValueType::Float1))
            }
        }

        impl Add<$type> for $type {
            type Output = $type;
            fn add(self, rhs: $type) -> Self::Output {
                Self(push_op(ValueSource::Add(self.0, rhs.0), ValueType::$vtype))
            }
        }

        impl Sub<$type> for $type {
            type Output = $type;
            fn sub(self, rhs: $type) -> Self::Output {
                Self(push_op(ValueSource::Sub(self.0, rhs.0), ValueType::$vtype))
            }
        }

        impl Mul<$type> for $type {
            type Output = $type;
            fn mul(self, rhs: $type) -> Self::Output {
                Self(push_op(ValueSource::Mul(self.0, rhs.0), ValueType::$vtype))
            }
        }

        impl Div<$type> for $type {
            type Output = $type;
            fn div(self, rhs: $type) -> Self::Output {
                Self(push_op(ValueSource::Div(self.0, rhs.0), ValueType::$vtype))
            }
        }

        impl Rem<$type> for $type {
            type Output = $type;
            fn rem(self, rhs: $type) -> Self::Output {
                Self(push_op(ValueSource::Rem(self.0, rhs.0), ValueType::$vtype))
            }
        }

        impl Neg for $type {
            type Output = $type;
            fn neg(self) -> Self::Output {
                Self(push_op(ValueSource::Neg(self.0), ValueType::$vtype))
            }
        }

        impl Add<f32> for $type {
            type Output = Self;
            fn add(self, rhs: f32) -> Self::Output {
                self + <$type>::from(rhs)
            }
        }

        impl Sub<f32> for $type {
            type Output = Self;
            fn sub(self, rhs: f32) -> Self::Output {
                self - <$type>::from(rhs)
            }
        }

        impl Mul<f32> for $type {
            type Output = Self;
            fn mul(self, rhs: f32) -> Self::Output {
                self * <$type>::from(rhs)
            }
        }

        impl Div<f32> for $type {
            type Output = Self;
            fn div(self, rhs: f32) -> Self::Output {
                self / <$type>::from(rhs)
            }
        }

        impl Rem<f32> for $type {
            type Output = Self;
            fn rem(self, rhs: f32) -> Self::Output {
                self % <$type>::from(rhs)
            }
        }

        impl Add<$type> for f32 {
            type Output = $type;
            fn add(self, rhs: $type) -> Self::Output {
                <$type>::from(self) + rhs
            }
        }

        impl Sub<$type> for f32 {
            type Output = $type;
            fn sub(self, rhs: $type) -> Self::Output {
                <$type>::from(self) - rhs
            }
        }

        impl Mul<$type> for f32 {
            type Output = $type;
            fn mul(self, rhs: $type) -> Self::Output {
                <$type>::from(self) * rhs
            }
        }

        impl Div<$type> for f32 {
            type Output = $type;
            fn div(self, rhs: $type) -> Self::Output {
                <$type>::from(self) / rhs
            }
        }

        impl Rem<$type> for f32 {
            type Output = $type;
            fn rem(self, rhs: $type) -> Self::Output {
                <$type>::from(self) % rhs
            }
        }
    };
}

macro_rules! impl_int {
    ($type:ty, $vtype:ident) => {
        impl GlType for $type {
            const TYPE: ValueType = ValueType::$vtype;
            fn wrap(value: Value) -> Self {
                Self(value)
            }

            fn unwrap(self) -> Value {
                self.0
            }
        }

        impl $type {
            pub fn min(self, x: impl Into<Self>) -> Self {
                Self(push_op(
                    ValueSource::Min(self.0, x.into().0),
                    ValueType::$vtype,
                ))
            }

            pub fn max(self, x: impl Into<Self>) -> Self {
                Self(push_op(
                    ValueSource::Max(self.0, x.into().0),
                    ValueType::$vtype,
                ))
            }

            pub fn clamp(self, min: impl Into<Self>, max: impl Into<Self>) -> Self {
                Self(push_op(
                    ValueSource::Clamp(self.0, min.into().0, max.into().0),
                    ValueType::$vtype,
                ))
            }
        }

        impl Add<$type> for $type {
            type Output = $type;
            fn add(self, rhs: $type) -> Self::Output {
                Self(push_op(ValueSource::Add(self.0, rhs.0), ValueType::$vtype))
            }
        }

        impl Sub<$type> for $type {
            type Output = $type;
            fn sub(self, rhs: $type) -> Self::Output {
                Self(push_op(ValueSource::Sub(self.0, rhs.0), ValueType::$vtype))
            }
        }

        impl Mul<$type> for $type {
            type Output = $type;
            fn mul(self, rhs: $type) -> Self::Output {
                Self(push_op(ValueSource::Mul(self.0, rhs.0), ValueType::$vtype))
            }
        }

        impl Div<$type> for $type {
            type Output = $type;
            fn div(self, rhs: $type) -> Self::Output {
                Self(push_op(ValueSource::Div(self.0, rhs.0), ValueType::$vtype))
            }
        }

        impl Rem<$type> for $type {
            type Output = $type;
            fn rem(self, rhs: $type) -> Self::Output {
                Self(push_op(ValueSource::Rem(self.0, rhs.0), ValueType::$vtype))
            }
        }

        impl Neg for $type {
            type Output = $type;
            fn neg(self) -> Self::Output {
                Self(push_op(ValueSource::Neg(self.0), ValueType::$vtype))
            }
        }

        impl Add<i32> for $type {
            type Output = Self;
            fn add(self, rhs: i32) -> Self::Output {
                self + <$type>::from(rhs)
            }
        }

        impl Sub<i32> for $type {
            type Output = Self;
            fn sub(self, rhs: i32) -> Self::Output {
                self - <$type>::from(rhs)
            }
        }

        impl Mul<i32> for $type {
            type Output = Self;
            fn mul(self, rhs: i32) -> Self::Output {
                self * <$type>::from(rhs)
            }
        }

        impl Div<i32> for $type {
            type Output = Self;
            fn div(self, rhs: i32) -> Self::Output {
                self / <$type>::from(rhs)
            }
        }

        impl Rem<i32> for $type {
            type Output = Self;
            fn rem(self, rhs: i32) -> Self::Output {
                self % <$type>::from(rhs)
            }
        }

        impl Add<$type> for i32 {
            type Output = $type;
            fn add(self, rhs: $type) -> Self::Output {
                <$type>::from(self) + rhs
            }
        }

        impl Sub<$type> for i32 {
            type Output = $type;
            fn sub(self, rhs: $type) -> Self::Output {
                <$type>::from(self) - rhs
            }
        }

        impl Mul<$type> for i32 {
            type Output = $type;
            fn mul(self, rhs: $type) -> Self::Output {
                <$type>::from(self) * rhs
            }
        }

        impl Div<$type> for i32 {
            type Output = $type;
            fn div(self, rhs: $type) -> Self::Output {
                <$type>::from(self) / rhs
            }
        }

        impl Rem<$type> for i32 {
            type Output = $type;
            fn rem(self, rhs: $type) -> Self::Output {
                <$type>::from(self) % rhs
            }
        }
    };
}

macro_rules! impl_bool {
    ($type:ty, $vtype:ident) => {
        impl GlType for $type {
            const TYPE: ValueType = ValueType::$vtype;
            fn wrap(value: Value) -> Self {
                Self(value)
            }

            fn unwrap(self) -> Value {
                self.0
            }
        }

        impl From<bool> for $type {
            fn from(value: bool) -> Self {
                Self(push_op(ValueSource::LitBool(value), ValueType::$vtype))
            }
        }

        impl BitAnd<$type> for $type {
            type Output = Self;
            fn bitand(self, rhs: $type) -> Self::Output {
                Self(push_op(ValueSource::And(self.0, rhs.0), ValueType::$vtype))
            }
        }

        impl BitOr<$type> for $type {
            type Output = Self;
            fn bitor(self, rhs: $type) -> Self::Output {
                Self(push_op(ValueSource::Or(self.0, rhs.0), ValueType::$vtype))
            }
        }

        impl BitXor<$type> for $type {
            type Output = Self;
            fn bitxor(self, rhs: $type) -> Self::Output {
                Self(push_op(ValueSource::Xor(self.0, rhs.0), ValueType::$vtype))
            }
        }

        impl Not for $type {
            type Output = Self;
            fn not(self) -> Self::Output {
                Self(push_op(ValueSource::Not(self.0), ValueType::$vtype))
            }
        }

        impl BitAnd<bool> for $type {
            type Output = Self;
            fn bitand(self, rhs: bool) -> Self::Output {
                if rhs {
                    self
                } else {
                    Self::from(false)
                }
            }
        }

        impl BitOr<bool> for $type {
            type Output = Self;
            fn bitor(self, rhs: bool) -> Self::Output {
                if rhs {
                    Self::from(true)
                } else {
                    self
                }
            }
        }

        impl BitXor<bool> for $type {
            type Output = Self;
            fn bitxor(self, rhs: bool) -> Self::Output {
                if rhs {
                    !self
                } else {
                    self
                }
            }
        }
    };
}

macro_rules! impl_float_vec {
    ($type:ty, $vtype:ident) => {
        impl_float!($type, $vtype);

        impl Add<Float> for $type {
            type Output = Self;
            fn add(self, rhs: Float) -> Self::Output {
                self + <$type>::from(rhs)
            }
        }

        impl Sub<Float> for $type {
            type Output = Self;
            fn sub(self, rhs: Float) -> Self::Output {
                self - <$type>::from(rhs)
            }
        }

        impl Mul<Float> for $type {
            type Output = Self;
            fn mul(self, rhs: Float) -> Self::Output {
                self * <$type>::from(rhs)
            }
        }

        impl Div<Float> for $type {
            type Output = Self;
            fn div(self, rhs: Float) -> Self::Output {
                self / <$type>::from(rhs)
            }
        }

        impl Rem<Float> for $type {
            type Output = Self;
            fn rem(self, rhs: Float) -> Self::Output {
                self % <$type>::from(rhs)
            }
        }

        impl Add<$type> for Float {
            type Output = $type;
            fn add(self, rhs: $type) -> Self::Output {
                <$type>::from(self) + rhs
            }
        }

        impl Sub<$type> for Float {
            type Output = $type;
            fn sub(self, rhs: $type) -> Self::Output {
                <$type>::from(self) - rhs
            }
        }

        impl Mul<$type> for Float {
            type Output = $type;
            fn mul(self, rhs: $type) -> Self::Output {
                <$type>::from(self) * rhs
            }
        }

        impl Div<$type> for Float {
            type Output = $type;
            fn div(self, rhs: $type) -> Self::Output {
                <$type>::from(self) / rhs
            }
        }

        impl Rem<$type> for Float {
            type Output = $type;
            fn rem(self, rhs: $type) -> Self::Output {
                <$type>::from(self) % rhs
            }
        }
    };
}

impl_float!(Float, Float1);
impl_float_vec!(Float2, Float2);
impl_float_vec!(Float3, Float3);
impl_float_vec!(Float4, Float4);
impl_int!(Int, Int1);
impl_bool!(Bool, Bool1);
