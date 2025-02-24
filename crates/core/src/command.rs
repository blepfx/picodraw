use crate::{Bounds, ImageData, ShaderData, ShaderDataWriter, Size, graph::Graph};

/// The heart of `picodraw`.
///
/// Context is used to interact with the rendering backend.
pub trait Context {
    /// Create a dynamic render texture and returns its ID. See [`RenderTexture`] for more info.
    ///
    /// If you want to delete the render texture, you should call [`Context::delete_texture_render`] with the returned ID.
    fn create_texture_render(&mut self) -> RenderTexture;

    /// Delete a dynamic render texture by its ID.
    fn delete_texture_render(&mut self, id: RenderTexture) -> bool;

    /// Create a static texture from the given image data and returns its ID. See [`Texture`] for more info.
    ///
    /// If you want to delete the texture, you should call [`Context::delete_texture_static`] with the returned ID.
    fn create_texture_static(&mut self, data: ImageData) -> Texture;

    /// Delete a static texture by its ID.
    fn delete_texture_static(&mut self, id: Texture) -> bool;

    /// Create a shader from the given shader graph and returns its ID. See [`Shader`] for more info.
    ///
    /// If you want to delete the shader, you should call [`Context::delete_shader`] with the returned ID.
    fn create_shader(&mut self, graph: Graph) -> Shader;

    /// Delete a shader by its ID.
    fn delete_shader(&mut self, id: Shader) -> bool;

    /// Execute a list of draw commands on the backend
    fn draw(&mut self, buffer: &CommandBuffer);
}

/// A single draw command.
#[derive(Clone, Copy, Debug)]
pub enum Command {
    SetRenderTarget {
        texture: Option<RenderTexture>,
        size: Size,
    },
    ClearBuffer {
        bounds: Bounds,
    },
    BeginQuad {
        shader: Shader,
        bounds: Bounds,
    },
    EndQuad,

    WriteFloat(f32),
    WriteInt(i32),
    WriteStaticTexture(Texture),
    WriteRenderTexture(RenderTexture),
}

/// Shader.
///
/// A shader is a program that is executed on the backend.
/// It is represented by a computation graph ([`Graph`]) that computes a pixel color based on it's position, frame resolution and other data.
/// It's possible to send arbitrary data to a shader using the [`ShaderData`] mechanism.
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

/// Draw command buffer.
///
/// Used to store a list of draw commands to be executed on the backend.
/// The commands are executed in order they are added to the buffer.
#[derive(Clone, Debug, Default)]
pub struct CommandBuffer {
    commands: Vec<Command>,
}

/// A frame writer for draw command buffer.
///
/// Used to write frame information, like a list of quads to draw or when to clear the frame.
pub struct CommandBufferFrame<'a> {
    owner: &'a mut CommandBuffer,
    frame: Size,
}

/// A quad writer for draw command buffer.
///
/// Used to write quad information, like what data to pass to the shader.
/// Implements [`ShaderDataWriter`]
pub struct CommandBufferQuad<'a> {
    owner: &'a mut CommandBuffer,
    frame: Size,
    quad: Bounds,
}

impl CommandBuffer {
    /// Create a new empty command buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Clear the command buffer and reset it to it's initial state
    pub fn reset_commands(&mut self) {
        self.commands.clear();
    }

    /// List the commands currently stored in the buffer
    pub fn list_commands(&self) -> &[Command] {
        &self.commands
    }

    /// Append a command buffer to the current buffer
    ///
    /// Clears the other buffer
    pub fn extend_commands(&mut self, other: &mut Self) {
        self.commands.extend(other.commands.drain(..));
    }

    /// Record a command to begin rendering to a [`RenderTexture`]
    ///
    /// The buffer contents are preserved unless you call [`CommandBufferFrame::clear`] manually.
    pub fn begin_buffer(
        &mut self,
        buffer: RenderTexture,
        size: impl Into<Size>,
    ) -> CommandBufferFrame<'_> {
        let size = size.into();
        self.commands.push(Command::SetRenderTarget {
            texture: Some(buffer),
            size,
        });
        CommandBufferFrame {
            owner: self,
            frame: size,
        }
    }

    /// Record a command to begin rendering to the screen
    pub fn begin_screen(&mut self, size: impl Into<Size>) -> CommandBufferFrame<'_> {
        let size = size.into();
        self.commands.push(Command::SetRenderTarget {
            texture: None,
            size,
        });
        CommandBufferFrame {
            owner: self,
            frame: size,
        }
    }
}

impl<'a> CommandBufferFrame<'a> {
    /// Record a command to clear (reset every pixel of the region to `#00000000`) a region of the current draw target (screen OR render texture)
    pub fn clear(&mut self, bounds: impl Into<Bounds>) -> &mut Self {
        let bounds = bounds.into();
        self.owner.commands.push(Command::ClearBuffer { bounds });
        self
    }

    /// Record a command to begin rendering a quad
    pub fn begin_quad(
        &mut self,
        shader: Shader,
        bounds: impl Into<Bounds>,
    ) -> CommandBufferQuad<'_> {
        let bounds = bounds.into();
        self.owner
            .commands
            .push(Command::BeginQuad { shader, bounds });
        CommandBufferQuad {
            owner: &mut self.owner,
            frame: self.frame,
            quad: bounds,
        }
    }
}

impl<'a> CommandBufferQuad<'a> {
    /// Record the writes of the [`ShaderData`] to be associated to the current quad.
    ///
    /// The data can be retrieved in the shader graph context using [`shader::io::read`](crate::shader::io::read) or [`ShaderData::read`]
    ///
    /// The data should be read in the same order as it was written, failure to do so may result in backend implementation defined behavior (reading garbage data or panics, it shoult NOT cause _undefined behavior_)
    pub fn write_data<T: ShaderData>(&mut self, data: T) -> &mut Self {
        T::write(&data, self);
        self
    }
}

impl<'a> Drop for CommandBufferQuad<'a> {
    fn drop(&mut self) {
        self.owner.commands.push(Command::EndQuad);
    }
}

impl<'a> ShaderDataWriter for CommandBufferQuad<'a> {
    fn write_i32(&mut self, x: i32) {
        self.owner.commands.push(Command::WriteInt(x));
    }

    fn write_f32(&mut self, x: f32) {
        self.owner.commands.push(Command::WriteFloat(x));
    }

    fn write_texture_static(&mut self, texture: Texture) {
        self.owner
            .commands
            .push(Command::WriteStaticTexture(texture));
    }

    fn write_texture_render(&mut self, texture: RenderTexture) {
        self.owner
            .commands
            .push(Command::WriteRenderTexture(texture));
    }

    fn resolution(&self) -> Size {
        self.frame
    }

    fn quad_bounds(&self) -> Bounds {
        self.quad
    }
}
