use crate::{Bounds, Color, ShaderData, Size, TextureData, TextureFormat};

/// The heart of `picodraw`.
///
/// The `Context` trait is used to interact with the rendering backend.
pub trait Context: Sized {
    /// A handle representing a shader used for drawing.
    type Shader: 'static;

    /// A handle representing a texture used for drawing or as a render target.
    type Texture: 'static;

    /// Create a texture of the given size and returns its handle.
    ///
    /// To upload data from the CPU, use [`Context::upload_texture`].
    ///
    /// # Errors
    /// - [`TextureError::OutOfMemory`]: If the backend ran out of memory (e.g. the requested size is too large).
    fn create_texture(&mut self, size: Size, format: TextureFormat) -> Result<Self::Texture, TextureError>;

    /// Upload texture data from the CPU and put it into a subregion of a texture object.
    ///
    /// # Errors
    /// - [`TextureError::OutOfMemory`]: If the backend ran out of memory (e.g. region is too large).
    /// - [`TextureError::MalformedData`]: If the provided data was malformed.
    fn upload_texture(&mut self, texture: &mut Self::Texture, data: TextureData) -> Result<(), TextureError>;

    /// Delete a texture.
    fn delete_texture(&mut self, texture: Self::Texture);

    /// Create a shader from the given shader graph and returns its handle.
    ///
    /// # Errors
    /// - [`ShaderError::TooComplex`]: If the shader graph is too complex to be compiled. (e.g. uses too many registers)
    /// - [`ShaderError::TooManyTextures`]: If the shader uses more texture units at once than the backend supports.
    fn create_shader(&mut self, shader: &ShaderData) -> Result<Self::Shader, ShaderError>;

    /// Delete a shader.
    fn delete_shader(&mut self, shader: Self::Shader);

    /// Execute a list of draw commands on the backend
    fn draw<'a>(
        &'a mut self,
        target: DrawTarget<'a, Self::Texture>,
        f: impl FnOnce(&mut dyn FrameEncoder<'a, Shader = Self::Shader, Texture = Self::Texture>),
    );
}

/// The target to draw into.
/// Could be either the screen or a texture.
#[derive(Debug)]
pub enum DrawTarget<'a, T> {
    /// Render onto the screen.
    Screen,

    /// Render onto the given texture.
    ///
    /// The texture could not be used for sampling during the same draw call, this is ensured by the borrow checker.
    Texture(&'a mut T),
}

/// A frame encoder is used to encode draw commands for a single frame.
pub trait FrameEncoder<'a> {
    /// The type of shader used for drawing.
    type Shader: 'static;

    /// The type of texture used for drawing.
    type Texture: 'static;

    /// Set every pixel in the region to be undefined. (i.e. arbitrary garbage data)
    ///
    /// The backend is free to assume that the contents of this region are not needed anymore.  
    /// This is useful for optimizing regions that will be fully overwritten later.
    fn invalidate(&mut self, bounds: Bounds);

    /// Set every pixel in the region to the given color.
    ///
    /// Does not do alpha blending, just overwrites the pixels.
    fn fill(&mut self, bounds: Bounds, color: Color);

    /// Emit an **object** to be drawn with the given shader.
    ///
    /// An object is a list of rectangles drawn by the same shader, and an associated data blob and a list of textures to be used by the shader.
    ///
    /// [`FrameEncoder::add_rect`], [`FrameEncoder::add_data`] and [`FrameEncoder::add_texture`] calls
    /// before this call are associated with this object.
    fn draw(&mut self, shader: &'a Self::Shader);

    /// Add a rectangle to be used the subsequent [`FrameEncoder::draw`] call.
    fn add_rect(&mut self, quad: Bounds);

    /// Add a data blob to be used by the subsequent [`FrameEncoder::draw`] call.
    /// It will be appended to the end of the object's data buffer.
    ///
    /// Data could be accessed in the shader by using [`ShaderOp::ReadI32`](crate::ShaderOp::ReadI32), [`trace::int1::read_i32`](crate::trace::int1::read_i32) or similar operations.
    fn add_data(&mut self, buffer: &[u8]);

    /// Add a texture to be used by the subsequent [`FrameEncoder::draw`] call.
    ///
    /// A texture could be accessed in the shader by using [`ShaderOp::TexSample`](crate::ShaderOp::TexSample), [`trace::texture2d::read`](crate::trace::texture2d::read) or similar operations.
    fn add_texture(&mut self, texture: &'a Self::Texture);
}

/// An error that occurred during texture creation or data upload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureError {
    /// The backend ran out of memory.
    OutOfMemory,

    /// The provided data was malformed or incompatible with the provided texture format.
    MalformedData,
}

/// An error that occurred during shader compilation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderError {
    /// The shader graph was too complex to be compiled (too many registers used, etc.)
    TooComplex,
    /// The shader used too many textures, more than texture units that the backend supports.
    TooManyTextures,
}

impl<'a, T> From<&'a mut T> for DrawTarget<'a, T> {
    fn from(texture: &'a mut T) -> Self {
        DrawTarget::Texture(texture)
    }
}
