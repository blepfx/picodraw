use crate::{graph::Graph, Bounds, ImageData, ShaderData, ShaderDataWriter, Size};

pub trait Context {
    fn create_texture_render(&mut self) -> RenderTexture;
    fn delete_texture_render(&mut self, id: RenderTexture) -> bool;

    fn create_texture_static(&mut self, data: ImageData) -> Texture;
    fn delete_texture_static(&mut self, id: Texture) -> bool;

    fn create_shader(&mut self, graph: Graph) -> Shader;
    fn delete_shader(&mut self, id: Shader) -> bool;

    fn draw(&mut self, buffer: &CommandBuffer);
}

#[derive(Clone, Copy, Debug)]
pub enum Command {
    SetRenderTarget { texture: Option<RenderTexture>, size: Size },
    ClearBuffer { bounds: Bounds },
    BeginQuad { shader: Shader, bounds: Bounds },
    EndQuad,

    WriteF32(f32),
    WriteI32(i32),
    WriteStaticTexture(Texture),
    WriteRenderTexture(RenderTexture),
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct Shader(pub u64);
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct Texture(pub u64);
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct RenderTexture(pub u64);

#[derive(Clone, Debug, Default)]
pub struct CommandBuffer {
    commands: Vec<Command>,
}

pub struct CommandBufferFrame<'a> {
    owner: &'a mut CommandBuffer,
    frame: Size,
}

pub struct CommandBufferQuad<'a> {
    owner: &'a mut CommandBuffer,
    frame: Size,
    quad: Bounds,
}

impl CommandBuffer {
    pub fn reset_commands(&mut self) {
        self.commands.clear();
    }

    pub fn list_commands(&self) -> &[Command] {
        &self.commands
    }

    pub fn extend_commands(&mut self, other: &mut Self) {
        self.commands.extend(other.commands.drain(..));
    }

    pub fn begin_buffer(&mut self, buffer: RenderTexture, size: impl Into<Size>) -> CommandBufferFrame<'_> {
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

    pub fn begin_screen(&mut self, size: impl Into<Size>) -> CommandBufferFrame<'_> {
        let size = size.into();
        self.commands.push(Command::SetRenderTarget { texture: None, size });
        CommandBufferFrame {
            owner: self,
            frame: size,
        }
    }
}

impl<'a> CommandBufferFrame<'a> {
    pub fn clear(&mut self, bounds: impl Into<Bounds>) -> &mut Self {
        let bounds = bounds.into();
        self.owner.commands.push(Command::ClearBuffer { bounds });
        self
    }

    pub fn begin_quad(&mut self, shader: Shader, bounds: impl Into<Bounds>) -> CommandBufferQuad<'_> {
        let bounds = bounds.into();
        self.owner.commands.push(Command::BeginQuad { shader, bounds });
        CommandBufferQuad {
            owner: &mut self.owner,
            frame: self.frame,
            quad: bounds,
        }
    }
}

impl<'a> CommandBufferQuad<'a> {
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
        self.owner.commands.push(Command::WriteI32(x));
    }

    fn write_f32(&mut self, x: f32) {
        self.owner.commands.push(Command::WriteF32(x));
    }

    fn write_texture_static(&mut self, texture: Texture) {
        self.owner.commands.push(Command::WriteStaticTexture(texture));
    }

    fn write_texture_render(&mut self, texture: RenderTexture) {
        self.owner.commands.push(Command::WriteRenderTexture(texture));
    }

    fn resolution(&self) -> Size {
        self.frame
    }

    fn quad_bounds(&self) -> Bounds {
        self.quad
    }
}
