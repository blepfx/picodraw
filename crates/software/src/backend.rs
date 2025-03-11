use crate::{dispatch::Dispatcher, vm::CompiledShader};
use picodraw_core::{CommandBuffer, Context, Graph, ImageData, RenderTexture, Shader, Texture};
use slotmap::{DefaultKey, Key, KeyData, SlotMap};

pub struct SoftwareBackend {
    shaders: SlotMap<DefaultKey, CompiledShader>,
    dispatcher: Dispatcher,
}

pub struct SoftwareContext<'a> {
    buffer: &'a mut dyn SoftwareBuffer,
    owner: &'a mut SoftwareBackend,
}

pub trait SoftwareBuffer {
    fn as_mut_slice(&mut self) -> &mut [u32];
    fn width(&self) -> u32;
    fn height(&self) -> u32;
}

impl SoftwareBackend {
    pub fn new() -> Self {
        Self {
            shaders: SlotMap::new(),
            dispatcher: Dispatcher::new(),
        }
    }

    pub fn begin<'a>(&'a mut self, buffer: &'a mut dyn SoftwareBuffer) -> SoftwareContext<'a> {
        SoftwareContext {
            buffer,
            owner: self,
        }
    }
}

impl SoftwareBuffer for () {
    fn as_mut_slice(&mut self) -> &mut [u32] {
        &mut []
    }

    fn width(&self) -> u32 {
        0
    }

    fn height(&self) -> u32 {
        0
    }
}

impl<'a> Context for SoftwareContext<'a> {
    fn create_texture_render(&mut self) -> RenderTexture {
        todo!()
    }

    fn delete_texture_render(&mut self, id: RenderTexture) -> bool {
        todo!()
    }

    fn create_texture_static(&mut self, data: ImageData) -> Texture {
        todo!()
    }

    fn delete_texture_static(&mut self, id: Texture) -> bool {
        todo!()
    }

    fn create_shader(&mut self, graph: Graph) -> Shader {
        let compiled = CompiledShader::compile(&graph);
        let key = self.owner.shaders.insert(compiled);
        Shader(key.data().as_ffi())
    }

    fn delete_shader(&mut self, id: Shader) -> bool {
        self.owner
            .shaders
            .remove(KeyData::from_ffi(id.0).into())
            .is_some()
    }

    fn draw(&mut self, buffer: &CommandBuffer) {
        for command in buffer.list_commands() {}
    }
}
