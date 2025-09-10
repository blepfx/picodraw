use crate::{Bounds, Graph, Size, TextureData};
use std::fmt::{self, Display};

/// The heart of `picodraw`.
///
/// Context is used to interact with the rendering backend.
pub trait Context {
    /// Create a dynamic render texture and returns its ID. See [`RenderTexture`] for more info.
    ///
    /// If you want to delete the render texture, you should call [`Context::delete_texture_render`] with the returned ID.
    fn create_texture_render(&mut self, size: Size) -> RenderTexture;

    /// Delete a dynamic render texture by its ID.
    fn delete_texture_render(&mut self, id: RenderTexture) -> bool;

    /// Create a static texture from the given image data and returns its ID. See [`Texture`] for more info.
    ///
    /// If you want to delete the texture, you should call [`Context::delete_texture_static`] with the returned ID.
    fn create_texture_static(&mut self, data: TextureData) -> Texture;

    /// Delete a static texture by its ID.
    fn delete_texture_static(&mut self, id: Texture) -> bool;

    /// Create a shader from the given shader graph and returns its ID. See [`Shader`] for more info.
    ///
    /// If you want to delete the shader, you should call [`Context::delete_shader`] with the returned ID.
    fn create_shader(&mut self, graph: Graph) -> Shader;

    /// Delete a shader by its ID.
    fn delete_shader(&mut self, id: Shader) -> bool;

    /// Execute a list of draw commands on the backend
    fn draw_screen(&mut self, commands: &[Command]) -> Result<(), DrawError>;

    /// Execute a list of draw commands on the backend
    fn draw_texture(&mut self, target: RenderTexture, commands: &[Command]) -> Result<(), DrawError>;
}

/// A single draw command.
#[derive(Clone, Copy, Debug)]
pub enum Command {
    /// Reset a region of the target to the initial color (#0000)
    Clear(Bounds),

    /// Begin rendering a single quad.
    Begin(Bounds, Shader),

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
    Texture(Texture),
    RenderTexture(RenderTexture),
}

/// Shader.
///
/// A shader is a program that is executed on the backend.
/// It is represented by a computation graph ([`Graph`]) that computes a pixel color based on it's position, frame resolution and other data.
/// It's possible to send arbitrary data to a shader using the [`ShaderData`](crate::ShaderData) mechanism.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct Shader(pub u64);

/// Static texture.
///
/// A texture is a 2D image that can be sampled in shaders.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct Texture(pub u64);

/// Dynamic render texture.
///
/// A render texture is an off-screen buffer you can render to and use it as a texture later.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct RenderTexture(pub u64);

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
