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
    ObjectBegin(ShaderId),

    ObjectRect(Bounds),

    /// Add data to the current quad. Should be sandwiched between [`Command::ObjectBegin`] and [`Command::ObjectEnd`] commands.
    ObjectData(ObjectData),

    /// Close the scope of a single quad.
    /// Must be preceded by a [`Command::ObjectBegin`] command.
    ObjectEnd,
}

#[derive(Clone, Copy, Debug)]
pub enum ObjectData {
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

/// Texture.
///
/// A texture is a 2D image that can be sampled in shaders.
/// An empty texture can be created by calling [`Context::create_texture`],
/// which can be later populated by either drawing onto it with [`Context::draw_texture`]
/// or uploading texture data from an external source via [`Context::upload_texture`]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct TextureId(pub u64);

/// An error that has occured while drawing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawError {
    /// Command stream contains an invalid [`ShaderId`]
    InvalidShader,

    /// Command stream contains an invalid [`TextureId`]
    InvalidTexture,

    /// Render target passed to [`Context::draw_texture`] is invalid
    InvalidTarget,

    /// Object data passed via [`Command::Data`] does not correspond to the input schema expected by the shader
    InvalidObjectData,

    /// Attempt to sample from a texture that is being used as the target as the same time
    TargetInUse,

    /// The command stream is malformed. See [`Command::Begin`] and [`Command::End`] for more info
    MalformedStream,
}

impl Display for DrawError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DrawError::InvalidShader => write!(f, "command stream contains an invalid shader reference"),
            DrawError::InvalidTexture => write!(f, "command stream contains an invalid source texture"),
            DrawError::InvalidTarget => write!(f, "passed render target is not valid"),
            DrawError::InvalidObjectData => {
                write!(f, "passed quad data does not correspond to the used shader's schema")
            }
            DrawError::TargetInUse => write!(f, "attempt to use the destination render target as a texture"),
            DrawError::MalformedStream => write!(f, "command stream is malformed"),
        }
    }
}
