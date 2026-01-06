/// A color represented as RGBA components in SRGB space.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Default)]
#[repr(C)]
pub struct Color {
    /// Red component.
    pub r: u8,
    /// Green component.
    pub g: u8,
    /// Blue component.
    pub b: u8,
    /// Alpha (transparency) component.
    pub a: u8,
}

impl Color {
    /// Linearly interpolate between this color and another color by a given factor.
    ///
    /// `t` is in the range `[0, 255]`, where 0 returns `self` and 255 returns `other`.
    #[inline]
    pub fn lerp(self, other: Color, t: u8) -> Color {
        let lerp = |a: u8, b: u8, x: u8| {
            let a = a as u16;
            let b = b as u16;
            let x = x as u16;
            ((a * (256 - x) + b * x) / 256) as u8
        };

        Color {
            r: lerp(self.r, other.r, t),
            g: lerp(self.g, other.g, t),
            b: lerp(self.b, other.b, t),
            a: lerp(self.a, other.a, t),
        }
    }
}

/// Pixel format of an image.
#[derive(Clone, Copy, Debug)]
pub enum TextureFormat {
    /// 8-bit single channel format TODO: clarify the shader output when sampling
    R8,
    /// 8-bit RGB format, assumes alpha of 255.
    RGB8,
    /// 8-bit RGBA format.
    RGBA8,
}

impl TextureFormat {
    /// Get the number of bytes per pixel for this format.
    pub fn bytes_per_pixel(&self) -> usize {
        match self {
            TextureFormat::R8 => 1,
            TextureFormat::RGB8 => 3,
            TextureFormat::RGBA8 => 4,
        }
    }
}

/// Texture filtering mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TextureFilter {
    /// Linear filtering.
    Linear,

    /// Nearest neighbor filtering (point sampling).
    Nearest,
}

/// A single channel of a texture.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum TextureChannel {
    /// Red channel.
    Red,

    /// Green channel.
    Green,

    /// Blue channel.
    Blue,

    /// Alpha channel.
    Alpha,
}

/// Texture data. Used for uploading static textures to the backend.
#[derive(Clone, Copy, Debug)]
pub struct TextureData<'a> {
    /// The region of the texture to upload data to.
    pub bounds: Bounds,

    /// The format of the texture data.
    pub format: TextureFormat,

    /// The raw pixel data.
    pub data: &'a [u8],
}

/// A struct representing a size in physical pixels.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct Size {
    /// Width, in physical pixels.
    pub width: u32,

    /// Height, in physical pixels.
    pub height: u32,
}

/// A struct representing an axis aligned rectangle in physical pixels with origin in the top left corner.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct Bounds {
    /// Left coordinate (min X) of the rectangle.
    pub left: u32,

    /// Right coordinate (max X) of the rectangle.
    pub right: u32,

    /// Top coordinate (min Y) of the rectangle.
    pub top: u32,

    /// Bottom coordinate (max Y) of the rectangle.
    pub bottom: u32,
}

impl Bounds {
    /// Get the size of the rectangle.
    ///
    /// Negative sizes are clamped to zero.
    pub fn size(&self) -> Size {
        Size {
            width: self.right.saturating_sub(self.left),
            height: self.bottom.saturating_sub(self.top),
        }
    }

    /// Get the width of the rectangle
    pub fn width(&self) -> u32 {
        self.size().width
    }

    /// Get the height of the rectangle
    pub fn height(&self) -> u32 {
        self.size().height
    }

    /// Check if the rectangle is empty (has zero or negative area)
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.left >= self.right || self.top >= self.bottom
    }

    /// Check if this rectangle fully contains another rectangle (touching edges is considered contained)
    #[inline]
    pub fn contains(&self, other: impl Into<Self>) -> bool {
        let other = other.into();
        self.left <= other.left && self.right >= other.right && self.top <= other.top && self.bottom >= other.bottom
    }

    /// Check if this rectangle intersects with another rectangle
    #[inline]
    pub fn intersects(&self, other: impl Into<Self>) -> bool {
        !self.intersect(other).is_empty()
    }

    /// Get the intersection of this rectangle with another rectangle
    ///
    /// If the rectangles do not intersect, an empty (negative area) rectangle is returned.
    #[inline]
    pub fn intersect(&self, other: impl Into<Self>) -> Self {
        let other = other.into();
        let left = self.left.max(other.left);
        let right = self.right.min(other.right);
        let top = self.top.max(other.top);
        let bottom = self.bottom.min(other.bottom);

        Self {
            left,
            right,
            top,
            bottom,
        }
    }

    /// Get the union of this rectangle with another rectangle
    #[inline]
    pub fn union(&self, other: impl Into<Self>) -> Self {
        let other = other.into();
        let left = self.left.min(other.left);
        let right = self.right.max(other.right);
        let top = self.top.min(other.top);
        let bottom = self.bottom.max(other.bottom);

        Self {
            left,
            right,
            top,
            bottom,
        }
    }

    /// Offset the rectangle by the given amounts in the X and Y directions.
    #[inline]
    pub fn offset(&self, x: i32, y: i32) -> Self {
        Self {
            left: self.left.saturating_add_signed(x),
            right: self.right.saturating_add_signed(x),
            top: self.top.saturating_add_signed(y),
            bottom: self.bottom.saturating_add_signed(y),
        }
    }
}

impl From<[u32; 2]> for Size {
    #[inline]
    fn from([width, height]: [u32; 2]) -> Self {
        Self { width, height }
    }
}

impl From<(u32, u32)> for Size {
    #[inline]
    fn from((width, height): (u32, u32)) -> Self {
        Self { width, height }
    }
}

impl From<[u32; 4]> for Bounds {
    #[inline]
    fn from([left, top, right, bottom]: [u32; 4]) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }
}

impl From<(u32, u32, u32, u32)> for Bounds {
    #[inline]
    fn from((left, top, right, bottom): (u32, u32, u32, u32)) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }
}

impl From<[i32; 4]> for Bounds {
    #[inline]
    fn from([left, top, right, bottom]: [i32; 4]) -> Self {
        Self {
            left: left.try_into().unwrap_or_default(),
            right: right.try_into().unwrap_or_default(),
            top: top.try_into().unwrap_or_default(),
            bottom: bottom.try_into().unwrap_or_default(),
        }
    }
}

impl From<(i32, i32, i32, i32)> for Bounds {
    #[inline]
    fn from((left, top, right, bottom): (i32, i32, i32, i32)) -> Self {
        Self::from([left, top, right, bottom])
    }
}
