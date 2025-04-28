use picodraw_core::{ImageData, ImageFormat, TextureFilter};
use std::{
    marker::PhantomData,
    ops::{Deref, Index, IndexMut},
};

#[derive(Clone)]
pub struct Buffer {
    data: Box<[u32]>,
    width: usize,
    height: usize,
}

#[derive(Clone, Copy)]
pub struct BufferRef<'a> {
    data: *const u32,
    width: usize,
    height: usize,
    stride: usize,
    phantom: PhantomData<&'a [u32]>,
}

pub struct BufferMut<'a>(BufferRef<'a>);

unsafe impl Send for BufferRef<'_> {}
unsafe impl Sync for BufferRef<'_> {}

impl Buffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            data: vec![0; width * height].into_boxed_slice(),
            width,
            height,
        }
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        let mut data = std::mem::replace(&mut self.data, Box::new([])).into_vec();
        data.resize(width * height, 0);
        self.data = data.into_boxed_slice();
        self.width = width;
        self.height = height;
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn as_ref(&self) -> BufferRef {
        BufferRef::from_slice(&self.data, self.width, self.height)
    }

    pub fn as_mut(&mut self) -> BufferMut {
        BufferMut::from_slice(&mut self.data, self.width, self.height)
    }
}

impl<'a> From<ImageData<'a>> for Buffer {
    fn from(data: ImageData) -> Self {
        assert!(
            data.data.len() == data.width as usize * data.height as usize * data.format.bytes_per_pixel(),
            "invalid {:?} data length: {} != {} (width x height x {})",
            data.format,
            data.data.len(),
            data.width as usize * data.height as usize * data.format.bytes_per_pixel(),
            data.format.bytes_per_pixel()
        );

        let mut buffer = Self::new(data.width as usize, data.height as usize);
        match data.format {
            ImageFormat::RGBA8 => {
                let data = data.data.as_ref();
                for y in 0..buffer.height {
                    for x in 0..buffer.width {
                        let offset = (y * buffer.width + x) * 4;
                        buffer.data[y * buffer.width + x] =
                            pack_rgba(data[offset + 0], data[offset + 1], data[offset + 2], data[offset + 3]);
                    }
                }
            }

            ImageFormat::RGB8 => {
                let data = data.data.as_ref();
                for y in 0..buffer.height {
                    for x in 0..buffer.width {
                        let offset = (y * buffer.width + x) * 3;
                        buffer.data[y * buffer.width + x] =
                            pack_rgba(data[offset + 0], data[offset + 1], data[offset + 2], 0xFF);
                    }
                }
            }

            ImageFormat::R8 => {
                let data = data.data.as_ref();
                for y in 0..buffer.height {
                    for x in 0..buffer.width {
                        let offset = y * buffer.width + x;
                        buffer.data[offset] = pack_rgba(data[offset], 0, 0, 0xFF);
                    }
                }
            }
        }

        buffer
    }
}

impl<'a> BufferRef<'a> {
    pub fn from_slice(data: &'a [u32], width: usize, height: usize) -> Self {
        Self {
            data: data.as_ptr(),
            width,
            height,
            stride: width,
            phantom: PhantomData,
        }
    }

    pub unsafe fn from_raw_parts(data: *mut u32, width: usize, height: usize, stride: usize) -> Self {
        Self {
            data,
            width,
            height,
            stride,
            phantom: PhantomData,
        }
    }

    pub fn into_raw_parts(self) -> (*const u32, usize, usize, usize) {
        (self.data as *const u32, self.width, self.height, self.stride)
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

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

    #[inline]
    pub fn sample(&self, x: f32, y: f32, filter: TextureFilter) -> u32 {
        #[inline]
        fn sample_neasert(buffer: BufferRef, x: usize, y: usize) -> u32 {
            if buffer.width == 0 || buffer.height == 0 {
                return 0;
            }

            let x = x.min(buffer.width - 1);
            let y = y.min(buffer.height - 1);
            buffer[(x, y)]
        }

        match filter {
            TextureFilter::Nearest => sample_neasert(*self, x as usize, y as usize),
            TextureFilter::Linear => {
                let lerp = |a: u8, b: u8, x: u8| {
                    let a = a as u16;
                    let b = b as u16;
                    let x = x as u16;
                    ((a * (256 - x) + b * x) / 256) as u8
                };

                let p00 = sample_neasert(*self, x as usize, y as usize).to_ne_bytes();
                let p10 = sample_neasert(*self, x as usize + 1, y as usize).to_ne_bytes();
                let p01 = sample_neasert(*self, x as usize, y as usize + 1).to_ne_bytes();
                let p11 = sample_neasert(*self, x as usize + 1, y as usize + 1).to_ne_bytes();

                let x0 = (x.fract() * 256.0) as u8;
                let y0 = (y.fract() * 256.0) as u8;

                let a = [
                    lerp(p00[0], p10[0], x0),
                    lerp(p00[1], p10[1], x0),
                    lerp(p00[2], p10[2], x0),
                    lerp(p00[3], p10[3], x0),
                ];

                let b = [
                    lerp(p01[0], p11[0], x0),
                    lerp(p01[1], p11[1], x0),
                    lerp(p01[2], p11[2], x0),
                    lerp(p01[3], p11[3], x0),
                ];

                let c = [
                    lerp(a[0], b[0], y0),
                    lerp(a[1], b[1], y0),
                    lerp(a[2], b[2], y0),
                    lerp(a[3], b[3], y0),
                ];

                u32::from_ne_bytes(c)
            }
        }
    }
}

impl<'a> BufferMut<'a> {
    pub fn from_slice(data: &'a mut [u32], width: usize, height: usize) -> Self {
        Self(BufferRef {
            data: data.as_mut_ptr(),
            width,
            height,
            stride: width,
            phantom: PhantomData,
        })
    }

    pub unsafe fn from_raw_parts(data: *mut u32, width: usize, height: usize, stride: usize) -> Self {
        Self(BufferRef {
            data,
            width,
            height,
            stride,
            phantom: PhantomData,
        })
    }

    pub fn into_raw_parts(self) -> (*mut u32, usize, usize, usize) {
        (self.0.data as *mut u32, self.0.width, self.0.height, self.0.stride)
    }

    pub fn reborrow(&mut self) -> Self {
        Self(BufferRef {
            data: self.0.data,
            width: self.0.width,
            height: self.0.height,
            stride: self.0.stride,
            phantom: PhantomData,
        })
    }

    pub fn subregion_mut(&mut self, x: usize, y: usize, width: usize, height: usize) -> Self {
        Self {
            0: self.subregion(x, y, width, height),
        }
    }
}

impl<'a> Index<(usize, usize)> for BufferRef<'a> {
    type Output = u32;

    fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
        assert!(x < self.width);
        assert!(y < self.height);

        unsafe { &*self.data.add(y * self.stride + x) }
    }
}

impl<'a> Index<(usize, usize)> for BufferMut<'a> {
    type Output = u32;

    fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
        &self.0[(x, y)]
    }
}

impl<'a> IndexMut<(usize, usize)> for BufferMut<'a> {
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut Self::Output {
        assert!(x < self.0.width);
        assert!(y < self.0.height);

        unsafe { &mut *(self.0.data as *mut u32).add(y * self.0.stride + x) }
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

impl<'a> Default for BufferMut<'a> {
    fn default() -> Self {
        Self(BufferRef::default())
    }
}

#[inline(always)]
pub fn pack_rgba(r: u8, g: u8, b: u8, a: u8) -> u32 {
    (r as u32) << 16 | (g as u32) << 8 | (b as u32) | ((a as u32) << 24)
}

#[inline(always)]
pub fn unpack_rgba(color: u32) -> (u8, u8, u8, u8) {
    (
        ((color >> 16) & 0xFF) as u8,
        ((color >> 8) & 0xFF) as u8,
        (color & 0xFF) as u8,
        ((color >> 24) & 0xFF) as u8,
    )
}
