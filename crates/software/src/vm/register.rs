pub const TILE_SIZE: usize = 8;
pub const REGISTER_COUNT: usize = 64;
pub const PIXEL_COUNT: usize = TILE_SIZE * TILE_SIZE;

#[derive(Copy, Clone)]
pub union VMSlot {
    pub int: i32,
    pub float: f32,
}

#[derive(Copy, Clone)]
#[repr(align(128))]
pub struct VMTile([VMSlot; PIXEL_COUNT]);

impl VMTile {
    pub fn zeroed() -> Self {
        Self([VMSlot { int: 0 }; PIXEL_COUNT])
    }

    #[inline(always)]
    pub fn as_f32(&self) -> &[f32; PIXEL_COUNT] {
        unsafe { &*(&self.0 as *const _ as *const [f32; PIXEL_COUNT]) }
    }

    #[inline(always)]
    pub fn as_f32_mut(&mut self) -> &mut [f32; PIXEL_COUNT] {
        unsafe { &mut *(&mut self.0 as *mut _ as *mut [f32; PIXEL_COUNT]) }
    }

    #[inline(always)]
    pub fn as_i32(&self) -> &[i32; PIXEL_COUNT] {
        unsafe { &*(&self.0 as *const _ as *const [i32; PIXEL_COUNT]) }
    }

    #[inline(always)]
    pub fn as_i32_mut(&mut self) -> &mut [i32; PIXEL_COUNT] {
        unsafe { &mut *(&mut self.0 as *mut _ as *mut [i32; PIXEL_COUNT]) }
    }
}

pub trait VMRegister: Copy + Sized + 'static {
    const SIZE: usize;

    fn as_f32(&self) -> &[f32];
    fn as_f32_mut(&mut self) -> &mut [f32];
    fn as_i32(&self) -> &[i32];
    fn as_i32_mut(&mut self) -> &mut [i32];
}

impl VMRegister for VMSlot {
    const SIZE: usize = 1;

    #[inline(always)]
    fn as_f32(&self) -> &[f32] {
        unsafe { &*(&self.float as *const _ as *const [f32; 1]) }
    }

    #[inline(always)]
    fn as_f32_mut(&mut self) -> &mut [f32] {
        unsafe { &mut *(&mut self.float as *mut _ as *mut [f32; 1]) }
    }

    #[inline(always)]
    fn as_i32(&self) -> &[i32] {
        unsafe { &*(&self.int as *const _ as *const [i32; 1]) }
    }

    #[inline(always)]
    fn as_i32_mut(&mut self) -> &mut [i32] {
        unsafe { &mut *(&mut self.int as *mut _ as *mut [i32; 1]) }
    }
}

impl VMRegister for VMTile {
    const SIZE: usize = TILE_SIZE;

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
