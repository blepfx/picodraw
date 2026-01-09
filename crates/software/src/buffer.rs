use picodraw_core::{Color, TextureError, TextureFilter, TextureFormat};
use std::{
    marker::PhantomData,
    ops::{Deref, Index, IndexMut},
};

/// An owned 2D image buffer of colors (RGBA 32-bit).
///
/// See [`Color`] for more information about the color format.
#[derive(Clone)]
pub struct Buffer {
    data: Box<[Color]>,
    width: usize,
    height: usize,
}

/// Represents a read-only view for an image buffer.
#[derive(Clone, Copy)]
pub struct BufferRef<'a> {
    data: *const Color,
    width: usize,
    height: usize,
    stride: usize,
    phantom: PhantomData<&'a [Color]>,
}

/// Represents a mutable view for an image buffer.
#[derive(Default)]
pub struct BufferMut<'a>(BufferRef<'a>);

unsafe impl Send for BufferRef<'_> {}
unsafe impl Sync for BufferRef<'_> {}

impl Buffer {
    /// Create a new buffer with the given dimensions filled with transparent (#0000) pixels
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            data: vec![Color::default(); width * height].into_boxed_slice(),
            width,
            height,
        }
    }

    /// Resize the buffer to the given dimensions.
    /// The contents will be cleared.
    pub fn resize(&mut self, width: usize, height: usize) {
        let mut data = std::mem::replace(&mut self.data, Box::new([])).into_vec();
        data.resize(width * height, Color::default());
        data.fill(Color::default());
        self.data = data.into_boxed_slice();
        self.width = width;
        self.height = height;
    }

    /// Get the width of the buffer
    pub fn width(&self) -> usize {
        self.width
    }

    /// Get the height of the buffer
    pub fn height(&self) -> usize {
        self.height
    }

    /// Get a read-only view of the buffer
    pub fn as_ref(&self) -> BufferRef<'_> {
        BufferRef::from_slice(&self.data, self.width, self.height)
    }

    /// Get a mutable view of the buffer
    pub fn as_mut(&mut self) -> BufferMut<'_> {
        BufferMut::from_slice(&mut self.data, self.width, self.height)
    }
}

impl<'a> BufferRef<'a> {
    /// Create a buffer view from a row-major slice
    pub fn from_slice(data: &'a [Color], width: usize, height: usize) -> Self {
        Self {
            data: data.as_ptr(),
            width,
            height,
            stride: width,
            phantom: PhantomData,
        }
    }

    /// Create a buffer view from raw pointer and dimensions
    /// # Safety
    /// The caller must ensure that the provided data pointer is valid for
    /// the given width, height, and stride.
    pub unsafe fn from_raw_parts(data: *const Color, width: usize, height: usize, stride: usize) -> Self {
        Self {
            data,
            width,
            height,
            stride,
            phantom: PhantomData,
        }
    }

    /// Decompose the buffer view into raw parts
    pub fn into_raw_parts(self) -> (*const Color, usize, usize, usize) {
        (self.data, self.width, self.height, self.stride)
    }

    /// Get the width of the buffer view
    pub fn width(&self) -> usize {
        self.width
    }

    /// Get the height of the buffer view
    pub fn height(&self) -> usize {
        self.height
    }

    /// Get a reference to a subregion of the view
    pub fn subregion(&self, x: usize, y: usize, width: usize, height: usize) -> Self {
        let width = width.min(self.width - x);
        let height = height.min(self.height - y);

        Self {
            data: if width == 0 || height == 0 {
                std::ptr::null()
            } else {
                unsafe { self.data.add(y * self.stride + x) }
            },

            width,
            height,
            stride: self.stride,
            phantom: PhantomData,
        }
    }

    /// Sample the buffer at the given coordinates using the specified texture filter
    #[inline]
    pub fn sample(&self, x: f32, y: f32, filter: TextureFilter) -> Color {
        #[inline]
        fn sample_nearest(buffer: BufferRef, x: usize, y: usize) -> Color {
            if buffer.width == 0 || buffer.height == 0 {
                return Color::default();
            }

            let x = x.min(buffer.width - 1);
            let y = y.min(buffer.height - 1);
            buffer[(x, y)]
        }

        match filter {
            TextureFilter::Nearest => sample_nearest(*self, x as usize, y as usize),
            TextureFilter::Linear => {
                let (x, y) = (x - 0.5, y - 0.5);

                let p00 = sample_nearest(*self, x as usize, y as usize);
                let p10 = sample_nearest(*self, x as usize + 1, y as usize);
                let p01 = sample_nearest(*self, x as usize, y as usize + 1);
                let p11 = sample_nearest(*self, x as usize + 1, y as usize + 1);

                let x0 = (x.fract() * 256.0) as u8;
                let y0 = (y.fract() * 256.0) as u8;

                let a = p00.lerp(p10, x0);
                let b = p01.lerp(p11, x0);

                a.lerp(b, y0)
            }
        }
    }
}

impl<'a> BufferMut<'a> {
    /// Create a mutable buffer view from a row-major slice
    pub fn from_slice(data: &'a mut [Color], width: usize, height: usize) -> Self {
        Self(BufferRef {
            data: data.as_mut_ptr(),
            width,
            height,
            stride: width,
            phantom: PhantomData,
        })
    }

    /// Create a mutable buffer view from raw pointer and dimensions
    /// # Safety
    /// The caller must ensure that the provided data pointer is valid for
    /// the given width, height, and stride.
    pub unsafe fn from_raw_parts(data: *mut Color, width: usize, height: usize, stride: usize) -> Self {
        debug_assert!(!data.is_null());

        Self(BufferRef {
            data,
            width,
            height,
            stride,
            phantom: PhantomData,
        })
    }

    /// Decompose the mutable buffer view into raw parts
    pub fn into_raw_parts(self) -> (*mut Color, usize, usize, usize) {
        (self.0.data as *mut Color, self.0.width, self.0.height, self.0.stride)
    }

    /// Reborrow the buffer for a shorter lifetime
    pub fn reborrow(&mut self) -> Self {
        Self(BufferRef {
            data: self.0.data,
            width: self.0.width,
            height: self.0.height,
            stride: self.0.stride,
            phantom: PhantomData,
        })
    }

    /// Get a mutable reference to a subregion of the buffer
    pub fn subregion_mut(&mut self, x: usize, y: usize, width: usize, height: usize) -> Self {
        Self(self.subregion(x, y, width, height))
    }

    /// Load pixel data into the buffer from raw image data of the specified format.
    ///
    /// # Errors
    /// - [`TextureError::MalformedData`]: If the provided data length does not match the expected size for the given width, height, and format.
    pub fn unpack_data(
        &mut self,
        width: usize,
        height: usize,
        format: TextureFormat,
        data: &[u8],
    ) -> Result<(), TextureError> {
        if width * height * format.bytes_per_pixel() != data.len() {
            return Err(TextureError::MalformedData);
        }

        match format {
            TextureFormat::RGBA8 => {
                for y in 0..self.height.min(height) {
                    for x in 0..self.width.min(width) {
                        let offset = (y * width + x) * 4;
                        self[(x, y)] = Color {
                            r: data[offset],
                            g: data[offset + 1],
                            b: data[offset + 2],
                            a: data[offset + 3],
                        };
                    }
                }
            }

            TextureFormat::RGB8 => {
                for y in 0..self.height.min(height) {
                    for x in 0..self.width.min(width) {
                        let offset = (y * width + x) * 3;
                        self[(x, y)] = Color {
                            r: data[offset],
                            g: data[offset + 1],
                            b: data[offset + 2],
                            a: 0xFF,
                        };
                    }
                }
            }

            TextureFormat::R8 => {
                for y in 0..self.height.min(height) {
                    for x in 0..self.width.min(width) {
                        let offset = y * width + x;
                        self[(x, y)] = Color {
                            r: data[offset],
                            g: 0,
                            b: 0,
                            a: 0xFF,
                        };
                    }
                }
            }
        }

        Ok(())
    }
}

impl<'a> Index<(usize, usize)> for BufferRef<'a> {
    type Output = Color;

    #[inline(always)]
    fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
        assert!(x < self.width);
        assert!(y < self.height);

        unsafe { &*self.data.add(y * self.stride + x) }
    }
}

impl<'a> Index<(usize, usize)> for BufferMut<'a> {
    type Output = Color;

    #[inline(always)]
    fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
        &self.0[(x, y)]
    }
}

impl<'a> IndexMut<(usize, usize)> for BufferMut<'a> {
    #[inline(always)]
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut Self::Output {
        assert!(x < self.0.width);
        assert!(y < self.0.height);

        unsafe { &mut *(self.0.data as *mut Color).add(y * self.0.stride + x) }
    }
}

impl<'a> Deref for BufferMut<'a> {
    type Target = BufferRef<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> Default for BufferRef<'a> {
    fn default() -> Self {
        Self::from_slice(&[], 0, 0)
    }
}
