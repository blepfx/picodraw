use crate::util::Pod;

pub trait VMTile: Pod + Copy + Sized + 'static {
    const WIDTH: usize;
    const HEIGHT: usize;

    fn as_slice(&self) -> &[VMSlot];
    fn as_slice_mut(&mut self) -> &mut [VMSlot];

    #[inline(always)]
    fn as_f32(&self) -> &[f32] {
        VMSlot::cast_slice::<f32>(self.as_slice())
    }

    #[inline(always)]
    fn as_f32_mut(&mut self) -> &mut [f32] {
        VMSlot::cast_slice_mut::<f32>(self.as_slice_mut())
    }

    #[inline(always)]
    fn as_i32(&self) -> &[i32] {
        VMSlot::cast_slice::<i32>(self.as_slice())
    }

    #[inline(always)]
    fn as_i32_mut(&mut self) -> &mut [i32] {
        VMSlot::cast_slice_mut::<i32>(self.as_slice_mut())
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union VMSlot {
    int: i32,
    float: f32,
}

impl Default for VMSlot {
    #[inline(always)]
    fn default() -> Self {
        Self { int: 0 }
    }
}

impl From<i32> for VMSlot {
    #[inline(always)]
    fn from(value: i32) -> Self {
        Self { int: value }
    }
}

impl From<f32> for VMSlot {
    #[inline(always)]
    fn from(value: f32) -> Self {
        Self { float: value }
    }
}

impl From<VMSlot> for i32 {
    #[inline(always)]
    fn from(slot: VMSlot) -> Self {
        unsafe { slot.int }
    }
}

impl From<VMSlot> for f32 {
    #[inline(always)]
    fn from(slot: VMSlot) -> Self {
        unsafe { slot.float }
    }
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
            pub fn as_slice(&self) -> &[VMSlot; $width * $height] {
                self.cast_ref()
            }

            #[inline(always)]
            pub fn as_slice_mut(&mut self) -> &mut [VMSlot; $width * $height] {
                self.cast_mut()
            }
        }

        impl VMTile for $type {
            const WIDTH: usize = $width;
            const HEIGHT: usize = $height;

            #[inline(always)]
            fn as_slice(&self) -> &[VMSlot] {
                &self.as_slice()[..]
            }

            #[inline(always)]
            fn as_slice_mut(&mut self) -> &mut [VMSlot] {
                &mut self.as_slice_mut()[..]
            }
        }
    };
}

impl_tile!(VMTile16, 16, 16);
impl_tile!(VMTile8, 8, 8);
impl_tile!(VMTile4, 4, 4);
impl_tile!(VMSlot, 1, 1);
