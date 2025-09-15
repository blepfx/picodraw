use crate::{Bounds, Graph, Size, TextureData, TextureFormat};
use std::fmt::{self, Display};

/// The heart of `picodraw`.
///
/// Context is used to interact with the rendering backend.
pub trait Context {
    /// Create a texture of the given size and returns its ID. See [`TextureId`] for more info.
    ///
    /// If you want to delete the texture, you should call [`Context::delete_texture`] with the returned ID.
    ///
    /// To upload data from the CPU, use [`Context::upload_texture`].
    fn create_texture(&mut self, size: Size, format: TextureFormat) -> TextureId;

    /// Delete a texture by its ID.
    fn delete_texture(&mut self, id: TextureId) -> bool;

    /// Upload texture data from the CPU and put it into a subregion of a texture object.
    fn upload_texture(&mut self, id: TextureId, data: TextureData) -> bool;

    /// Create a shader from the given shader graph and returns its ID. See [`ShaderId`] for more info.
    ///
    /// If you want to delete the shader, you should call [`Context::delete_shader`] with the returned ID.
    fn create_shader(&mut self, graph: Graph) -> ShaderId;

    /// Delete a shader by its ID.
    fn delete_shader(&mut self, id: ShaderId) -> bool;

    /// Execute a list of draw commands on the backend
    fn draw_screen(&mut self, commands: &[Command]) -> Result<(), DrawError>;

    /// Execute a list of draw commands on the backend
    fn draw_texture(&mut self, target: TextureId, commands: &[Command]) -> Result<(), DrawError>;
}

/// A single draw command.
#[derive(Clone, Copy, Debug)]
pub enum Command {
    /// Reset a region of the target to the initial color (#0000)
    Clear(Bounds),

    /// Begin rendering a single quad.
    Begin(Bounds, ShaderId),

    /// Add data to the current quad. Should be sandwiched between [`Command::Begin`] and [`Command::End`] commands.
    Data(QuadData),

    /// Close the scope of a single quad.
    /// Must be preceded by a [`Command::Begin`] command.
    End,
}

#[derive(Clone, Copy, Debug)]
pub enum QuadData {
    Float(f32),
    Int(i32),
    Texture(TextureId),
}

/// Shader.
///
/// A shader is a program that is executed on the backend.
/// It is represented by a computation graph ([`Graph`]) that computes a pixel color based on it's position, frame resolution and other data.
/// It's possible to send arbitrary data to a shader using the [`ShaderData`](crate::ShaderData) mechanism.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct ShaderId(pub u64);

/// Static texture.
///
/// A texture is a 2D image that can be sampled in shaders.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct TextureId(pub u64);

/// An error that is occured while drawing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawError {
    InvalidShader,
    InvalidTexture,
    InvalidTarget,
    TargetInUse,
    MalformedStream,
}

impl Display for DrawError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DrawError::InvalidShader => write!(f, "command stream contains an invalid shader reference"),
            DrawError::InvalidTexture => write!(f, "command stream contains an invalid source texture"),
            DrawError::InvalidTarget => write!(f, "passed render target is not valid"),
            DrawError::TargetInUse => write!(f, "attempt to use the destination render target as a texture"),
            DrawError::MalformedStream => write!(f, "command stream is malformed"),
        }
    }
}
