use crate::{Bounds, ShaderData, Size, TextureData, TextureFormat};

/// The heart of `picodraw`.
///
/// Context is used to interact with the rendering backend.
pub trait Context: Sized {
    type Shader: 'static;
    type Texture: 'static;

    /// Create a texture of the given size and returns its handle.
    ///
    /// To upload data from the CPU, use [`Context::upload_texture`].
    fn create_texture(&mut self, size: Size, format: TextureFormat) -> Result<Self::Texture, TextureError>;

    /// Upload texture data from the CPU and put it into a subregion of a texture object.
    fn upload_texture(&mut self, texture: &mut Self::Texture, data: TextureData) -> Result<(), TextureError>;

    /// Delete a texture.
    fn delete_texture(&mut self, texture: Self::Texture);

    /// Create a shader from the given shader graph and returns its handle.
    fn create_shader(&mut self, shader: ShaderData) -> Result<Self::Shader, ShaderError>;

    /// Delete a shader.
    fn delete_shader(&mut self, shader: Self::Shader);

    /// Execute a list of draw commands on the backend
    fn draw(
        &mut self,
        target: DrawTarget<Self>,
        f: impl FnOnce(&mut dyn FrameEncoder<Shader = Self::Shader, Texture = Self::Texture>),
    );
}

#[derive(Debug)]
pub enum DrawTarget<'a, C: Context> {
    Screen,
    Texture(&'a mut C::Texture),
}

pub trait FrameEncoder<'a> {
    type Shader: 'static;
    type Texture: 'static;

    fn clear(&mut self, bounds: Bounds);
    fn object(&mut self, shader: &Self::Shader);

    fn add_rect(&mut self, quad: Bounds);
    fn add_data(&mut self, buffer: &[u8]);
    fn add_texture(&mut self, texture: &Self::Texture);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureError {
    OutOfMemory,
    MalformedData,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderError {
    TooComplex,
    MalformedGraph,
}
