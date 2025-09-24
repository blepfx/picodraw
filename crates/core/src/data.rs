/// Pixel format of an image.
#[derive(Clone, Copy, Debug)]
pub enum TextureFormat {
    R8,
    RGB8,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TextureFilter {
    Linear,
    Nearest,
}

/// Texture data. Used for uploading static textures to the backend.
#[derive(Clone, Copy, Debug)]
pub struct TextureData<'a> {
    pub bounds: Bounds,
    pub format: TextureFormat,
    pub data: &'a [u8],
}

/// A struct representing a size in physical pixels.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

/// A struct representing an axis aligned rectangle in physical pixels with origin in the top left corner.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct Bounds {
    pub left: u32,
    pub right: u32,
    pub top: u32,
    pub bottom: u32,
}

impl Bounds {
    /// Get the size of the rectangle
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

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.left >= self.right || self.top >= self.bottom
    }

    #[inline]
    pub fn contains(&self, other: impl Into<Self>) -> bool {
        let other = other.into();
        self.left <= other.left && self.right >= other.right && self.top <= other.top && self.bottom >= other.bottom
    }

    #[inline]
    pub fn intersects(&self, other: impl Into<Self>) -> bool {
        !self.intersect(other).is_empty()
    }

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
