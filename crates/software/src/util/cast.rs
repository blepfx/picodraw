use std::slice::{from_raw_parts, from_raw_parts_mut};

pub unsafe trait Pod: Sized + Copy + 'static {
    #[inline(always)]
    fn zeroed() -> Self {
        unsafe { std::mem::zeroed() }
    }

    #[inline(always)]
    fn cast_ref<T: Pod>(&self) -> &T {
        const {
            assert!(align_of::<Self>() >= align_of::<T>());
            assert!(size_of::<Self>() == size_of::<T>());
        }

        unsafe { &*(self as *const Self as *const T) }
    }

    #[inline(always)]
    fn cast_mut<T: Pod>(&mut self) -> &mut T {
        const {
            assert!(align_of::<Self>() >= align_of::<T>());
            assert!(size_of::<Self>() == size_of::<T>());
        }

        unsafe { &mut *(self as *mut Self as *mut T) }
    }

    #[inline(always)]
    fn cast_slice<T: Pod>(slice: &[Self]) -> &[T] {
        const {
            assert!(align_of::<Self>() >= align_of::<T>());
            assert!(size_of::<Self>().is_multiple_of(size_of::<T>()));
        }

        let new_len = if size_of::<Self>() == size_of::<T>() {
            slice.len()
        } else {
            slice.len() * (size_of::<Self>() / size_of::<T>())
        };

        unsafe { from_raw_parts(slice.as_ptr() as *const T, new_len) }
    }

    #[inline(always)]
    fn cast_slice_mut<T: Pod>(slice: &mut [Self]) -> &mut [T] {
        const {
            assert!(align_of::<Self>() >= align_of::<T>());
            assert!(size_of::<Self>().is_multiple_of(size_of::<T>()));
        }

        let new_len = if size_of::<Self>() == size_of::<T>() {
            slice.len()
        } else {
            slice.len() * (size_of::<Self>() / size_of::<T>())
        };

        unsafe { from_raw_parts_mut(slice.as_mut_ptr() as *mut T, new_len) }
    }
}

unsafe impl Pod for i32 {}
unsafe impl Pod for f32 {}
unsafe impl<const N: usize, T: Pod> Pod for [T; N] {}
