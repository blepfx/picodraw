#![allow(non_camel_case_types)]

mod int {
    use crate::{
        ShaderOp,
        trace::{Lerp, boolean, emit, float1, inspect},
    };
    use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Neg, Not, Shl, Shr, Sub};

    /// A traced integer value.
    #[derive(Debug, Clone, Copy)]
    pub struct int1(pub(crate) u32);

    impl Add for int1 {
        type Output = Self;

        fn add(self, rhs: Self) -> Self::Output {
            Self(emit(ShaderOp::IAdd(self.0, rhs.0)))
        }
    }

    impl Sub for int1 {
        type Output = Self;

        fn sub(self, rhs: Self) -> Self::Output {
            Self(emit(ShaderOp::ISub(self.0, rhs.0)))
        }
    }

    impl Mul for int1 {
        type Output = Self;

        fn mul(self, rhs: Self) -> Self::Output {
            Self(emit(ShaderOp::IMul(self.0, rhs.0)))
        }
    }

    impl Div for int1 {
        type Output = Self;

        fn div(self, rhs: Self) -> Self::Output {
            Self(emit(ShaderOp::IDiv(self.0, rhs.0)))
        }
    }

    impl Neg for int1 {
        type Output = Self;

        fn neg(self) -> Self::Output {
            Self(emit(ShaderOp::INeg(self.0)))
        }
    }

    impl Add<i32> for int1 {
        type Output = Self;
        fn add(self, rhs: i32) -> Self::Output {
            self + Self::from(rhs)
        }
    }

    impl Add<int1> for i32 {
        type Output = int1;
        fn add(self, rhs: int1) -> Self::Output {
            int1::from(self) + rhs
        }
    }

    impl Sub<i32> for int1 {
        type Output = Self;
        fn sub(self, rhs: i32) -> Self::Output {
            self - Self::from(rhs)
        }
    }

    impl Sub<int1> for i32 {
        type Output = int1;
        fn sub(self, rhs: int1) -> Self::Output {
            int1::from(self) - rhs
        }
    }

    impl Mul<i32> for int1 {
        type Output = Self;
        fn mul(self, rhs: i32) -> Self::Output {
            self * Self::from(rhs)
        }
    }

    impl Mul<int1> for i32 {
        type Output = int1;
        fn mul(self, rhs: int1) -> Self::Output {
            int1::from(self) * rhs
        }
    }

    impl Div<i32> for int1 {
        type Output = Self;
        fn div(self, rhs: i32) -> Self::Output {
            self / Self::from(rhs)
        }
    }

    impl Div<int1> for i32 {
        type Output = int1;
        fn div(self, rhs: int1) -> Self::Output {
            int1::from(self) / rhs
        }
    }

    impl BitAnd for int1 {
        type Output = Self;

        fn bitand(self, rhs: Self) -> Self::Output {
            Self(emit(ShaderOp::IAnd(self.0, rhs.0)))
        }
    }

    impl BitOr for int1 {
        type Output = Self;

        fn bitor(self, rhs: Self) -> Self::Output {
            Self(emit(ShaderOp::IOr(self.0, rhs.0)))
        }
    }

    impl BitXor for int1 {
        type Output = Self;

        fn bitxor(self, rhs: Self) -> Self::Output {
            Self(emit(ShaderOp::IXor(self.0, rhs.0)))
        }
    }

    impl BitAnd<i32> for int1 {
        type Output = Self;
        fn bitand(self, rhs: i32) -> Self::Output {
            self & Self::from(rhs)
        }
    }

    impl BitAnd<int1> for i32 {
        type Output = int1;
        fn bitand(self, rhs: int1) -> Self::Output {
            int1::from(self) & rhs
        }
    }

    impl BitOr<i32> for int1 {
        type Output = Self;
        fn bitor(self, rhs: i32) -> Self::Output {
            self | Self::from(rhs)
        }
    }

    impl BitOr<int1> for i32 {
        type Output = int1;
        fn bitor(self, rhs: int1) -> Self::Output {
            int1::from(self) | rhs
        }
    }

    impl BitXor<i32> for int1 {
        type Output = Self;
        fn bitxor(self, rhs: i32) -> Self::Output {
            self ^ Self::from(rhs)
        }
    }

    impl BitXor<int1> for i32 {
        type Output = int1;
        fn bitxor(self, rhs: int1) -> Self::Output {
            int1::from(self) ^ rhs
        }
    }

    impl Shl for int1 {
        type Output = Self;

        fn shl(self, rhs: Self) -> Self::Output {
            Self(emit(ShaderOp::IShl(self.0, rhs.0)))
        }
    }

    impl Shr for int1 {
        type Output = Self;

        fn shr(self, rhs: Self) -> Self::Output {
            Self(emit(ShaderOp::IShr(self.0, rhs.0)))
        }
    }

    impl Shl<i32> for int1 {
        type Output = Self;
        fn shl(self, rhs: i32) -> Self::Output {
            self << Self::from(rhs)
        }
    }

    impl Shl<int1> for i32 {
        type Output = int1;
        fn shl(self, rhs: int1) -> Self::Output {
            int1::from(self) << rhs
        }
    }

    impl Shr<i32> for int1 {
        type Output = Self;
        fn shr(self, rhs: i32) -> Self::Output {
            self >> Self::from(rhs)
        }
    }

    impl Shr<int1> for i32 {
        type Output = int1;
        fn shr(self, rhs: int1) -> Self::Output {
            int1::from(self) >> rhs
        }
    }

    impl Not for int1 {
        type Output = Self;

        fn not(self) -> Self::Output {
            Self(emit(ShaderOp::INot(self.0)))
        }
    }

    impl Lerp<boolean> for int1 {
        fn lerp(sel: boolean, start: Self, end: Self) -> Self {
            Self(emit(ShaderOp::ISelect(sel.0, start.0, end.0)))
        }
    }

    impl From<i32> for int1 {
        fn from(value: i32) -> Self {
            Self(emit(ShaderOp::ILit(value)))
        }
    }

    impl From<float1> for int1 {
        fn from(value: float1) -> Self {
            Self(emit(ShaderOp::ICastFloat(value.0)))
        }
    }

    impl int1 {
        /// If this integer is a literal, returns its value. Otherwise, returns None.
        ///
        /// This is mainly useful for optimization purposes,
        /// like implementing a separate code path if the value is known at compile time.
        pub fn as_lit(&self) -> Option<i32> {
            match inspect(self.0) {
                ShaderOp::ILit(v) => Some(v),
                _ => None,
            }
        }

        /// Read an unsigned 8-bit integer from the given byte offset in the object's data buffer.
        pub fn read_u8(offset: u32) -> Self {
            Self(emit(ShaderOp::ReadU8(offset)))
        }

        /// Read an unsigned 16-bit integer from the given byte offset in the object's data buffer.
        ///
        /// # Panics
        /// This function will panic if the offset is not 2-byte aligned.
        pub fn read_u16(offset: u32) -> Self {
            assert!(offset.is_multiple_of(2), "u16 reads must be 2-byte aligned");
            Self(emit(ShaderOp::ReadU16(offset / 2)))
        }

        /// Read a signed 32-bit integer from the given byte offset in the object's data buffer.
        ///
        /// # Panics
        /// This function will panic if the offset is not 4-byte aligned.
        pub fn read_i32(offset: u32) -> Self {
            assert!(offset.is_multiple_of(4), "i32 reads must be 4-byte aligned");
            Self(emit(ShaderOp::ReadI32(offset / 4)))
        }

        /// Get the remainder of the division.
        ///
        /// See [`i32::rem_euclid`] for more info.
        pub fn rem_euclid(self, rhs: impl Into<Self>) -> Self {
            Self(emit(ShaderOp::IMod(self.0, rhs.into().0)))
        }

        /// Get the absolute value.
        ///
        /// See [`i32::abs`] for more info.
        pub fn abs(self) -> int1 {
            Self(emit(ShaderOp::IAbs(self.0)))
        }

        /// Get the sign of the value.
        ///
        /// See [`i32::signum`] for more info.
        pub fn signum(self) -> Self {
            Self(emit(ShaderOp::ISign(self.0)))
        }

        /// Get the smallest of two values.
        ///
        /// See [`i32::min`] for more info.
        pub fn min(self, rhs: impl Into<Self>) -> Self {
            Self(emit(ShaderOp::IMin(self.0, rhs.into().0)))
        }

        /// Get the largest of two values.
        ///
        /// See [`i32::max`] for more info.
        pub fn max(self, rhs: impl Into<Self>) -> Self {
            Self(emit(ShaderOp::IMax(self.0, rhs.into().0)))
        }

        /// Clamp the value between a minimum and maximum value.
        ///
        /// See [`i32::clamp`] for more info.
        pub fn clamp(self, min: impl Into<Self>, max: impl Into<Self>) -> Self {
            self.min(max).max(min)
        }
    }
}

mod float {
    use crate::{
        ShaderOp,
        trace::{Lerp, boolean, emit, inspect, int1},
    };
    use std::ops::{Add, Div, Mul, Neg, Sub};

    /// A traced floating point value.
    #[derive(Debug, Clone, Copy)]
    pub struct float1(pub(crate) u32);

    /// A traced 2 dimensional floating point vector.
    #[derive(Debug, Clone, Copy)]
    pub struct float2 {
        x: float1,
        y: float1,
    }

    /// A traced 3 dimensional floating point vector.
    #[derive(Debug, Clone, Copy)]
    pub struct float3 {
        x: float1,
        y: float1,
        z: float1,
    }

    /// A traced 4 dimensional floating point vector.
    #[derive(Debug, Clone, Copy)]
    pub struct float4 {
        x: float1,
        y: float1,
        z: float1,
        w: float1,
    }

    impl float1 {
        /// If this float is a literal, returns its value. Otherwise, returns None.
        ///
        /// This is mainly useful for optimization purposes,
        /// like implementing a separate code path if the value is known at compile time.
        pub fn as_lit(&self) -> Option<f32> {
            match inspect(self.0) {
                ShaderOp::FLit(v) => Some(v),
                _ => None,
            }
        }

        /// Read a 32-bit floating point value from the given byte offset in the object's data buffer.
        ///
        /// # Panics
        /// This function will panic if the offset is not 4-byte aligned.
        pub fn read_f32(offset: u32) -> Self {
            assert!(offset.is_multiple_of(4), "f32 reads must be 4-byte aligned");
            Self(emit(ShaderOp::ReadF32(offset / 4)))
        }

        /// Compute the sine of the value (in radians).
        ///
        /// See [`f32::sin`] for more info.
        pub fn sin(self) -> Self {
            Self(emit(ShaderOp::Sin(self.0)))
        }

        /// Compute the cosine of the value (in radians).
        ///
        /// See [`f32::cos`] for more info.
        pub fn cos(self) -> Self {
            Self(emit(ShaderOp::Cos(self.0)))
        }

        /// Compute the tangent of the value (in radians).
        ///
        /// See [`f32::tan`] for more info.
        pub fn tan(self) -> Self {
            Self(emit(ShaderOp::Tan(self.0)))
        }

        /// Compute the arcsine of the value (in radians).
        ///
        /// See [`f32::asin`] for more info.
        pub fn asin(self) -> Self {
            Self(emit(ShaderOp::Asin(self.0)))
        }

        /// Compute the arccosine of the value (in radians).
        ///
        /// See [`f32::acos`] for more info.
        pub fn acos(self) -> Self {
            Self(emit(ShaderOp::Acos(self.0)))
        }

        /// Compute the arctangent of the value (in radians).
        ///
        /// See [`f32::atan`] for more info.
        pub fn atan(self) -> Self {
            Self(emit(ShaderOp::Atan(self.0)))
        }

        /// Compute the square root of the value.
        ///
        /// See [`f32::sqrt`] for more info.
        pub fn sqrt(self) -> Self {
            Self(emit(ShaderOp::Sqrt(self.0)))
        }

        /// Compute the exponential (e^x)of the value.
        ///
        /// See [`f32::exp`] for more info.
        pub fn exp(self) -> Self {
            Self(emit(ShaderOp::Exp(self.0)))
        }

        /// Compute the natural logarithm (base e) of the value.
        ///
        /// See [`f32::ln`] for more info.
        pub fn ln(self) -> Self {
            Self(emit(ShaderOp::Ln(self.0)))
        }

        /// Compute the largest integer less than or equal to the value.
        ///
        /// See [`f32::floor`] for more info.
        pub fn floor(self) -> Self {
            Self(emit(ShaderOp::Floor(self.0)))
        }

        /// Compute the derivative of the value in the X direction (difference between neighboring pixels).
        pub fn dx(self) -> Self {
            Self(emit(ShaderOp::DerivX(self.0)))
        }

        /// Compute the derivative of the value in the Y direction (difference between neighboring pixels).
        pub fn dy(self) -> Self {
            Self(emit(ShaderOp::DerivY(self.0)))
        }

        /// Compute the absolute value of the value.
        ///
        /// See [`f32::abs`] for more info.
        pub fn abs(self) -> Self {
            Self(emit(ShaderOp::FAbs(self.0)))
        }

        /// Compute the sign of the value.
        ///
        /// See [`f32::signum`] for more info.
        pub fn signum(self) -> Self {
            Self(emit(ShaderOp::FSign(self.0)))
        }

        /// Get the smallest of two values.
        ///
        /// See [`f32::min`] for more info.
        pub fn min(self, rhs: impl Into<Self>) -> Self {
            Self(emit(ShaderOp::FMin(self.0, rhs.into().0)))
        }

        /// Get the largest of two values.
        ///
        /// See [`f32::max`] for more info.
        pub fn max(self, rhs: impl Into<Self>) -> Self {
            Self(emit(ShaderOp::FMax(self.0, rhs.into().0)))
        }

        /// Clamp the value between a minimum and maximum value.
        ///
        /// See [`f32::clamp`] for more info.
        pub fn clamp(self, min: impl Into<Self>, max: impl Into<Self>) -> Self {
            self.min(max).max(min)
        }

        /// Compute atan(`rhs` / `self`) with consideration for which quadrant the point (self, rhs) is in.
        ///
        /// See [`f32::atan2`] for more info.
        pub fn atan2(self, rhs: impl Into<Self>) -> Self {
            Self(emit(ShaderOp::Atan2(self.0, rhs.into().0)))
        }

        /// Compute `self` raised to the power of `rhs`.
        ///
        /// See [`f32::powf`] for more info.
        pub fn powf(self, rhs: impl Into<Self>) -> Self {
            Self(emit(ShaderOp::Pow(self.0, rhs.into().0)))
        }

        /// Compute `self` raised to the power of `rhs` (an integer).
        /// Could be more efficient than `powf` (depending on the backend).
        ///
        /// See [`f32::powi`] for more info.
        pub fn powi(self, rhs: impl Into<int1>) -> Self {
            let rhs = rhs.into();
            match rhs.as_lit() {
                Some(-2) => 1.0 / (self * self),
                Some(-1) => 1.0 / self,
                Some(0) => Self::from(1.0),
                Some(1) => self,
                Some(2) => self * self,
                Some(3) => self * self * self,
                _ => self.powf(float1::from(rhs)),
            }
        }

        /// Get the remainder of the division.
        ///
        /// See [`f32::rem_euclid`] for more info.
        pub fn rem_euclid(self, rhs: impl Into<Self>) -> Self {
            Self(emit(ShaderOp::FMod(self.0, rhs.into().0)))
        }

        /// Linearly interpolate between two values.
        pub fn lerp<T: Lerp<Self>>(self, start: T, end: impl Into<T>) -> T {
            T::lerp(self, start, end.into())
        }
    }

    impl float2 {
        /// Get the X component.
        pub fn x(self) -> float1 {
            self.x
        }

        /// Get the Y component.
        pub fn y(self) -> float1 {
            self.y
        }

        /// Get the position of the current pixel in screen space.
        pub fn position() -> Self {
            Self {
                x: float1(emit(ShaderOp::PosX)),
                y: float1(emit(ShaderOp::PosY)),
            }
        }

        /// Get the resolution of the render target.
        pub fn resolution() -> Self {
            Self {
                x: float1(emit(ShaderOp::ResX)),
                y: float1(emit(ShaderOp::ResY)),
            }
        }

        /// Get the dot product with another vector.
        pub fn dot(self, other: impl Into<Self>) -> float1 {
            let other = other.into();
            self.x * other.x + self.y * other.y
        }

        /// Get the length of the vector.
        pub fn len(self) -> float1 {
            self.dot(self).sqrt()
        }

        /// Normalize the vector to length 1.
        pub fn norm(self) -> Self {
            self / self.len()
        }
    }

    impl float3 {
        /// Get the X component.
        pub fn x(self) -> float1 {
            self.x
        }

        /// Get the Y component.
        pub fn y(self) -> float1 {
            self.y
        }

        /// Get the Z component.
        pub fn z(self) -> float1 {
            self.z
        }

        /// Get the dot product with another vector.
        pub fn dot(self, other: impl Into<Self>) -> float1 {
            let other = other.into();
            self.x * other.x + self.y * other.y + self.z * other.z
        }

        /// Get the cross product with another vector.
        pub fn cross(self, other: impl Into<Self>) -> Self {
            let other = other.into();
            Self {
                x: self.y * other.z - self.z * other.y,
                y: self.z * other.x - self.x * other.z,
                z: self.x * other.y - self.y * other.x,
            }
        }

        /// Get the length of the vector.
        pub fn len(self) -> float1 {
            self.dot(self).sqrt()
        }

        /// Normalize the vector to length 1.
        pub fn norm(self) -> Self {
            self / self.len()
        }
    }

    impl float4 {
        /// Get the X component.
        #[doc(alias = "r")]
        pub fn x(self) -> float1 {
            self.x
        }

        /// Get the Y component.
        #[doc(alias = "g")]
        pub fn y(self) -> float1 {
            self.y
        }

        /// Get the Z component.
        #[doc(alias = "b")]
        pub fn z(self) -> float1 {
            self.z
        }

        /// Get the W component.
        #[doc(alias = "a")]
        pub fn w(self) -> float1 {
            self.w
        }

        /// Get the X component.
        #[doc(alias = "x")]
        pub fn r(self) -> float1 {
            self.x
        }

        /// Get the Y component.
        #[doc(alias = "y")]
        pub fn g(self) -> float1 {
            self.y
        }

        /// Get the Z component.
        #[doc(alias = "z")]
        pub fn b(self) -> float1 {
            self.z
        }

        /// Get the W component.
        #[doc(alias = "w")]
        pub fn a(self) -> float1 {
            self.w
        }

        /// Get the dot product with another vector.
        pub fn dot(self, other: impl Into<Self>) -> float1 {
            let other = other.into();
            self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w
        }

        /// Get the length of the vector.
        pub fn len(self) -> float1 {
            self.dot(self).sqrt()
        }

        /// Normalize the vector to length 1.
        pub fn norm(self) -> Self {
            self / self.len()
        }
    }

    impl Add for float1 {
        type Output = Self;

        fn add(self, rhs: Self) -> Self::Output {
            Self(emit(ShaderOp::FAdd(self.0, rhs.0)))
        }
    }

    impl Sub for float1 {
        type Output = Self;

        fn sub(self, rhs: Self) -> Self::Output {
            Self(emit(ShaderOp::FSub(self.0, rhs.0)))
        }
    }

    impl Mul for float1 {
        type Output = Self;

        fn mul(self, rhs: Self) -> Self::Output {
            Self(emit(ShaderOp::FMul(self.0, rhs.0)))
        }
    }

    impl Div for float1 {
        type Output = Self;

        fn div(self, rhs: Self) -> Self::Output {
            Self(emit(ShaderOp::FDiv(self.0, rhs.0)))
        }
    }

    impl Neg for float1 {
        type Output = Self;

        fn neg(self) -> Self::Output {
            Self(emit(ShaderOp::FNeg(self.0)))
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
            float1::from(self) - rhs
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
            float1::from(self) * rhs
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
            float1::from(self) / rhs
        }
    }

    impl From<i32> for float1 {
        fn from(value: i32) -> Self {
            Self(emit(ShaderOp::FLit(value as f32)))
        }
    }

    impl From<f32> for float1 {
        fn from(value: f32) -> Self {
            Self(emit(ShaderOp::FLit(value)))
        }
    }

    impl From<int1> for float1 {
        fn from(value: int1) -> Self {
            Self(emit(ShaderOp::FCastInt32(value.0)))
        }
    }

    impl Lerp<float1> for float1 {
        fn lerp(sel: Self, start: Self, end: Self) -> Self {
            Self(emit(ShaderOp::Lerp(sel.0, start.0, end.0)))
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
            Self(emit(ShaderOp::FSelect(sel.0, start.0, end.0)))
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
                #[doc = concat!("See [`float1::", stringify!($func), "`] for more info.")]
                pub fn $func(self) -> Self {
                    Self { $($field: self.$field.$func()),* }
                }
            }
        };

        ($type:ty, $func:ident(_), $($field:ident),*) => {
            impl $type {
                #[doc = concat!("See [`float1::", stringify!($func), "`] for more info.")]
                pub fn $func(self, arg0: impl Into<Self>) -> Self {
                    let arg0 = arg0.into();
                    Self { $($field: self.$field.$func(arg0.$field)),* }
                }
            }
        };

        ($type:ty, $func:ident(_, _), $($field:ident),*) => {
            impl $type {
                #[doc = concat!("See [`float1::", stringify!($func), "`] for more info.")]
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
    impl_float_vec_forward!(float2, powf(_), x, y);
    impl_float_vec_forward!(float2, powi(_), x, y);
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
    impl_float_vec_forward!(float3, powf(_), x, y, z);
    impl_float_vec_forward!(float3, powi(_), x, y, z);
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
    impl_float_vec_forward!(float4, powf(_), x, y, z, w);
    impl_float_vec_forward!(float4, powi(_), x, y, z, w);
    impl_float_vec_forward!(float4, atan2(_), x, y, z, w);
    impl_float_vec_forward!(float4, min(_), x, y, z, w);
    impl_float_vec_forward!(float4, max(_), x, y, z, w);
    impl_float_vec_forward!(float4, clamp(_, _), x, y, z, w);
    impl_float_vec_forward!(float4, lerp(_, _), x, y, z, w);
}

mod bool {
    use crate::{
        ShaderOp,
        trace::{Lerp, emit, float1, inspect, int1},
    };
    use std::ops::{BitAnd, BitOr, BitXor, Not};

    /// A traced boolean value.
    #[derive(Debug, Clone, Copy)]
    pub struct boolean(pub(crate) u32);

    impl BitAnd for boolean {
        type Output = Self;

        fn bitand(self, rhs: Self) -> Self::Output {
            Self(emit(ShaderOp::BAnd(self.0, rhs.0)))
        }
    }

    impl BitOr for boolean {
        type Output = Self;

        fn bitor(self, rhs: Self) -> Self::Output {
            Self(emit(ShaderOp::BOr(self.0, rhs.0)))
        }
    }

    impl BitXor for boolean {
        type Output = Self;

        fn bitxor(self, rhs: Self) -> Self::Output {
            Self(emit(ShaderOp::BXor(self.0, rhs.0)))
        }
    }

    impl Not for boolean {
        type Output = Self;

        fn not(self) -> Self::Output {
            Self(emit(ShaderOp::BNot(self.0)))
        }
    }

    impl From<bool> for boolean {
        fn from(value: bool) -> Self {
            Self(emit(ShaderOp::BLit(value)))
        }
    }

    impl Lerp<boolean> for boolean {
        fn lerp(switch: boolean, x: Self, y: Self) -> Self {
            Self(emit(ShaderOp::BSelect(switch.0, x.0, y.0)))
        }
    }

    impl boolean {
        /// If this boolean is a literal, returns its value. Otherwise, returns None.
        ///
        /// This is mainly useful for optimization purposes,
        /// like implementing a separate code path if the value is known at compile time.
        pub fn as_lit(&self) -> Option<bool> {
            match inspect(self.0) {
                ShaderOp::BLit(v) => Some(v),
                _ => None,
            }
        }

        /// Select between two values based on the boolean.
        ///
        /// Returns the first argument if `self` is true, otherwise returns the second argument.
        pub fn select<T: Lerp<Self>>(self, r#true: T, r#false: impl Into<T>) -> T {
            T::lerp(self, r#true, r#false.into())
        }
    }

    impl float1 {
        /// Returns true if the floats are equal.
        pub fn eq(self, other: impl Into<Self>) -> boolean {
            boolean(emit(ShaderOp::FEq(self.0, other.into().0)))
        }

        /// Returns true if the floats are not equal.
        pub fn ne(self, other: impl Into<Self>) -> boolean {
            boolean(emit(ShaderOp::FNe(self.0, other.into().0)))
        }

        /// Returns true if `self` is less than or equal to `other`.
        pub fn le(self, other: impl Into<Self>) -> boolean {
            boolean(emit(ShaderOp::FLe(self.0, other.into().0)))
        }

        /// Returns true if `self` is less than `other`.
        pub fn lt(self, other: impl Into<Self>) -> boolean {
            boolean(emit(ShaderOp::FLt(self.0, other.into().0)))
        }

        /// Returns true if `self` is greater than or equal to `other`.
        pub fn ge(self, other: impl Into<Self>) -> boolean {
            boolean(emit(ShaderOp::FGe(self.0, other.into().0)))
        }

        /// Returns true if `self` is greater than `other`.
        pub fn gt(self, other: impl Into<Self>) -> boolean {
            boolean(emit(ShaderOp::FGt(self.0, other.into().0)))
        }
    }

    impl int1 {
        /// Returns true if the integers are equal.
        pub fn eq(self, other: impl Into<Self>) -> boolean {
            boolean(emit(ShaderOp::IEq(self.0, other.into().0)))
        }

        /// Returns true if the integers are not equal.
        pub fn ne(self, other: impl Into<Self>) -> boolean {
            boolean(emit(ShaderOp::INe(self.0, other.into().0)))
        }

        /// Returns true if `self` is less than or equal to `other`.
        pub fn le(self, other: impl Into<Self>) -> boolean {
            boolean(emit(ShaderOp::ILe(self.0, other.into().0)))
        }

        /// Returns true if `self` is less than `other`.
        pub fn lt(self, other: impl Into<Self>) -> boolean {
            boolean(emit(ShaderOp::ILt(self.0, other.into().0)))
        }

        /// Returns true if `self` is greater than or equal to `other`.
        pub fn ge(self, other: impl Into<Self>) -> boolean {
            boolean(emit(ShaderOp::IGe(self.0, other.into().0)))
        }

        /// Returns true if `self` is greater than `other`.
        pub fn gt(self, other: impl Into<Self>) -> boolean {
            boolean(emit(ShaderOp::IGt(self.0, other.into().0)))
        }
    }
}

mod util {
    /// Trait for types that support linear interpolation (or selection) based on a selector value.
    pub trait Lerp<T> {
        #[doc(hidden)]
        fn lerp(sel: T, start: Self, end: Self) -> Self;
    }
}

mod texture {
    use super::{
        float::{float1, float2, float4},
        int::int1,
    };
    use crate::{ShaderOp, TextureChannel, TextureFilter, trace::emit};

    /// A traced 2D texture.
    #[derive(Debug, Clone, Copy)]
    pub struct texture2d(u32);

    impl texture2d {
        /// Create a `texture2d` from a texture index.
        ///
        /// Textures are added to objects via [`FrameEncoder::add_texture`](crate::FrameEncoder::add_texture).
        pub fn read(index: u32) -> Self {
            Self(index)
        }

        /// Get the width of the texture in pixels.
        pub fn width(&self) -> int1 {
            int1(emit(ShaderOp::TexW(self.0)))
        }

        /// Get the height of the texture in pixels.
        pub fn height(&self) -> int1 {
            int1(emit(ShaderOp::TexH(self.0)))
        }

        /// Sample the texture at the given pixel coordinates with the specified filtering mode.
        ///
        /// The coordinates are in pixel space (0 to width-1, 0 to height-1).
        ///
        /// TODO: clarify the behavior at the edges (clamp, wrap, etc.)
        pub fn sample(&self, xy: impl Into<float2>, filter: TextureFilter) -> float4 {
            let xy = xy.into();
            let x = xy.x();
            let y = xy.y();

            float4::from((
                float1(emit(ShaderOp::TexSample(self.0, x.0, y.0, filter, TextureChannel::Red))),
                float1(emit(ShaderOp::TexSample(
                    self.0,
                    x.0,
                    y.0,
                    filter,
                    TextureChannel::Green,
                ))),
                float1(emit(ShaderOp::TexSample(
                    self.0,
                    x.0,
                    y.0,
                    filter,
                    TextureChannel::Blue,
                ))),
                float1(emit(ShaderOp::TexSample(
                    self.0,
                    x.0,
                    y.0,
                    filter,
                    TextureChannel::Alpha,
                ))),
            ))
        }
    }
}

pub use bool::*;
pub use float::*;
pub use int::*;
pub use texture::*;
pub use util::*;

macro_rules! impl_constructor {
    ($($type:ident),*) => {
        $(
            #[doc = concat!("Construct a new `", stringify!($type), "` from a value that can be converted into it.")]
            pub fn $type(x: impl Into<$type>) -> $type {
                x.into()
            }
        )*
    };
}

impl_constructor!(float1, float2, float3, float4, int1, boolean);

/// A switch-case-like macro for selecting values based on conditions.
///
/// # Example
/// ```rust
/// use picodraw_core::{ShaderData, select, trace::{float1, float4, boolean}};
///
/// let shader = ShaderData::trace(|| {
///     let x = float1(5.0);
///     let result = select! {
///         x.gt(10.0) => float1(1.0),
///         x.gt(3.0) => float1(2.0),
///         else => float1(3.0)
///     }; // result will be 2.0
///
///     float4(result)
/// });
/// ```
#[macro_export]
macro_rules! select {
    (else => $default:expr $(,)?) => {
        $default
    };

    (
        $cond:expr => $body:expr,
        $( $cond_rest:expr => $body_rest:expr, )*
        else => $default:expr $(,)?
    ) => {
        $crate::trace::boolean::select(
            $cond,
            $body,
            $crate::select! {
                $( $cond_rest => $body_rest, )*
                else => $default
            }
        )
    };
}
