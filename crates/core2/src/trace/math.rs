#![allow(non_camel_case_types)]

mod int {
    use crate::{
        ShaderOp,
        trace::{Lerp, boolean, float1, trace_emit},
    };
    use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Neg, Not, Rem, Shl, Shr, Sub};

    #[derive(Debug, Clone, Copy)]
    pub struct int1(pub(crate) u32);

    impl Add for int1 {
        type Output = Self;

        fn add(self, rhs: Self) -> Self::Output {
            Self(trace_emit(ShaderOp::IAdd(self.0, rhs.0)))
        }
    }

    impl Sub for int1 {
        type Output = Self;

        fn sub(self, rhs: Self) -> Self::Output {
            Self(trace_emit(ShaderOp::ISub(self.0, rhs.0)))
        }
    }

    impl Mul for int1 {
        type Output = Self;

        fn mul(self, rhs: Self) -> Self::Output {
            Self(trace_emit(ShaderOp::IMul(self.0, rhs.0)))
        }
    }

    impl Div for int1 {
        type Output = Self;

        fn div(self, rhs: Self) -> Self::Output {
            Self(trace_emit(ShaderOp::IDiv(self.0, rhs.0)))
        }
    }

    impl Rem for int1 {
        type Output = Self;

        fn rem(self, rhs: Self) -> Self::Output {
            Self(trace_emit(ShaderOp::IMod(self.0, rhs.0)))
        }
    }

    impl Neg for int1 {
        type Output = Self;

        fn neg(self) -> Self::Output {
            Self(trace_emit(ShaderOp::INeg(self.0)))
        }
    }

    impl BitAnd for int1 {
        type Output = Self;

        fn bitand(self, rhs: Self) -> Self::Output {
            Self(trace_emit(ShaderOp::IAnd(self.0, rhs.0)))
        }
    }

    impl BitOr for int1 {
        type Output = Self;

        fn bitor(self, rhs: Self) -> Self::Output {
            Self(trace_emit(ShaderOp::IOr(self.0, rhs.0)))
        }
    }

    impl BitXor for int1 {
        type Output = Self;

        fn bitxor(self, rhs: Self) -> Self::Output {
            Self(trace_emit(ShaderOp::IXor(self.0, rhs.0)))
        }
    }

    impl Shl for int1 {
        type Output = Self;

        fn shl(self, rhs: Self) -> Self::Output {
            Self(trace_emit(ShaderOp::IShl(self.0, rhs.0)))
        }
    }

    impl Shr for int1 {
        type Output = Self;

        fn shr(self, rhs: Self) -> Self::Output {
            Self(trace_emit(ShaderOp::IShr(self.0, rhs.0)))
        }
    }

    impl Not for int1 {
        type Output = Self;

        fn not(self) -> Self::Output {
            Self(trace_emit(ShaderOp::INot(self.0)))
        }
    }

    impl Lerp<boolean> for int1 {
        fn lerp(sel: boolean, start: Self, end: Self) -> Self {
            Self(trace_emit(ShaderOp::ISelect(sel.0, start.0, end.0)))
        }
    }

    impl From<i32> for int1 {
        fn from(value: i32) -> Self {
            Self(trace_emit(ShaderOp::ILit(value)))
        }
    }

    impl From<float1> for int1 {
        fn from(value: float1) -> Self {
            Self(trace_emit(ShaderOp::ICastFloat(value.0)))
        }
    }

    impl int1 {
        pub fn abs(self) -> int1 {
            Self(trace_emit(ShaderOp::IAbs(self.0)))
        }

        pub fn min(self, rhs: impl Into<Self>) -> Self {
            Self(trace_emit(ShaderOp::IMin(self.0, rhs.into().0)))
        }

        pub fn max(self, rhs: impl Into<Self>) -> Self {
            Self(trace_emit(ShaderOp::IMax(self.0, rhs.into().0)))
        }

        pub fn clamp(self, min: impl Into<Self>, max: impl Into<Self>) -> Self {
            self.min(max).max(min)
        }
    }
}

mod float {
    use crate::{
        ShaderOp,
        trace::{Lerp, boolean, int1, trace_emit},
    };
    use std::ops::{Add, Div, Mul, Neg, Rem, Sub};

    #[derive(Debug, Clone, Copy)]
    pub struct float1(pub(crate) u32);

    impl Add for float1 {
        type Output = Self;

        fn add(self, rhs: Self) -> Self::Output {
            Self(trace_emit(ShaderOp::FAdd(self.0, rhs.0)))
        }
    }

    impl Sub for float1 {
        type Output = Self;

        fn sub(self, rhs: Self) -> Self::Output {
            Self(trace_emit(ShaderOp::FSub(self.0, rhs.0)))
        }
    }

    impl Mul for float1 {
        type Output = Self;

        fn mul(self, rhs: Self) -> Self::Output {
            Self(trace_emit(ShaderOp::FMul(self.0, rhs.0)))
        }
    }

    impl Div for float1 {
        type Output = Self;

        fn div(self, rhs: Self) -> Self::Output {
            Self(trace_emit(ShaderOp::FDiv(self.0, rhs.0)))
        }
    }

    impl Rem for float1 {
        type Output = Self;

        fn rem(self, rhs: Self) -> Self::Output {
            Self(trace_emit(ShaderOp::FMod(self.0, rhs.0)))
        }
    }

    impl Neg for float1 {
        type Output = Self;

        fn neg(self) -> Self::Output {
            Self(trace_emit(ShaderOp::FNeg(self.0)))
        }
    }

    impl From<i32> for float1 {
        fn from(value: i32) -> Self {
            Self(trace_emit(ShaderOp::FLit(value as f32)))
        }
    }

    impl From<f32> for float1 {
        fn from(value: f32) -> Self {
            Self(trace_emit(ShaderOp::FLit(value)))
        }
    }

    impl From<int1> for float1 {
        fn from(value: int1) -> Self {
            Self(trace_emit(ShaderOp::FCastInt32(value.0)))
        }
    }

    impl Lerp<float1> for float1 {
        fn lerp(sel: Self, start: Self, end: Self) -> Self {
            Self(trace_emit(ShaderOp::Lerp(sel.0, start.0, end.0)))
        }
    }

    impl Lerp<boolean> for float1 {
        fn lerp(sel: boolean, start: Self, end: Self) -> Self {
            Self(trace_emit(ShaderOp::FSelect(sel.0, start.0, end.0)))
        }
    }

    impl float1 {
        pub fn sin(self) -> Self {
            Self(trace_emit(ShaderOp::Sin(self.0)))
        }

        pub fn cos(self) -> Self {
            Self(trace_emit(ShaderOp::Cos(self.0)))
        }

        pub fn tan(self) -> Self {
            Self(trace_emit(ShaderOp::Tan(self.0)))
        }

        pub fn asin(self) -> Self {
            Self(trace_emit(ShaderOp::Asin(self.0)))
        }

        pub fn acos(self) -> Self {
            Self(trace_emit(ShaderOp::Acos(self.0)))
        }

        pub fn atan(self) -> Self {
            Self(trace_emit(ShaderOp::Atan(self.0)))
        }

        pub fn sqrt(self) -> Self {
            Self(trace_emit(ShaderOp::Sqrt(self.0)))
        }

        pub fn exp(self) -> Self {
            Self(trace_emit(ShaderOp::Exp(self.0)))
        }

        pub fn ln(self) -> Self {
            Self(trace_emit(ShaderOp::Ln(self.0)))
        }

        pub fn floor(self) -> Self {
            Self(trace_emit(ShaderOp::Floor(self.0)))
        }

        pub fn dx(self) -> Self {
            Self(trace_emit(ShaderOp::DerivX(self.0)))
        }

        pub fn dy(self) -> Self {
            Self(trace_emit(ShaderOp::DerivY(self.0)))
        }

        pub fn abs(self) -> Self {
            Self(trace_emit(ShaderOp::FAbs(self.0)))
        }

        pub fn min(self, rhs: impl Into<Self>) -> Self {
            Self(trace_emit(ShaderOp::FMin(self.0, rhs.into().0)))
        }

        pub fn max(self, rhs: impl Into<Self>) -> Self {
            Self(trace_emit(ShaderOp::FMax(self.0, rhs.into().0)))
        }

        pub fn clamp(self, min: impl Into<Self>, max: impl Into<Self>) -> Self {
            self.min(max).max(min)
        }

        pub fn atan2(self, rhs: impl Into<Self>) -> Self {
            Self(trace_emit(ShaderOp::Atan2(self.0, rhs.into().0)))
        }

        pub fn powf(self, rhs: impl Into<Self>) -> Self {
            Self(trace_emit(ShaderOp::PowFloat(self.0, rhs.into().0)))
        }

        pub fn powi(self, rhs: impl Into<int1>) -> Self {
            Self(trace_emit(ShaderOp::PowInt32(self.0, rhs.into().0)))
        }

        pub fn lerp<T: Lerp<Self>>(self, start: T, end: impl Into<T>) -> T {
            T::lerp(self, start, end.into())
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub struct float2 {
        pub x: float1,
        pub y: float1,
    }

    #[derive(Debug, Clone, Copy)]
    pub struct float3 {
        pub x: float1,
        pub y: float1,
        pub z: float1,
    }

    #[derive(Debug, Clone, Copy)]
    pub struct float4 {
        pub x: float1,
        pub y: float1,
        pub z: float1,
        pub w: float1,
    }

    impl<X: Into<float1>, Y: Into<float1>> From<(X, Y)> for float2 {
        fn from((x, y): (X, Y)) -> Self {
            Self {
                x: x.into(),
                y: y.into(),
            }
        }
    }

    impl<X: Into<float1>, Y: Into<float1>, Z: Into<float1>> From<(X, Y, Z)> for float3 {
        fn from((x, y, z): (X, Y, Z)) -> Self {
            Self {
                x: x.into(),
                y: y.into(),
                z: z.into(),
            }
        }
    }

    impl<X: Into<float1>, Y: Into<float1>, Z: Into<float1>, W: Into<float1>> From<(X, Y, Z, W)> for float4 {
        fn from((x, y, z, w): (X, Y, Z, W)) -> Self {
            Self {
                x: x.into(),
                y: y.into(),
                z: z.into(),
                w: w.into(),
            }
        }
    }
}

mod bool {
    use crate::{
        ShaderOp,
        trace::{Lerp, float1, int1, trace_emit},
    };
    use std::ops::{BitAnd, BitOr, BitXor, Not};

    #[derive(Debug, Clone, Copy)]
    pub struct boolean(pub(crate) u32);

    impl BitAnd for boolean {
        type Output = Self;

        fn bitand(self, rhs: Self) -> Self::Output {
            Self(trace_emit(ShaderOp::BAnd(self.0, rhs.0)))
        }
    }

    impl BitOr for boolean {
        type Output = Self;

        fn bitor(self, rhs: Self) -> Self::Output {
            Self(trace_emit(ShaderOp::BOr(self.0, rhs.0)))
        }
    }

    impl BitXor for boolean {
        type Output = Self;

        fn bitxor(self, rhs: Self) -> Self::Output {
            Self(trace_emit(ShaderOp::BXor(self.0, rhs.0)))
        }
    }

    impl Not for boolean {
        type Output = Self;

        fn not(self) -> Self::Output {
            Self(trace_emit(ShaderOp::BNot(self.0)))
        }
    }

    impl From<bool> for boolean {
        fn from(value: bool) -> Self {
            Self(trace_emit(ShaderOp::BLit(value)))
        }
    }

    impl Lerp<boolean> for boolean {
        fn lerp(switch: boolean, x: Self, y: Self) -> Self {
            y ^ ((x ^ y) & switch)
        }
    }

    impl boolean {
        pub fn lerp<T: Lerp<Self>>(self, start: T, end: impl Into<T>) -> T {
            T::lerp(self, start, end.into())
        }
    }

    impl float1 {
        pub fn eq(self, other: impl Into<Self>) -> boolean {
            boolean(trace_emit(ShaderOp::EqFloat(self.0, other.into().0)))
        }

        pub fn ne(self, other: impl Into<Self>) -> boolean {
            boolean(trace_emit(ShaderOp::NeFloat(self.0, other.into().0)))
        }

        pub fn le(self, other: impl Into<Self>) -> boolean {
            boolean(trace_emit(ShaderOp::LeFloat(self.0, other.into().0)))
        }

        pub fn lt(self, other: impl Into<Self>) -> boolean {
            boolean(trace_emit(ShaderOp::LtFloat(self.0, other.into().0)))
        }

        pub fn ge(self, other: impl Into<Self>) -> boolean {
            boolean(trace_emit(ShaderOp::GeFloat(self.0, other.into().0)))
        }

        pub fn gt(self, other: impl Into<Self>) -> boolean {
            boolean(trace_emit(ShaderOp::GtFloat(self.0, other.into().0)))
        }
    }

    impl int1 {
        pub fn eq(self, other: impl Into<Self>) -> boolean {
            boolean(trace_emit(ShaderOp::EqInt32(self.0, other.into().0)))
        }

        pub fn ne(self, other: impl Into<Self>) -> boolean {
            boolean(trace_emit(ShaderOp::NeInt32(self.0, other.into().0)))
        }

        pub fn le(self, other: impl Into<Self>) -> boolean {
            boolean(trace_emit(ShaderOp::LeInt32(self.0, other.into().0)))
        }

        pub fn lt(self, other: impl Into<Self>) -> boolean {
            boolean(trace_emit(ShaderOp::LtInt32(self.0, other.into().0)))
        }

        pub fn ge(self, other: impl Into<Self>) -> boolean {
            boolean(trace_emit(ShaderOp::GeInt32(self.0, other.into().0)))
        }

        pub fn gt(self, other: impl Into<Self>) -> boolean {
            boolean(trace_emit(ShaderOp::GtInt32(self.0, other.into().0)))
        }
    }
}

mod util {
    use crate::trace::{boolean, int1};

    pub trait Lerp<T> {
        #[doc(hidden)]
        fn lerp(sel: T, start: Self, end: Self) -> Self;
    }

    pub trait ControlFlow {}

    pub fn branch<T: ControlFlow>(
        cond: boolean,
        true_branch: impl FnOnce() -> T,
        false_branch: impl FnOnce() -> T,
    ) -> T {
        todo!()
    }

    pub fn while_loop<T: ControlFlow>(init: T, cond: impl FnOnce(T) -> boolean, body: impl FnOnce(T) -> T) -> T {
        todo!()
    }

    pub fn for_loop<T: ControlFlow>(init: T, iters: int1, body: impl FnOnce(int1, T) -> T) -> T {
        todo!()
    }
}

pub use bool::*;
pub use float::*;
pub use int::*;
pub use util::*;

macro_rules! impl_constructor {
    ($($type:ident),*) => {
        $(
            pub fn $type(x: impl Into<$type>) -> $type {
                x.into()
            }
        )*
    };
}

impl_constructor!(float1, float2, float3, float4, int1, boolean);
