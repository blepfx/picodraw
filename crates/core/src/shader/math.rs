#![allow(non_camel_case_types)]

pub use types::*;
pub mod types {
    use crate::graph::{Graph, Op, OpAddr, Swizzle};
    use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Neg, Not, Rem, Sub};

    #[doc(hidden)]
    pub trait Select {
        fn select(x: Self, y: Self, switch: boolean) -> Self;
    }

    #[doc(hidden)]
    pub trait Type {
        fn into_addr(self) -> OpAddr;
        fn from_addr(addr: OpAddr) -> Self;
    }

    #[derive(Clone, Copy)]
    pub struct float1(pub(crate) OpAddr);
    #[derive(Clone, Copy)]
    pub struct float2(pub(crate) OpAddr);
    #[derive(Clone, Copy)]
    pub struct float3(pub(crate) OpAddr);
    #[derive(Clone, Copy)]
    pub struct float4(pub(crate) OpAddr);

    #[derive(Clone, Copy)]
    pub struct int1(pub(crate) OpAddr);
    #[derive(Clone, Copy)]
    pub struct int2(pub(crate) OpAddr);
    #[derive(Clone, Copy)]
    pub struct int3(pub(crate) OpAddr);
    #[derive(Clone, Copy)]
    pub struct int4(pub(crate) OpAddr);

    #[derive(Clone, Copy)]
    pub struct boolean(pub(crate) OpAddr);

    #[derive(Clone, Copy)]
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
                    Self(Graph::push_collect(Op::Sin(self.0)))
                }

                pub fn cos(self) -> Self {
                    Self(Graph::push_collect(Op::Cos(self.0)))
                }

                pub fn tan(self) -> Self {
                    Self(Graph::push_collect(Op::Tan(self.0)))
                }

                pub fn asin(self) -> Self {
                    Self(Graph::push_collect(Op::Asin(self.0)))
                }

                pub fn acos(self) -> Self {
                    Self(Graph::push_collect(Op::Acos(self.0)))
                }

                pub fn atan(self) -> Self {
                    Self(Graph::push_collect(Op::Atan(self.0)))
                }

                pub fn sqrt(self) -> Self {
                    Self(Graph::push_collect(Op::Sqrt(self.0)))
                }

                pub fn exp(self) -> Self {
                    Self(Graph::push_collect(Op::Exp(self.0)))
                }

                pub fn ln(self) -> Self {
                    Self(Graph::push_collect(Op::Ln(self.0)))
                }

                pub fn floor(self) -> Self {
                    Self(Graph::push_collect(Op::Floor(self.0)))
                }

                pub fn dx(self) -> Self {
                    Self(Graph::push_collect(Op::DerivX(self.0)))
                }

                pub fn dy(self) -> Self {
                    Self(Graph::push_collect(Op::DerivY(self.0)))
                }

                pub fn fwidth(self) -> Self {
                    Self(Graph::push_collect(Op::DerivWidth(self.0)))
                }

                pub fn abs(self) -> Self {
                    Self(Graph::push_collect(Op::Abs(self.0)))
                }

                pub fn sign(self) -> Self {
                    Self(Graph::push_collect(Op::Sign(self.0)))
                }

                pub fn dot(self, rhs: impl Into<Self>) -> float1 {
                    float1(Graph::push_collect(Op::Dot(self.0, rhs.into().0)))
                }

                pub fn min(self, rhs: impl Into<Self>) -> Self {
                    Self(Graph::push_collect(Op::Min(self.0, rhs.into().0)))
                }

                pub fn max(self, rhs: impl Into<Self>) -> Self {
                    Self(Graph::push_collect(Op::Max(self.0, rhs.into().0)))
                }

                pub fn atan2(self, rhs: impl Into<Self>) -> Self {
                    Self(Graph::push_collect(Op::Atan2(self.0, rhs.into().0)))
                }

                pub fn pow(self, rhs: impl Into<Self>) -> Self {
                    Self(Graph::push_collect(Op::Pow(self.0, rhs.into().0)))
                }

                pub fn step(self, min: impl Into<Self>) -> Self {
                    Self(Graph::push_collect(Op::Step(self.0, min.into().0)))
                }

                pub fn len(self) -> float1 {
                    float1(Graph::push_collect(Op::Length(self.0)))
                }

                pub fn norm(self) -> Self {
                    Self(Graph::push_collect(Op::Normalize(self.0)))
                }

                pub fn clamp(self, min: impl Into<Self>, max: impl Into<Self>) -> Self {
                    Self(Graph::push_collect(Op::Clamp(self.0, min.into().0, max.into().0)))
                }

                pub fn lerp(self, min: impl Into<Self>, max: impl Into<Self>) -> Self {
                    Self(Graph::push_collect(Op::Lerp(self.0, min.into().0, max.into().0)))
                }

                pub fn smoothstep(self, min: impl Into<Self>, max: impl Into<Self>) -> Self {
                    Self(Graph::push_collect(Op::Smoothstep(
                        self.0,
                        min.into().0,
                        max.into().0,
                    )))
                }
            }

            impl From<$int> for $type {
                fn from(x: $int) -> Self {
                    Self(Graph::push_collect(Op::CastFloat(x.0)))
                }
            }

            impl From<$type> for $int {
                fn from(x: $type) -> Self {
                    Self(Graph::push_collect(Op::CastInt(x.0)))
                }
            }
        };
    }

    macro_rules! impl_boolean {
        ($type:ty) => {
            impl BitAnd for $type {
                type Output = $type;
                fn bitand(self, rhs: Self) -> Self::Output {
                    Self(Graph::push_collect(Op::And(self.0, rhs.0)))
                }
            }

            impl BitOr for $type {
                type Output = $type;
                fn bitor(self, rhs: Self) -> Self::Output {
                    Self(Graph::push_collect(Op::Or(self.0, rhs.0)))
                }
            }

            impl BitXor for $type {
                type Output = $type;
                fn bitxor(self, rhs: Self) -> Self::Output {
                    Self(Graph::push_collect(Op::Xor(self.0, rhs.0)))
                }
            }

            impl Not for $type {
                type Output = $type;
                fn not(self) -> Self::Output {
                    Self(Graph::push_collect(Op::Not(self.0)))
                }
            }
        };
    }

    macro_rules! impl_num_base {
        ($type:ty, $elem:ty) => {
            impl Select for $type {
                fn select(x: Self, y: Self, switch: boolean) -> Self {
                    Self(Graph::push_collect(Op::Select(switch.0, x.0, y.0)))
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
                    Self(Graph::push_collect(Op::Add(self.0, rhs.0)))
                }
            }

            impl Sub<$type> for $type {
                type Output = $type;
                fn sub(self, rhs: $type) -> Self::Output {
                    Self(Graph::push_collect(Op::Sub(self.0, rhs.0)))
                }
            }

            impl Mul<$type> for $type {
                type Output = $type;
                fn mul(self, rhs: $type) -> Self::Output {
                    Self(Graph::push_collect(Op::Mul(self.0, rhs.0)))
                }
            }

            impl Div<$type> for $type {
                type Output = $type;
                fn div(self, rhs: $type) -> Self::Output {
                    Self(Graph::push_collect(Op::Div(self.0, rhs.0)))
                }
            }

            impl Rem<$type> for $type {
                type Output = $type;
                fn rem(self, rhs: $type) -> Self::Output {
                    Self(Graph::push_collect(Op::Rem(self.0, rhs.0)))
                }
            }

            impl Neg for $type {
                type Output = $type;
                fn neg(self) -> Self::Output {
                    Self(Graph::push_collect(Op::Neg(self.0)))
                }
            }

            impl $type {
                pub fn eq(self, other: impl Into<Self>) -> boolean {
                    boolean(Graph::push_collect(Op::Eq(self.0, other.into().0)))
                }

                pub fn ne(self, other: impl Into<Self>) -> boolean {
                    boolean(Graph::push_collect(Op::Ne(self.0, other.into().0)))
                }

                pub fn le(self, other: impl Into<Self>) -> boolean {
                    boolean(Graph::push_collect(Op::Le(self.0, other.into().0)))
                }

                pub fn ge(self, other: impl Into<Self>) -> boolean {
                    boolean(Graph::push_collect(Op::Ge(self.0, other.into().0)))
                }

                pub fn lt(self, other: impl Into<Self>) -> boolean {
                    boolean(Graph::push_collect(Op::Lt(self.0, other.into().0)))
                }

                pub fn gt(self, other: impl Into<Self>) -> boolean {
                    boolean(Graph::push_collect(Op::Gt(self.0, other.into().0)))
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
                    $scalar(Graph::push_collect(Op::Swizzle1(self.0, [Swizzle::X])))
                }

                pub fn y(self) -> $scalar {
                    $scalar(Graph::push_collect(Op::Swizzle1(self.0, [Swizzle::Y])))
                }
            }

            impl From<$scalar> for $type {
                fn from(x: $scalar) -> Self {
                    Self(Graph::push_collect(Op::Splat2(x.0)))
                }
            }

            impl From<$elem> for $type {
                fn from(x: $elem) -> Self {
                    Self::from(<$scalar>::from(x))
                }
            }

            impl<X: Into<$scalar>, Y: Into<$scalar>> From<(X, Y)> for $type {
                fn from((x, y): (X, Y)) -> Self {
                    Self(Graph::push_collect(Op::Vec2(
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
                    $scalar(Graph::push_collect(Op::Swizzle1(self.0, [Swizzle::X])))
                }

                pub fn y(self) -> $scalar {
                    $scalar(Graph::push_collect(Op::Swizzle1(self.0, [Swizzle::Y])))
                }

                pub fn z(self) -> $scalar {
                    $scalar(Graph::push_collect(Op::Swizzle1(self.0, [Swizzle::Z])))
                }
            }

            impl From<$scalar> for $type {
                fn from(x: $scalar) -> Self {
                    Self(Graph::push_collect(Op::Splat3(x.0)))
                }
            }

            impl From<$elem> for $type {
                fn from(x: $elem) -> Self {
                    Self::from(<$scalar>::from(x))
                }
            }

            impl<X: Into<$scalar>, Y: Into<$scalar>, Z: Into<$scalar>> From<(X, Y, Z)> for $type {
                fn from((x, y, z): (X, Y, Z)) -> Self {
                    Self(Graph::push_collect(Op::Vec3(
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
                    $scalar(Graph::push_collect(Op::Swizzle1(self.0, [Swizzle::X])))
                }

                pub fn y(self) -> $scalar {
                    $scalar(Graph::push_collect(Op::Swizzle1(self.0, [Swizzle::Y])))
                }

                pub fn z(self) -> $scalar {
                    $scalar(Graph::push_collect(Op::Swizzle1(self.0, [Swizzle::Z])))
                }

                pub fn w(self) -> $scalar {
                    $scalar(Graph::push_collect(Op::Swizzle1(self.0, [Swizzle::W])))
                }
            }

            impl From<$scalar> for $type {
                fn from(x: $scalar) -> Self {
                    Self(Graph::push_collect(Op::Splat4(x.0)))
                }
            }

            impl From<$elem> for $type {
                fn from(x: $elem) -> Self {
                    Self::from(<$scalar>::from(x))
                }
            }

            impl<X: Into<$scalar>, Y: Into<$scalar>, Z: Into<$scalar>, W: Into<$scalar>> From<(X, Y, Z, W)> for $type {
                fn from((x, y, z, w): (X, Y, Z, W)) -> Self {
                    Self(Graph::push_collect(Op::Vec4(
                        Into::<$scalar>::into(x).0,
                        Into::<$scalar>::into(y).0,
                        Into::<$scalar>::into(z).0,
                        Into::<$scalar>::into(w).0,
                    )))
                }
            }
        };
    }

    impl_float!(float1, int1);
    impl_float!(float2, int2);
    impl_float!(float3, int3);
    impl_float!(float4, int4);

    impl_boolean!(boolean);

    impl_num_vec!(float1, f32, float1, 1);
    impl_num_vec!(float2, f32, float1, 2);
    impl_num_vec!(float3, f32, float1, 3);
    impl_num_vec!(float4, f32, float1, 4);

    impl_num_vec!(int1, i32, int1, 1);
    impl_num_vec!(int2, i32, int1, 2);
    impl_num_vec!(int3, i32, int1, 3);
    impl_num_vec!(int4, i32, int1, 4);

    impl float3 {
        pub fn cross(self, rhs: Self) -> Self {
            Self(Graph::push_collect(Op::Cross(self.0, rhs.0)))
        }
    }

    impl boolean {
        pub fn select<T: Select>(self, x: T, y: impl Into<T>) -> T {
            T::select(x, y.into(), self)
        }
    }

    impl texture {
        pub fn size(&self) -> int2 {
            int2(Graph::push_collect(Op::TextureSize(self.0)))
        }

        pub fn sample_linear(&self, pos: impl Into<float2>) -> float4 {
            float4(Graph::push_collect(Op::TextureLinear(self.0, pos.into().0)))
        }

        pub fn sample_nearest(&self, pos: impl Into<float2>) -> float4 {
            float4(Graph::push_collect(Op::TextureNearest(self.0, pos.into().0)))
        }
    }

    impl From<f32> for float1 {
        fn from(value: f32) -> Self {
            Self(Graph::push_collect(Op::LiteralFloat(value.into())))
        }
    }

    impl From<i32> for float1 {
        fn from(value: i32) -> Self {
            Self(Graph::push_collect(Op::LiteralFloat((value as f32).into())))
        }
    }

    impl From<i32> for int1 {
        fn from(value: i32) -> Self {
            Self(Graph::push_collect(Op::LiteralInt(value)))
        }
    }

    impl From<bool> for boolean {
        fn from(value: bool) -> Self {
            Self(Graph::push_collect(Op::LiteralBool(value)))
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
