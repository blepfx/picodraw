#![allow(non_camel_case_types)]

pub use types::*;
pub mod types {
    use crate::{
        TextureFilter,
        graph::{Graph, OpAddr, OpLiteral, OpValue},
    };
    use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Neg, Not, Rem, Shl, Shr, Sub};

    #[doc(hidden)]
    pub trait Select {
        fn select(x: Self, y: Self, switch: boolean) -> Self;
    }

    #[doc(hidden)]
    pub trait Type {
        fn into_addr(self) -> OpAddr;
        fn from_addr(addr: OpAddr) -> Self;
    }

    #[derive(Debug, Clone, Copy)]
    pub struct float1(pub(crate) OpAddr);
    #[derive(Debug, Clone, Copy)]
    pub struct float2(pub(crate) OpAddr);
    #[derive(Debug, Clone, Copy)]
    pub struct float3(pub(crate) OpAddr);
    #[derive(Debug, Clone, Copy)]
    pub struct float4(pub(crate) OpAddr);

    #[derive(Debug, Clone, Copy)]
    pub struct int1(pub(crate) OpAddr);
    #[derive(Debug, Clone, Copy)]
    pub struct int2(pub(crate) OpAddr);
    #[derive(Debug, Clone, Copy)]
    pub struct int3(pub(crate) OpAddr);
    #[derive(Debug, Clone, Copy)]
    pub struct int4(pub(crate) OpAddr);

    #[derive(Debug, Clone, Copy)]
    pub struct boolean(pub(crate) OpAddr);

    #[derive(Debug, Clone, Copy)]
    pub struct texture(pub(crate) OpAddr);

    macro_rules! impl_binop {
        ($type:ty, $elem:ty) => {
            impl Add<$elem> for $type {
                type Output = Self;
                fn add(self, rhs: $elem) -> Self::Output {
                    self + <$type>::from(rhs)
                }
            }

            impl Sub<$elem> for $type {
                type Output = Self;
                fn sub(self, rhs: $elem) -> Self::Output {
                    self - <$type>::from(rhs)
                }
            }

            impl Mul<$elem> for $type {
                type Output = Self;
                fn mul(self, rhs: $elem) -> Self::Output {
                    self * <$type>::from(rhs)
                }
            }

            impl Div<$elem> for $type {
                type Output = Self;
                fn div(self, rhs: $elem) -> Self::Output {
                    self / <$type>::from(rhs)
                }
            }

            impl Rem<$elem> for $type {
                type Output = Self;
                fn rem(self, rhs: $elem) -> Self::Output {
                    self % <$type>::from(rhs)
                }
            }

            impl Add<$type> for $elem {
                type Output = $type;
                fn add(self, rhs: $type) -> Self::Output {
                    <$type>::from(self) + rhs
                }
            }

            impl Sub<$type> for $elem {
                type Output = $type;
                fn sub(self, rhs: $type) -> Self::Output {
                    <$type>::from(self) - rhs
                }
            }

            impl Mul<$type> for $elem {
                type Output = $type;
                fn mul(self, rhs: $type) -> Self::Output {
                    <$type>::from(self) * rhs
                }
            }

            impl Div<$type> for $elem {
                type Output = $type;
                fn div(self, rhs: $type) -> Self::Output {
                    <$type>::from(self) / rhs
                }
            }

            impl Rem<$type> for $elem {
                type Output = $type;
                fn rem(self, rhs: $type) -> Self::Output {
                    <$type>::from(self) % rhs
                }
            }
        };
    }

    macro_rules! impl_float {
        ($type:ty, $int:ty) => {
            impl $type {
                pub fn sin(self) -> Self {
                    Self(Graph::push_collect(OpValue::Sin(self.0)))
                }

                pub fn cos(self) -> Self {
                    Self(Graph::push_collect(OpValue::Cos(self.0)))
                }

                pub fn tan(self) -> Self {
                    Self(Graph::push_collect(OpValue::Tan(self.0)))
                }

                pub fn asin(self) -> Self {
                    Self(Graph::push_collect(OpValue::Asin(self.0)))
                }

                pub fn acos(self) -> Self {
                    Self(Graph::push_collect(OpValue::Acos(self.0)))
                }

                pub fn atan(self) -> Self {
                    Self(Graph::push_collect(OpValue::Atan(self.0)))
                }

                pub fn sqrt(self) -> Self {
                    Self(Graph::push_collect(OpValue::Sqrt(self.0)))
                }

                pub fn exp(self) -> Self {
                    Self(Graph::push_collect(OpValue::Exp(self.0)))
                }

                pub fn ln(self) -> Self {
                    Self(Graph::push_collect(OpValue::Ln(self.0)))
                }

                pub fn floor(self) -> Self {
                    Self(Graph::push_collect(OpValue::Floor(self.0)))
                }

                pub fn dx(self) -> Self {
                    Self(Graph::push_collect(OpValue::DerivX(self.0)))
                }

                pub fn dy(self) -> Self {
                    Self(Graph::push_collect(OpValue::DerivY(self.0)))
                }

                pub fn fwidth(self) -> Self {
                    Self(Graph::push_collect(OpValue::DerivWidth(self.0)))
                }

                pub fn abs(self) -> Self {
                    Self(Graph::push_collect(OpValue::Abs(self.0)))
                }

                pub fn sign(self) -> Self {
                    Self(Graph::push_collect(OpValue::Sign(self.0)))
                }

                pub fn dot(self, rhs: impl Into<Self>) -> float1 {
                    float1(Graph::push_collect(OpValue::Dot(self.0, rhs.into().0)))
                }

                pub fn min(self, rhs: impl Into<Self>) -> Self {
                    Self(Graph::push_collect(OpValue::Min(self.0, rhs.into().0)))
                }

                pub fn max(self, rhs: impl Into<Self>) -> Self {
                    Self(Graph::push_collect(OpValue::Max(self.0, rhs.into().0)))
                }

                pub fn atan2(self, rhs: impl Into<Self>) -> Self {
                    Self(Graph::push_collect(OpValue::Atan2(self.0, rhs.into().0)))
                }

                pub fn pow(self, rhs: impl Into<Self>) -> Self {
                    Self(Graph::push_collect(OpValue::Pow(self.0, rhs.into().0)))
                }

                pub fn step(self, min: impl Into<Self>) -> Self {
                    Self(Graph::push_collect(OpValue::Step(self.0, min.into().0)))
                }

                pub fn len(self) -> float1 {
                    float1(Graph::push_collect(OpValue::Length(self.0)))
                }

                pub fn norm(self) -> Self {
                    Self(Graph::push_collect(OpValue::Normalize(self.0)))
                }

                pub fn clamp(self, min: impl Into<Self>, max: impl Into<Self>) -> Self {
                    Self(Graph::push_collect(OpValue::Clamp(
                        self.0,
                        min.into().0,
                        max.into().0,
                    )))
                }

                pub fn lerp(self, min: impl Into<Self>, max: impl Into<Self>) -> Self {
                    Self(Graph::push_collect(OpValue::Lerp(
                        self.0,
                        min.into().0,
                        max.into().0,
                    )))
                }

                pub fn smoothstep(self, min: impl Into<Self>, max: impl Into<Self>) -> Self {
                    Self(Graph::push_collect(OpValue::Smoothstep(
                        self.0,
                        min.into().0,
                        max.into().0,
                    )))
                }
            }

            impl From<$int> for $type {
                fn from(x: $int) -> Self {
                    Self(Graph::push_collect(OpValue::CastFloat(x.0)))
                }
            }

            impl From<$type> for $int {
                fn from(x: $type) -> Self {
                    Self(Graph::push_collect(OpValue::CastInt(x.0)))
                }
            }
        };
    }

    macro_rules! impl_num_base {
        ($type:ty, $elem:ty) => {
            impl Select for $type {
                fn select(x: Self, y: Self, switch: boolean) -> Self {
                    Self(Graph::push_collect(OpValue::Select(switch.0, x.0, y.0)))
                }
            }

            impl Type for $type {
                fn into_addr(self) -> OpAddr {
                    self.0
                }
                fn from_addr(addr: OpAddr) -> Self {
                    Self(addr)
                }
            }

            impl Add<$type> for $type {
                type Output = $type;
                fn add(self, rhs: $type) -> Self::Output {
                    Self(Graph::push_collect(OpValue::Add(self.0, rhs.0)))
                }
            }

            impl Sub<$type> for $type {
                type Output = $type;
                fn sub(self, rhs: $type) -> Self::Output {
                    Self(Graph::push_collect(OpValue::Sub(self.0, rhs.0)))
                }
            }

            impl Mul<$type> for $type {
                type Output = $type;
                fn mul(self, rhs: $type) -> Self::Output {
                    Self(Graph::push_collect(OpValue::Mul(self.0, rhs.0)))
                }
            }

            impl Div<$type> for $type {
                type Output = $type;
                fn div(self, rhs: $type) -> Self::Output {
                    Self(Graph::push_collect(OpValue::Div(self.0, rhs.0)))
                }
            }

            impl Rem<$type> for $type {
                type Output = $type;
                fn rem(self, rhs: $type) -> Self::Output {
                    Self(Graph::push_collect(OpValue::Rem(self.0, rhs.0)))
                }
            }

            impl Neg for $type {
                type Output = $type;
                fn neg(self) -> Self::Output {
                    Self(Graph::push_collect(OpValue::Neg(self.0)))
                }
            }

            impl $type {
                pub fn eq(self, other: impl Into<Self>) -> boolean {
                    boolean(Graph::push_collect(OpValue::Eq(self.0, other.into().0)))
                }

                pub fn ne(self, other: impl Into<Self>) -> boolean {
                    boolean(Graph::push_collect(OpValue::Ne(self.0, other.into().0)))
                }

                pub fn le(self, other: impl Into<Self>) -> boolean {
                    boolean(Graph::push_collect(OpValue::Le(self.0, other.into().0)))
                }

                pub fn ge(self, other: impl Into<Self>) -> boolean {
                    boolean(Graph::push_collect(OpValue::Ge(self.0, other.into().0)))
                }

                pub fn lt(self, other: impl Into<Self>) -> boolean {
                    boolean(Graph::push_collect(OpValue::Lt(self.0, other.into().0)))
                }

                pub fn gt(self, other: impl Into<Self>) -> boolean {
                    boolean(Graph::push_collect(OpValue::Gt(self.0, other.into().0)))
                }
            }
        };
    }

    macro_rules! impl_num_vec {
        ($type:ty, $elem:ty, $scalar:ident, 1) => {
            impl_num_base!($type, $elem);
            impl_binop!($type, $elem);
        };

        ($type:ty, $elem:ty, $scalar:ident, 2) => {
            impl_num_base!($type, $elem);
            impl_binop!($type, $elem);
            impl_binop!($type, $scalar);

            impl $type {
                pub fn x(self) -> $scalar {
                    $scalar(Graph::push_collect(OpValue::ExtractX(self.0)))
                }

                pub fn y(self) -> $scalar {
                    $scalar(Graph::push_collect(OpValue::ExtractY(self.0)))
                }
            }

            impl From<$scalar> for $type {
                fn from(x: $scalar) -> Self {
                    Self(Graph::push_collect(OpValue::Splat2(x.0)))
                }
            }

            impl From<$elem> for $type {
                fn from(x: $elem) -> Self {
                    Self::from(<$scalar>::from(x))
                }
            }

            impl<X: Into<$scalar>, Y: Into<$scalar>> From<(X, Y)> for $type {
                fn from((x, y): (X, Y)) -> Self {
                    Self(Graph::push_collect(OpValue::Vec2(
                        Into::<$scalar>::into(x).0,
                        Into::<$scalar>::into(y).0,
                    )))
                }
            }
        };

        ($type:ty, $elem:ty, $scalar:ident, 3) => {
            impl_num_base!($type, $elem);
            impl_binop!($type, $elem);
            impl_binop!($type, $scalar);

            impl $type {
                pub fn x(self) -> $scalar {
                    $scalar(Graph::push_collect(OpValue::ExtractX(self.0)))
                }

                pub fn y(self) -> $scalar {
                    $scalar(Graph::push_collect(OpValue::ExtractY(self.0)))
                }

                pub fn z(self) -> $scalar {
                    $scalar(Graph::push_collect(OpValue::ExtractZ(self.0)))
                }
            }

            impl From<$scalar> for $type {
                fn from(x: $scalar) -> Self {
                    Self(Graph::push_collect(OpValue::Splat3(x.0)))
                }
            }

            impl From<$elem> for $type {
                fn from(x: $elem) -> Self {
                    Self::from(<$scalar>::from(x))
                }
            }

            impl<X: Into<$scalar>, Y: Into<$scalar>, Z: Into<$scalar>> From<(X, Y, Z)> for $type {
                fn from((x, y, z): (X, Y, Z)) -> Self {
                    Self(Graph::push_collect(OpValue::Vec3(
                        Into::<$scalar>::into(x).0,
                        Into::<$scalar>::into(y).0,
                        Into::<$scalar>::into(z).0,
                    )))
                }
            }
        };

        ($type:ty, $elem:ty, $scalar:ident, 4) => {
            impl_num_base!($type, $elem);
            impl_binop!($type, $elem);
            impl_binop!($type, $scalar);

            impl $type {
                pub fn x(self) -> $scalar {
                    $scalar(Graph::push_collect(OpValue::ExtractX(self.0)))
                }

                pub fn y(self) -> $scalar {
                    $scalar(Graph::push_collect(OpValue::ExtractY(self.0)))
                }

                pub fn z(self) -> $scalar {
                    $scalar(Graph::push_collect(OpValue::ExtractZ(self.0)))
                }

                pub fn w(self) -> $scalar {
                    $scalar(Graph::push_collect(OpValue::ExtractW(self.0)))
                }
            }

            impl From<$scalar> for $type {
                fn from(x: $scalar) -> Self {
                    Self(Graph::push_collect(OpValue::Splat4(x.0)))
                }
            }

            impl From<$elem> for $type {
                fn from(x: $elem) -> Self {
                    Self::from(<$scalar>::from(x))
                }
            }

            impl<X: Into<$scalar>, Y: Into<$scalar>, Z: Into<$scalar>, W: Into<$scalar>> From<(X, Y, Z, W)> for $type {
                fn from((x, y, z, w): (X, Y, Z, W)) -> Self {
                    Self(Graph::push_collect(OpValue::Vec4(
                        Into::<$scalar>::into(x).0,
                        Into::<$scalar>::into(y).0,
                        Into::<$scalar>::into(z).0,
                        Into::<$scalar>::into(w).0,
                    )))
                }
            }
        };
    }

    macro_rules! impl_bit_binop {
        ($type:ty, $elem:ty) => {
            impl BitAnd<$elem> for $type {
                type Output = $type;
                fn bitand(self, rhs: $elem) -> Self::Output {
                    Self(Graph::push_collect(OpValue::And(self.0, Self::from(rhs).0)))
                }
            }

            impl BitOr<$elem> for $type {
                type Output = $type;
                fn bitor(self, rhs: $elem) -> Self::Output {
                    Self(Graph::push_collect(OpValue::Or(self.0, Self::from(rhs).0)))
                }
            }

            impl BitXor<$elem> for $type {
                type Output = $type;
                fn bitxor(self, rhs: $elem) -> Self::Output {
                    Self(Graph::push_collect(OpValue::Xor(self.0, Self::from(rhs).0)))
                }
            }
        };
    }

    macro_rules! impl_bit_shift {
        ($type:ty, $elem:ty) => {
            impl Shl<$elem> for $type {
                type Output = $type;
                fn shl(self, rhs: $elem) -> Self::Output {
                    Self(Graph::push_collect(OpValue::Shl(self.0, Self::from(rhs).0)))
                }
            }

            impl Shr<$elem> for $type {
                type Output = $type;
                fn shr(self, rhs: $elem) -> Self::Output {
                    Self(Graph::push_collect(OpValue::Shr(self.0, Self::from(rhs).0)))
                }
            }
        };
    }

    macro_rules! impl_bit_base {
        ($type:ty) => {
            impl BitAnd<$type> for $type {
                type Output = $type;
                fn bitand(self, rhs: Self) -> Self::Output {
                    Self(Graph::push_collect(OpValue::And(self.0, Self::from(rhs).0)))
                }
            }

            impl BitOr<$type> for $type {
                type Output = $type;
                fn bitor(self, rhs: Self) -> Self::Output {
                    Self(Graph::push_collect(OpValue::Or(self.0, Self::from(rhs).0)))
                }
            }

            impl BitXor<$type> for $type {
                type Output = $type;
                fn bitxor(self, rhs: Self) -> Self::Output {
                    Self(Graph::push_collect(OpValue::Xor(self.0, Self::from(rhs).0)))
                }
            }

            impl Not for $type {
                type Output = $type;
                fn not(self) -> Self::Output {
                    Self(Graph::push_collect(OpValue::Not(self.0)))
                }
            }
        };
    }

    macro_rules! impl_bit_vec {
        ($type:ty, $elem:ty, $scalar:ty, 1) => {
            impl_bit_base!($type);
            impl_bit_binop!($type, $elem);
        };

        ($type:ty, $elem:ty, $scalar:ty, $n:literal) => {
            impl_bit_base!($type);
            impl_bit_binop!($type, $elem);
            impl_bit_binop!($type, $scalar);
        };
    }

    impl_float!(float1, int1);
    impl_float!(float2, int2);
    impl_float!(float3, int3);
    impl_float!(float4, int4);

    impl_num_vec!(float1, f32, float1, 1);
    impl_num_vec!(float2, f32, float1, 2);
    impl_num_vec!(float3, f32, float1, 3);
    impl_num_vec!(float4, f32, float1, 4);

    impl_num_vec!(int1, i32, int1, 1);
    impl_num_vec!(int2, i32, int1, 2);
    impl_num_vec!(int3, i32, int1, 3);
    impl_num_vec!(int4, i32, int1, 4);

    impl_bit_vec!(boolean, bool, boolean, 1);
    impl_bit_vec!(int1, i32, int1, 1);
    impl_bit_vec!(int2, i32, int1, 2);
    impl_bit_vec!(int3, i32, int1, 3);
    impl_bit_vec!(int4, i32, int1, 4);

    impl_bit_shift!(int1, int1);
    impl_bit_shift!(int2, int1);
    impl_bit_shift!(int3, int1);
    impl_bit_shift!(int4, int1);
    impl_bit_shift!(int1, i32);
    impl_bit_shift!(int2, i32);
    impl_bit_shift!(int3, i32);
    impl_bit_shift!(int4, i32);

    impl float3 {
        pub fn cross(self, rhs: impl Into<Self>) -> Self {
            Self(Graph::push_collect(OpValue::Cross(self.0, rhs.into().0)))
        }
    }

    impl boolean {
        pub fn select<T: Select>(self, x: T, y: impl Into<T>) -> T {
            T::select(x, y.into(), self)
        }

        pub fn eq(self, other: boolean) -> boolean {
            !(self ^ other)
        }

        pub fn ne(self, other: boolean) -> boolean {
            self ^ other
        }
    }

    impl texture {
        pub fn size(&self) -> int2 {
            int2(Graph::push_collect(OpValue::TextureSize(self.0)))
        }

        pub fn sample(&self, pos: impl Into<float2>, filter: TextureFilter) -> float4 {
            float4(Graph::push_collect(OpValue::TextureSample(
                self.0,
                pos.into().0,
                filter,
            )))
        }
    }

    impl From<f32> for float1 {
        fn from(value: f32) -> Self {
            Self(Graph::push_collect(OpValue::Literal(OpLiteral::Float(value))))
        }
    }

    impl From<i32> for float1 {
        fn from(value: i32) -> Self {
            Self(Graph::push_collect(OpValue::Literal(OpLiteral::Float(value as _))))
        }
    }

    impl From<i32> for int1 {
        fn from(value: i32) -> Self {
            Self(Graph::push_collect(OpValue::Literal(OpLiteral::Int(value))))
        }
    }

    impl From<bool> for boolean {
        fn from(value: bool) -> Self {
            Self(Graph::push_collect(OpValue::Literal(OpLiteral::Bool(value))))
        }
    }

    impl Select for boolean {
        fn select(x: Self, y: Self, switch: boolean) -> Self {
            y ^ ((x ^ y) & switch)
        }
    }
}

macro_rules! impl_constructor {
    ($($type:ident),*) => {
        $(
            pub fn $type(x: impl Into<$type>) -> $type {
                x.into()
            }
        )*
    };
}

impl_constructor!(float1, float2, float3, float4, int1, int2, int3, int4, boolean);
