use crate::util::Pod;

pub unsafe trait VMTile: Pod + Copy + Sized + 'static {
    const WIDTH: usize;
    const HEIGHT: usize;

    fn as_f32(&self) -> &[f32];
    fn as_f32_mut(&mut self) -> &mut [f32];
    fn as_i32(&self) -> &[i32];
    fn as_i32_mut(&mut self) -> &mut [i32];
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union VMSlot {
    pub int: i32,
    pub float: f32,
}

#[derive(Copy, Clone)]
#[repr(C, align(128))]
pub struct VMTile16([[VMSlot; 16]; 16]);
#[derive(Copy, Clone)]
#[repr(C, align(128))]
pub struct VMTile8([[VMSlot; 8]; 8]);
#[derive(Copy, Clone)]
#[repr(C, align(64))]
pub struct VMTile4([[VMSlot; 4]; 4]);

macro_rules! impl_tile {
    ($type:ty, $width:literal, $height:literal) => {
        unsafe impl Pod for $type {}

        impl $type {
            #[inline(always)]
            pub fn as_f32(&self) -> &[f32; $width * $height] {
                self.cast_ref()
            }

            #[inline(always)]
            pub fn as_f32_mut(&mut self) -> &mut [f32; $width * $height] {
                self.cast_mut()
            }

            #[inline(always)]
            pub fn as_i32(&self) -> &[i32; $width * $height] {
                self.cast_ref()
            }

            #[inline(always)]
            pub fn as_i32_mut(&mut self) -> &mut [i32; $width * $height] {
                self.cast_mut()
            }
        }

        unsafe impl VMTile for $type {
            const WIDTH: usize = $width;
            const HEIGHT: usize = $height;

            #[inline(always)]
            fn as_f32(&self) -> &[f32] {
                self.as_f32()
            }

            #[inline(always)]
            fn as_f32_mut(&mut self) -> &mut [f32] {
                self.as_f32_mut()
            }

            #[inline(always)]
            fn as_i32(&self) -> &[i32] {
                self.as_i32()
            }

            #[inline(always)]
            fn as_i32_mut(&mut self) -> &mut [i32] {
                self.as_i32_mut()
            }
        }
    };
}

impl_tile!(VMTile16, 16, 16);
impl_tile!(VMTile8, 8, 8);
impl_tile!(VMTile4, 4, 4);
impl_tile!(VMSlot, 1, 1);
