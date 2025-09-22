use bytemuck::{Pod, Zeroable, must_cast_mut, must_cast_ref};

pub trait VMRegister: Pod + Copy + Sized + 'static {
    const SIZE: usize;

    fn as_f32(&self) -> &[f32];
    fn as_f32_mut(&mut self) -> &mut [f32];
    fn as_i32(&self) -> &[i32];
    fn as_i32_mut(&mut self) -> &mut [i32];
}

#[derive(Copy, Clone)]
pub union VMSlot {
    pub int: i32,
    pub float: f32,
}

unsafe impl Zeroable for VMSlot {}
unsafe impl Pod for VMSlot {}

impl VMRegister for VMSlot {
    const SIZE: usize = 1;

    #[inline(always)]
    fn as_f32(&self) -> &[f32] {
        must_cast_ref::<_, [f32; 1]>(self)
    }

    #[inline(always)]
    fn as_f32_mut(&mut self) -> &mut [f32] {
        must_cast_mut::<_, [f32; 1]>(self)
    }

    #[inline(always)]
    fn as_i32(&self) -> &[i32] {
        must_cast_ref::<_, [i32; 1]>(self)
    }

    #[inline(always)]
    fn as_i32_mut(&mut self) -> &mut [i32] {
        must_cast_mut::<_, [i32; 1]>(self)
    }
}

#[derive(Copy, Clone)]
#[repr(align(128))]
pub struct VMTile16([[VMSlot; 16]; 16]);
#[derive(Copy, Clone)]
#[repr(align(128))]
pub struct VMTile8([[VMSlot; 8]; 8]);
#[derive(Copy, Clone)]
#[repr(align(64))]
pub struct VMTile4([[VMSlot; 4]; 4]);
#[derive(Copy, Clone)]
#[repr(align(16))]
pub struct VMTile2([[VMSlot; 2]; 2]);

macro_rules! impl_tile {
    ($type:ty, $size:literal) => {
        unsafe impl Zeroable for $type {}
        unsafe impl Pod for $type {}

        impl $type {
            #[inline(always)]
            pub fn as_f32(&self) -> &[f32; $size * $size] {
                must_cast_ref(&self.0)
            }

            #[inline(always)]
            pub fn as_f32_mut(&mut self) -> &mut [f32; $size * $size] {
                must_cast_mut(&mut self.0)
            }

            #[inline(always)]
            pub fn as_i32(&self) -> &[i32; $size * $size] {
                must_cast_ref(&self.0)
            }

            #[inline(always)]
            pub fn as_i32_mut(&mut self) -> &mut [i32; $size * $size] {
                must_cast_mut(&mut self.0)
            }
        }

        impl VMRegister for $type {
            const SIZE: usize = $size;

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

impl_tile!(VMTile16, 16);
impl_tile!(VMTile8, 8);
impl_tile!(VMTile4, 4);
impl_tile!(VMTile2, 2);
