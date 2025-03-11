/// Pixel format of an image.
#[derive(Clone, Copy, Debug)]
pub enum ImageFormat {
    R8,
    RGB8,
    RGBA8,
}

/// Image/texture data. Used for uploading static textures to the backend.
#[derive(Clone, Copy, Debug)]
pub struct ImageData<'a> {
    pub width: u32,
    pub height: u32,
    pub format: ImageFormat,
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

    pub fn is_empty(&self) -> bool {
        self.left >= self.right || self.top >= self.bottom
    }

    pub fn intersect(&self, other: Self) -> Self {
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

    pub fn union(&self, other: Self) -> Self {
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
    fn from(value: [u32; 2]) -> Self {
        Self {
            width: value[0],
            height: value[1],
        }
    }
}

impl From<[u32; 4]> for Bounds {
    fn from(value: [u32; 4]) -> Self {
        Self {
            left: value[0],
            right: value[2],
            top: value[1],
            bottom: value[3],
        }
    }
}
