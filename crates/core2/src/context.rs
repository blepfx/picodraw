use crate::{Bounds, ShaderOp, Size, TextureData, TextureFormat};
use std::fmt::{self, Display};

/// The heart of `picodraw`.
///
/// Context is used to interact with the rendering backend.
pub trait Context: Sized {
    type Shader: 'static + Clone;
    type Texture: 'static + Clone;
    type Draw: DrawContext<Self>;

    /// Create a texture of the given size and returns its handle.
    ///
    /// To upload data from the CPU, use [`Context::upload_texture`].
    fn create_texture(&mut self, size: Size, format: TextureFormat) -> Result<Self::Texture, TextureError>;

    /// Upload texture data from the CPU and put it into a subregion of a texture object.
    fn upload_texture(&mut self, id: Self::Texture, data: TextureData) -> Result<Self::Texture, TextureError>;

    /// Create a shader from the given shader graph and returns its handle.
    fn create_shader(&mut self, shader: &[ShaderOp]) -> Result<Self::Shader, ShaderError>;

    /// Execute a list of draw commands on the backend
    fn draw_screen(&mut self, draw: impl FnOnce(Self::Draw)) -> Result<(), DrawError>;

    /// Execute a list of draw commands on the backend
    fn draw_texture(&mut self, target: Self::Texture, draw: impl FnOnce(Self::Draw)) -> Result<(), DrawError>;
}

pub trait DrawContext<C: Context> {
    fn clear(&mut self, bounds: Bounds);
    fn draw(&mut self, shader: C::Shader, quads: &[Bounds], textures: &[C::Texture], buffer: &[u8]);
}

/// An error that has occured while drawing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawError {
    /// Attempt to sample from a texture that is being used as the target as the same time
    TargetInUse,

    /// Command stream contains an invalid [`Context::Shader`]
    InvalidShader,

    /// Command stream contains an invalid [`Context::Texture`]
    InvalidTexture,

    /// Render target passed to [`Context::draw_texture`] is invalid
    InvalidTarget,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureError {
    OutOfMemory,
    MalformedData,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderError {
    MalformedBytecode,
}

impl Display for DrawError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DrawError::TargetInUse => write!(f, "attempt to use the destination render target as a texture"),
            DrawError::InvalidShader => write!(f, "command stream contains an invalid shader reference"),
            DrawError::InvalidTexture => write!(f, "command stream contains an invalid source texture"),
            DrawError::InvalidTarget => write!(f, "passed render target is not valid"),
        }
    }
}
