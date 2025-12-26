#![allow(non_camel_case_types)]

mod int {
    use crate::{
        ShaderOp,
        trace::{Lerp, TraceGraph, boolean, float1},
    };
    use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Neg, Not, Shl, Shr, Sub};

    #[derive(Debug, Clone, Copy)]
    pub struct int1(pub(crate) u32);

    impl Add for int1 {
        type Output = Self;

        fn add(self, rhs: Self) -> Self::Output {
            Self(TraceGraph::emit(ShaderOp::IAdd(self.0, rhs.0)))
        }
    }

    impl Sub for int1 {
        type Output = Self;

        fn sub(self, rhs: Self) -> Self::Output {
            Self(TraceGraph::emit(ShaderOp::ISub(self.0, rhs.0)))
        }
    }

    impl Mul for int1 {
        type Output = Self;

        fn mul(self, rhs: Self) -> Self::Output {
            Self(TraceGraph::emit(ShaderOp::IMul(self.0, rhs.0)))
        }
    }

    impl Div for int1 {
        type Output = Self;

        fn div(self, rhs: Self) -> Self::Output {
            Self(TraceGraph::emit(ShaderOp::IDiv(self.0, rhs.0)))
        }
    }

    impl Neg for int1 {
        type Output = Self;

        fn neg(self) -> Self::Output {
            Self(TraceGraph::emit(ShaderOp::INeg(self.0)))
        }
    }

    impl BitAnd for int1 {
        type Output = Self;

        fn bitand(self, rhs: Self) -> Self::Output {
            Self(TraceGraph::emit(ShaderOp::IAnd(self.0, rhs.0)))
        }
    }

    impl BitOr for int1 {
        type Output = Self;

        fn bitor(self, rhs: Self) -> Self::Output {
            Self(TraceGraph::emit(ShaderOp::IOr(self.0, rhs.0)))
        }
    }

    impl BitXor for int1 {
        type Output = Self;

        fn bitxor(self, rhs: Self) -> Self::Output {
            Self(TraceGraph::emit(ShaderOp::IXor(self.0, rhs.0)))
        }
    }

    impl Shl for int1 {
        type Output = Self;

        fn shl(self, rhs: Self) -> Self::Output {
            Self(TraceGraph::emit(ShaderOp::IShl(self.0, rhs.0)))
        }
    }

    impl Shr for int1 {
        type Output = Self;

        fn shr(self, rhs: Self) -> Self::Output {
            Self(TraceGraph::emit(ShaderOp::IShr(self.0, rhs.0)))
        }
    }

    impl Not for int1 {
        type Output = Self;

        fn not(self) -> Self::Output {
            Self(TraceGraph::emit(ShaderOp::INot(self.0)))
        }
    }

    impl Lerp<boolean> for int1 {
        fn lerp(sel: boolean, start: Self, end: Self) -> Self {
            Self(TraceGraph::emit(ShaderOp::ISelect(sel.0, start.0, end.0)))
        }
    }

    impl From<i32> for int1 {
        fn from(value: i32) -> Self {
            Self(TraceGraph::emit(ShaderOp::ILit(value)))
        }
    }

    impl From<float1> for int1 {
        fn from(value: float1) -> Self {
            Self(TraceGraph::emit(ShaderOp::ICastFloat(value.0)))
        }
    }

    impl int1 {
        pub fn read_u8(offset: u32) -> Self {
            Self(TraceGraph::emit(ShaderOp::ReadU8(offset)))
        }

        pub fn read_u16(offset: u32) -> Self {
            assert!(offset % 2 == 0, "u16 reads must be 2-byte aligned");
            Self(TraceGraph::emit(ShaderOp::ReadU16(offset)))
        }

        pub fn read_i32(offset: u32) -> Self {
            assert!(offset % 4 == 0, "i32 reads must be 4-byte aligned");
            Self(TraceGraph::emit(ShaderOp::ReadI32(offset)))
        }

        pub fn rem_euclid(self, rhs: impl Into<Self>) -> Self {
            Self(TraceGraph::emit(ShaderOp::IMod(self.0, rhs.into().0)))
        }

        pub fn abs(self) -> int1 {
            Self(TraceGraph::emit(ShaderOp::IAbs(self.0)))
        }

        pub fn min(self, rhs: impl Into<Self>) -> Self {
            Self(TraceGraph::emit(ShaderOp::IMin(self.0, rhs.into().0)))
        }

        pub fn max(self, rhs: impl Into<Self>) -> Self {
            Self(TraceGraph::emit(ShaderOp::IMax(self.0, rhs.into().0)))
        }

        pub fn clamp(self, min: impl Into<Self>, max: impl Into<Self>) -> Self {
            self.min(max).max(min)
        }
    }
}

mod float {
    use crate::{
        ShaderOp,
        trace::{Lerp, TraceGraph, boolean, int1},
    };
    use std::ops::{Add, Div, Mul, Neg, Sub};

    #[derive(Debug, Clone, Copy)]
    pub struct float1(pub(crate) u32);

    #[derive(Debug, Clone, Copy)]
    pub struct float2 {
        x: float1,
        y: float1,
    }

    #[derive(Debug, Clone, Copy)]
    pub struct float3 {
        x: float1,
        y: float1,
        z: float1,
    }

    #[derive(Debug, Clone, Copy)]
    pub struct float4 {
        x: float1,
        y: float1,
        z: float1,
        w: float1,
    }

    impl float1 {
        pub fn read(offset: u32) -> Self {
            assert!(offset % 4 == 0, "f32 reads must be 4-byte aligned");
            Self(TraceGraph::emit(ShaderOp::ReadF32(offset)))
        }

        pub fn sin(self) -> Self {
            Self(TraceGraph::emit(ShaderOp::Sin(self.0)))
        }

        pub fn cos(self) -> Self {
            Self(TraceGraph::emit(ShaderOp::Cos(self.0)))
        }

        pub fn tan(self) -> Self {
            Self(TraceGraph::emit(ShaderOp::Tan(self.0)))
        }

        pub fn asin(self) -> Self {
            Self(TraceGraph::emit(ShaderOp::Asin(self.0)))
        }

        pub fn acos(self) -> Self {
            Self(TraceGraph::emit(ShaderOp::Acos(self.0)))
        }

        pub fn atan(self) -> Self {
            Self(TraceGraph::emit(ShaderOp::Atan(self.0)))
        }

        pub fn sqrt(self) -> Self {
            Self(TraceGraph::emit(ShaderOp::Sqrt(self.0)))
        }

        pub fn exp(self) -> Self {
            Self(TraceGraph::emit(ShaderOp::Exp(self.0)))
        }

        pub fn ln(self) -> Self {
            Self(TraceGraph::emit(ShaderOp::Ln(self.0)))
        }

        pub fn floor(self) -> Self {
            Self(TraceGraph::emit(ShaderOp::Floor(self.0)))
        }

        pub fn dx(self) -> Self {
            Self(TraceGraph::emit(ShaderOp::DerivX(self.0)))
        }

        pub fn dy(self) -> Self {
            Self(TraceGraph::emit(ShaderOp::DerivY(self.0)))
        }

        pub fn abs(self) -> Self {
            Self(TraceGraph::emit(ShaderOp::FAbs(self.0)))
        }

        pub fn min(self, rhs: impl Into<Self>) -> Self {
            Self(TraceGraph::emit(ShaderOp::FMin(self.0, rhs.into().0)))
        }

        pub fn max(self, rhs: impl Into<Self>) -> Self {
            Self(TraceGraph::emit(ShaderOp::FMax(self.0, rhs.into().0)))
        }

        pub fn clamp(self, min: impl Into<Self>, max: impl Into<Self>) -> Self {
            self.min(max).max(min)
        }

        pub fn atan2(self, rhs: impl Into<Self>) -> Self {
            Self(TraceGraph::emit(ShaderOp::Atan2(self.0, rhs.into().0)))
        }

        pub fn pow(self, rhs: impl Into<Self>) -> Self {
            Self(TraceGraph::emit(ShaderOp::Pow(self.0, rhs.into().0)))
        }

        pub fn rem_euclid(self, rhs: impl Into<Self>) -> Self {
            Self(TraceGraph::emit(ShaderOp::FMod(self.0, rhs.into().0)))
        }

        pub fn lerp<T: Lerp<Self>>(self, start: T, end: impl Into<T>) -> T {
            T::lerp(self, start, end.into())
        }
    }

    impl float2 {
        pub fn x(self) -> float1 {
            self.x
        }

        pub fn y(self) -> float1 {
            self.y
        }

        pub fn position() -> Self {
            Self {
                x: float1(TraceGraph::emit(ShaderOp::PosX)),
                y: float1(TraceGraph::emit(ShaderOp::PosY)),
            }
        }

        pub fn resolution() -> Self {
            Self {
                x: float1(TraceGraph::emit(ShaderOp::ResX)),
                y: float1(TraceGraph::emit(ShaderOp::ResY)),
            }
        }

        pub fn dot(self, other: impl Into<Self>) -> float1 {
            let other = other.into();
            self.x * other.x + self.y * other.y
        }

        pub fn len(self) -> float1 {
            self.dot(self).sqrt()
        }

        pub fn norm(self) -> Self {
            self / self.len()
        }
    }

    impl float3 {
        pub fn x(self) -> float1 {
            self.x
        }

        pub fn y(self) -> float1 {
            self.y
        }

        pub fn z(self) -> float1 {
            self.z
        }

        pub fn dot(self, other: impl Into<Self>) -> float1 {
            let other = other.into();
            self.x * other.x + self.y * other.y + self.z * other.z
        }

        pub fn len(self) -> float1 {
            self.dot(self).sqrt()
        }

        pub fn norm(self) -> Self {
            self / self.len()
        }
    }

    impl float4 {
        pub fn x(self) -> float1 {
            self.x
        }

        pub fn y(self) -> float1 {
            self.y
        }

        pub fn z(self) -> float1 {
            self.z
        }

        pub fn w(self) -> float1 {
            self.w
        }

        pub fn dot(self, other: impl Into<Self>) -> float1 {
            let other = other.into();
            self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w
        }

        pub fn len(self) -> float1 {
            self.dot(self).sqrt()
        }

        pub fn norm(self) -> Self {
            self / self.len()
        }
    }

    impl Add for float1 {
        type Output = Self;

        fn add(self, rhs: Self) -> Self::Output {
            Self(TraceGraph::emit(ShaderOp::FAdd(self.0, rhs.0)))
        }
    }

    impl Sub for float1 {
        type Output = Self;

        fn sub(self, rhs: Self) -> Self::Output {
            Self(TraceGraph::emit(ShaderOp::FSub(self.0, rhs.0)))
        }
    }

    impl Mul for float1 {
        type Output = Self;

        fn mul(self, rhs: Self) -> Self::Output {
            Self(TraceGraph::emit(ShaderOp::FMul(self.0, rhs.0)))
        }
    }

    impl Div for float1 {
        type Output = Self;

        fn div(self, rhs: Self) -> Self::Output {
            Self(TraceGraph::emit(ShaderOp::FDiv(self.0, rhs.0)))
        }
    }

    impl Neg for float1 {
        type Output = Self;

        fn neg(self) -> Self::Output {
            Self(TraceGraph::emit(ShaderOp::FNeg(self.0)))
        }
    }

    impl Add<f32> for float1 {
        type Output = Self;
        fn add(self, rhs: f32) -> Self::Output {
            self + Self::from(rhs)
        }
    }

    impl Add<float1> for f32 {
        type Output = float1;
        fn add(self, rhs: float1) -> Self::Output {
            rhs + self
        }
    }

    impl Sub<f32> for float1 {
        type Output = Self;
        fn sub(self, rhs: f32) -> Self::Output {
            self - Self::from(rhs)
        }
    }

    impl Sub<float1> for f32 {
        type Output = float1;
        fn sub(self, rhs: float1) -> Self::Output {
            rhs - self
        }
    }

    impl Mul<f32> for float1 {
        type Output = Self;
        fn mul(self, rhs: f32) -> Self::Output {
            self * Self::from(rhs)
        }
    }

    impl Mul<float1> for f32 {
        type Output = float1;
        fn mul(self, rhs: float1) -> Self::Output {
            rhs * self
        }
    }

    impl Div<f32> for float1 {
        type Output = Self;
        fn div(self, rhs: f32) -> Self::Output {
            self / Self::from(rhs)
        }
    }

    impl Div<float1> for f32 {
        type Output = float1;
        fn div(self, rhs: float1) -> Self::Output {
            rhs / self
        }
    }

    impl From<i32> for float1 {
        fn from(value: i32) -> Self {
            Self(TraceGraph::emit(ShaderOp::FLit(value as f32)))
        }
    }

    impl From<f32> for float1 {
        fn from(value: f32) -> Self {
            Self(TraceGraph::emit(ShaderOp::FLit(value)))
        }
    }

    impl From<int1> for float1 {
        fn from(value: int1) -> Self {
            Self(TraceGraph::emit(ShaderOp::FCastInt32(value.0)))
        }
    }

    impl Lerp<float1> for float1 {
        fn lerp(sel: Self, start: Self, end: Self) -> Self {
            Self(TraceGraph::emit(ShaderOp::Lerp(sel.0, start.0, end.0)))
        }
    }

    impl Lerp<float1> for float2 {
        fn lerp(sel: float1, start: Self, end: Self) -> Self {
            Self {
                x: sel.lerp(start.x, end.x),
                y: sel.lerp(start.y, end.y),
            }
        }
    }

    impl Lerp<float1> for float3 {
        fn lerp(sel: float1, start: Self, end: Self) -> Self {
            Self {
                x: sel.lerp(start.x, end.x),
                y: sel.lerp(start.y, end.y),
                z: sel.lerp(start.z, end.z),
            }
        }
    }

    impl Lerp<float1> for float4 {
        fn lerp(sel: float1, start: Self, end: Self) -> Self {
            Self {
                x: sel.lerp(start.x, end.x),
                y: sel.lerp(start.y, end.y),
                z: sel.lerp(start.z, end.z),
                w: sel.lerp(start.w, end.w),
            }
        }
    }

    impl Lerp<float2> for float2 {
        fn lerp(sel: Self, start: Self, end: Self) -> Self {
            Self {
                x: sel.x.lerp(start.x, end.x),
                y: sel.y.lerp(start.y, end.y),
            }
        }
    }

    impl Lerp<float3> for float3 {
        fn lerp(sel: Self, start: Self, end: Self) -> Self {
            Self {
                x: sel.x.lerp(start.x, end.x),
                y: sel.y.lerp(start.y, end.y),
                z: sel.z.lerp(start.z, end.z),
            }
        }
    }

    impl Lerp<float4> for float4 {
        fn lerp(sel: Self, start: Self, end: Self) -> Self {
            Self {
                x: sel.x.lerp(start.x, end.x),
                y: sel.y.lerp(start.y, end.y),
                z: sel.z.lerp(start.z, end.z),
                w: sel.w.lerp(start.w, end.w),
            }
        }
    }

    impl Lerp<boolean> for float1 {
        fn lerp(sel: boolean, start: Self, end: Self) -> Self {
            Self(TraceGraph::emit(ShaderOp::FSelect(sel.0, start.0, end.0)))
        }
    }

    impl Lerp<boolean> for float2 {
        fn lerp(sel: boolean, start: Self, end: Self) -> Self {
            Self {
                x: sel.select(start.x, end.x),
                y: sel.select(start.y, end.y),
            }
        }
    }

    impl Lerp<boolean> for float3 {
        fn lerp(sel: boolean, start: Self, end: Self) -> Self {
            Self {
                x: sel.select(start.x, end.x),
                y: sel.select(start.y, end.y),
                z: sel.select(start.z, end.z),
            }
        }
    }

    impl Lerp<boolean> for float4 {
        fn lerp(sel: boolean, start: Self, end: Self) -> Self {
            Self {
                x: sel.select(start.x, end.x),
                y: sel.select(start.y, end.y),
                z: sel.select(start.z, end.z),
                w: sel.select(start.w, end.w),
            }
        }
    }

    impl<X: Into<float1>> From<X> for float2 {
        fn from(value: X) -> Self {
            let value = value.into();
            Self { x: value, y: value }
        }
    }

    impl<X: Into<float1>> From<X> for float3 {
        fn from(value: X) -> Self {
            let value = value.into();
            Self {
                x: value,
                y: value,
                z: value,
            }
        }
    }

    impl<X: Into<float1>> From<X> for float4 {
        fn from(value: X) -> Self {
            let value = value.into();
            Self {
                x: value,
                y: value,
                z: value,
                w: value,
            }
        }
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

    macro_rules! impl_float_vec_forward {
        ($type:ty, $trait:ident::$func:ident, $($field:ident),*) => {
            impl $trait for $type {
                type Output = Self;
                fn $func(self, rhs: Self) -> Self::Output {
                    Self { $($field: $trait::$func(self.$field, rhs.$field)),* }
                }
            }

            impl $trait<float1> for $type {
                type Output = Self;
                fn $func(self, rhs: float1) -> Self::Output {
                    Self { $($field: $trait::$func(self.$field, rhs)),* }
                }
            }

            impl $trait<$type> for float1  {
                type Output = $type;
                fn $func(self, rhs: $type) -> Self::Output {
                    Self::Output { $($field: $trait::$func(self, rhs.$field)),* }
                }
            }

            impl $trait<f32> for $type {
                type Output = Self;
                fn $func(self, rhs: f32) -> Self::Output {
                    Self { $($field: $trait::$func(self.$field, rhs)),* }
                }
            }

            impl $trait<$type> for f32  {
                type Output = $type;
                fn $func(self, rhs: $type) -> Self::Output {
                    Self::Output { $($field: $trait::$func(self, rhs.$field)),* }
                }
            }
        };

        ($type:ty, $func:ident, $($field:ident),*) => {
            impl $type {
                pub fn $func(self) -> Self {
                    Self { $($field: self.$field.$func()),* }
                }
            }
        };

        ($type:ty, $func:ident(_), $($field:ident),*) => {
            impl $type {
                pub fn $func(self, arg0: impl Into<Self>) -> Self {
                    let arg0 = arg0.into();
                    Self { $($field: self.$field.$func(arg0.$field)),* }
                }
            }
        };

        ($type:ty, $func:ident(_, _), $($field:ident),*) => {
            impl $type {
                pub fn $func(self, arg0: impl Into<Self>, arg1: impl Into<Self>) -> Self {
                    let arg0 = arg0.into();
                    let arg1 = arg1.into();
                    Self { $($field: self.$field.$func(arg0.$field, arg1.$field)),* }
                }
            }
        };
    }

    impl_float_vec_forward!(float2, Add::add, x, y);
    impl_float_vec_forward!(float3, Add::add, x, y, z);
    impl_float_vec_forward!(float4, Add::add, x, y, z, w);
    impl_float_vec_forward!(float2, Sub::sub, x, y);
    impl_float_vec_forward!(float3, Sub::sub, x, y, z);
    impl_float_vec_forward!(float4, Sub::sub, x, y, z, w);
    impl_float_vec_forward!(float2, Mul::mul, x, y);
    impl_float_vec_forward!(float3, Mul::mul, x, y, z);
    impl_float_vec_forward!(float4, Mul::mul, x, y, z, w);
    impl_float_vec_forward!(float2, Div::div, x, y);
    impl_float_vec_forward!(float3, Div::div, x, y, z);
    impl_float_vec_forward!(float4, Div::div, x, y, z, w);
    impl_float_vec_forward!(float2, sin, x, y);
    impl_float_vec_forward!(float2, cos, x, y);
    impl_float_vec_forward!(float2, tan, x, y);
    impl_float_vec_forward!(float2, asin, x, y);
    impl_float_vec_forward!(float2, acos, x, y);
    impl_float_vec_forward!(float2, atan, x, y);
    impl_float_vec_forward!(float2, sqrt, x, y);
    impl_float_vec_forward!(float2, exp, x, y);
    impl_float_vec_forward!(float2, ln, x, y);
    impl_float_vec_forward!(float2, floor, x, y);
    impl_float_vec_forward!(float2, abs, x, y);
    impl_float_vec_forward!(float2, dx, x, y);
    impl_float_vec_forward!(float2, dy, x, y);
    impl_float_vec_forward!(float2, rem_euclid(_), x, y);
    impl_float_vec_forward!(float2, pow(_), x, y);
    impl_float_vec_forward!(float2, atan2(_), x, y);
    impl_float_vec_forward!(float2, min(_), x, y);
    impl_float_vec_forward!(float2, max(_), x, y);
    impl_float_vec_forward!(float2, clamp(_, _), x, y);
    impl_float_vec_forward!(float2, lerp(_, _), x, y);
    impl_float_vec_forward!(float3, sin, x, y, z);
    impl_float_vec_forward!(float3, cos, x, y, z);
    impl_float_vec_forward!(float3, tan, x, y, z);
    impl_float_vec_forward!(float3, asin, x, y, z);
    impl_float_vec_forward!(float3, acos, x, y, z);
    impl_float_vec_forward!(float3, atan, x, y, z);
    impl_float_vec_forward!(float3, sqrt, x, y, z);
    impl_float_vec_forward!(float3, exp, x, y, z);
    impl_float_vec_forward!(float3, ln, x, y, z);
    impl_float_vec_forward!(float3, floor, x, y, z);
    impl_float_vec_forward!(float3, abs, x, y, z);
    impl_float_vec_forward!(float3, dx, x, y, z);
    impl_float_vec_forward!(float3, dy, x, y, z);
    impl_float_vec_forward!(float3, rem_euclid(_), x, y, z);
    impl_float_vec_forward!(float3, pow(_), x, y, z);
    impl_float_vec_forward!(float3, atan2(_), x, y, z);
    impl_float_vec_forward!(float3, min(_), x, y, z);
    impl_float_vec_forward!(float3, max(_), x, y, z);
    impl_float_vec_forward!(float3, clamp(_, _), x, y, z);
    impl_float_vec_forward!(float3, lerp(_, _), x, y, z);
    impl_float_vec_forward!(float4, sin, x, y, z, w);
    impl_float_vec_forward!(float4, cos, x, y, z, w);
    impl_float_vec_forward!(float4, tan, x, y, z, w);
    impl_float_vec_forward!(float4, asin, x, y, z, w);
    impl_float_vec_forward!(float4, acos, x, y, z, w);
    impl_float_vec_forward!(float4, atan, x, y, z, w);
    impl_float_vec_forward!(float4, sqrt, x, y, z, w);
    impl_float_vec_forward!(float4, exp, x, y, z, w);
    impl_float_vec_forward!(float4, ln, x, y, z, w);
    impl_float_vec_forward!(float4, floor, x, y, z, w);
    impl_float_vec_forward!(float4, abs, x, y, z, w);
    impl_float_vec_forward!(float4, dx, x, y, z, w);
    impl_float_vec_forward!(float4, dy, x, y, z, w);
    impl_float_vec_forward!(float4, rem_euclid(_), x, y, z, w);
    impl_float_vec_forward!(float4, pow(_), x, y, z, w);
    impl_float_vec_forward!(float4, atan2(_), x, y, z, w);
    impl_float_vec_forward!(float4, min(_), x, y, z, w);
    impl_float_vec_forward!(float4, max(_), x, y, z, w);
    impl_float_vec_forward!(float4, clamp(_, _), x, y, z, w);
    impl_float_vec_forward!(float4, lerp(_, _), x, y, z, w);
}

mod bool {
    use crate::{
        ShaderOp,
        trace::{Lerp, TraceGraph, float1, int1},
    };
    use std::ops::{BitAnd, BitOr, BitXor, Not};

    #[derive(Debug, Clone, Copy)]
    pub struct boolean(pub(crate) u32);

    impl BitAnd for boolean {
        type Output = Self;

        fn bitand(self, rhs: Self) -> Self::Output {
            Self(TraceGraph::emit(ShaderOp::BAnd(self.0, rhs.0)))
        }
    }

    impl BitOr for boolean {
        type Output = Self;

        fn bitor(self, rhs: Self) -> Self::Output {
            Self(TraceGraph::emit(ShaderOp::BOr(self.0, rhs.0)))
        }
    }

    impl BitXor for boolean {
        type Output = Self;

        fn bitxor(self, rhs: Self) -> Self::Output {
            Self(TraceGraph::emit(ShaderOp::BXor(self.0, rhs.0)))
        }
    }

    impl Not for boolean {
        type Output = Self;

        fn not(self) -> Self::Output {
            Self(TraceGraph::emit(ShaderOp::BNot(self.0)))
        }
    }

    impl From<bool> for boolean {
        fn from(value: bool) -> Self {
            Self(TraceGraph::emit(ShaderOp::BLit(value)))
        }
    }

    impl Lerp<boolean> for boolean {
        fn lerp(switch: boolean, x: Self, y: Self) -> Self {
            Self(TraceGraph::emit(ShaderOp::BSelect(switch.0, x.0, y.0)))
        }
    }

    impl boolean {
        pub fn select<T: Lerp<Self>>(self, start: T, end: impl Into<T>) -> T {
            T::lerp(self, start, end.into())
        }
    }

    impl float1 {
        pub fn eq(self, other: impl Into<Self>) -> boolean {
            boolean(TraceGraph::emit(ShaderOp::FEq(self.0, other.into().0)))
        }

        pub fn ne(self, other: impl Into<Self>) -> boolean {
            boolean(TraceGraph::emit(ShaderOp::FNe(self.0, other.into().0)))
        }

        pub fn le(self, other: impl Into<Self>) -> boolean {
            boolean(TraceGraph::emit(ShaderOp::FLe(self.0, other.into().0)))
        }

        pub fn lt(self, other: impl Into<Self>) -> boolean {
            boolean(TraceGraph::emit(ShaderOp::FLt(self.0, other.into().0)))
        }

        pub fn ge(self, other: impl Into<Self>) -> boolean {
            boolean(TraceGraph::emit(ShaderOp::FGe(self.0, other.into().0)))
        }

        pub fn gt(self, other: impl Into<Self>) -> boolean {
            boolean(TraceGraph::emit(ShaderOp::FGt(self.0, other.into().0)))
        }
    }

    impl int1 {
        pub fn eq(self, other: impl Into<Self>) -> boolean {
            boolean(TraceGraph::emit(ShaderOp::IEq(self.0, other.into().0)))
        }

        pub fn ne(self, other: impl Into<Self>) -> boolean {
            boolean(TraceGraph::emit(ShaderOp::INe(self.0, other.into().0)))
        }

        pub fn le(self, other: impl Into<Self>) -> boolean {
            boolean(TraceGraph::emit(ShaderOp::ILe(self.0, other.into().0)))
        }

        pub fn lt(self, other: impl Into<Self>) -> boolean {
            boolean(TraceGraph::emit(ShaderOp::ILt(self.0, other.into().0)))
        }

        pub fn ge(self, other: impl Into<Self>) -> boolean {
            boolean(TraceGraph::emit(ShaderOp::IGe(self.0, other.into().0)))
        }

        pub fn gt(self, other: impl Into<Self>) -> boolean {
            boolean(TraceGraph::emit(ShaderOp::IGt(self.0, other.into().0)))
        }
    }
}

mod util {
    pub trait Lerp<T> {
        #[doc(hidden)]
        fn lerp(sel: T, start: Self, end: Self) -> Self;
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
