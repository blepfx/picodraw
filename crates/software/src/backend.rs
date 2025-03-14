use crate::{DispatchBuffer, Dispatcher, VMSlot, util::ThreadPool, vm::CompiledShader};
use bumpalo::Bump;
use picodraw_core::{
    Bounds, Command, CommandBuffer, Context, Graph, ImageData, RenderTexture, Shader, Texture,
};
use slotmap::{DefaultKey, Key, KeyData, SlotMap};

pub struct SoftwareBackend {
    shaders: SlotMap<DefaultKey, CompiledShader>,
    textures: SlotMap<DefaultKey, ()>,
    buffers: SlotMap<DefaultKey, ()>,

    arena: Bump,
    thread_pool: ThreadPool,
}

pub struct SoftwareContext<'a> {
    owner: &'a mut SoftwareBackend,
}

impl SoftwareBackend {
    pub fn new() -> Self {
        Self {
            arena: Bump::new(),
            thread_pool: ThreadPool::new(),

            shaders: SlotMap::new(),
            textures: SlotMap::new(),
            buffers: SlotMap::new(),
        }
    }

    pub fn begin<'a>(&'a mut self) -> SoftwareContext<'a> {
        SoftwareContext { owner: self }
    }
}

impl<'a> Context for SoftwareContext<'a> {
    fn create_texture_render(&mut self) -> RenderTexture {
        todo!()
    }

    fn delete_texture_render(&mut self, _id: RenderTexture) -> bool {
        todo!()
    }

    fn create_texture_static(&mut self, _data: ImageData) -> Texture {
        todo!()
    }

    fn delete_texture_static(&mut self, _id: Texture) -> bool {
        todo!()
    }

    fn create_shader(&mut self, graph: Graph) -> Shader {
        let compiled = CompiledShader::compile(&self.owner.arena, &graph);
        let key = self.owner.shaders.insert(compiled);
        self.owner.arena.reset();

        Shader(key.data().as_ffi())
    }

    fn delete_shader(&mut self, id: Shader) -> bool {
        self.owner
            .shaders
            .remove(KeyData::from_ffi(id.0).into())
            .is_some()
    }

    fn draw(&mut self, buffer: &CommandBuffer) {
        let mut dispatcher = Dispatcher::new(&self.owner.arena, DispatchBuffer {
            buffer: &mut [],
            width: 0,
            height: 0,
            bounds: Bounds {
                top: 0,
                left: 0,
                bottom: 0,
                right: 0,
            },
        });

        for command in buffer.list_commands() {
            match command {
                Command::SetRenderTarget { texture, size } => {
                    dispatcher.dispatch(&mut self.owner.thread_pool);
                    self.owner.arena.reset();
                    dispatcher = Dispatcher::new(&self.owner.arena, DispatchBuffer {
                        buffer: &mut [], //TODO: guh!!!!
                        width: size.width,
                        height: size.height,
                        bounds: Bounds {
                            left: 0,
                            right: size.width,
                            top: 0,
                            bottom: size.height,
                        },
                    });
                }
                Command::ClearBuffer { bounds } => {
                    dispatcher.write_clear(*bounds);
                }
                Command::BeginQuad { shader, bounds } => {
                    let shader = self
                        .owner
                        .shaders
                        .get(KeyData::from_ffi(shader.0).into())
                        .expect("unknown shader id");

                    dispatcher.write_start(*bounds, &shader);
                }
                Command::EndQuad => {
                    dispatcher.write_end();
                }
                Command::WriteFloat(x) => {
                    dispatcher.write_data(&[VMSlot { float: *x }]);
                }
                Command::WriteInt(x) => {
                    dispatcher.write_data(&[VMSlot { int: *x }]);
                }
                Command::WriteStaticTexture(_) => {
                    todo!()
                }
                Command::WriteRenderTexture(_) => {
                    todo!()
                }
            }
        }

        dispatcher.dispatch(&mut self.owner.thread_pool);
    }
}
