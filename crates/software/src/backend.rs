use crate::{
    Dispatcher, VMSlot,
    buffer::{Buffer, BufferMut},
    util::ThreadPool,
    vm::CompiledShader,
};
use bumpalo::Bump;
use picodraw_core::{Command, CommandBuffer, Context, Graph, ImageData, RenderTexture, Shader, Size, Texture};
use slotmap::{DefaultKey, Key, KeyData, SlotMap};

pub struct SoftwareBackend {
    shaders: SlotMap<DefaultKey, CompiledShader>,
    textures: SlotMap<DefaultKey, Buffer>,
    buffers: SlotMap<DefaultKey, Buffer>,

    arena: Bump,
    thread_pool: ThreadPool,
}

pub struct SoftwareContext<'a> {
    owner: &'a mut SoftwareBackend,
    screen: BufferMut<'a>,
}

impl SoftwareBackend {
    pub fn new() -> Self {
        Self {
            arena: Bump::new(),
            thread_pool: ThreadPool::with_threads(1),

            shaders: SlotMap::new(),
            textures: SlotMap::new(),
            buffers: SlotMap::new(),
        }
    }

    pub fn begin<'a>(&'a mut self, screen: BufferMut<'a>) -> SoftwareContext<'a> {
        SoftwareContext { owner: self, screen }
    }
}

impl<'a> Context for SoftwareContext<'a> {
    fn create_texture_render(&mut self) -> RenderTexture {
        let id = self.owner.buffers.insert(Buffer::new(0, 0));
        RenderTexture(id.data().as_ffi())
    }

    fn delete_texture_render(&mut self, id: RenderTexture) -> bool {
        self.owner.buffers.remove(KeyData::from_ffi(id.0).into()).is_some()
    }

    fn create_texture_static(&mut self, data: ImageData) -> Texture {
        let id = self.owner.textures.insert(Buffer::from(data));
        Texture(id.data().as_ffi())
    }

    fn delete_texture_static(&mut self, id: Texture) -> bool {
        self.owner.textures.remove(KeyData::from_ffi(id.0).into()).is_some()
    }

    fn create_shader(&mut self, graph: Graph) -> Shader {
        let compiled = CompiledShader::compile(&self.owner.arena, &graph);
        let key = self.owner.shaders.insert(compiled);
        self.owner.arena.reset();

        Shader(key.data().as_ffi())
    }

    fn delete_shader(&mut self, id: Shader) -> bool {
        self.owner.shaders.remove(KeyData::from_ffi(id.0).into()).is_some()
    }

    fn draw(&mut self, buffer: &CommandBuffer) {
        struct DispatchGroup<'a> {
            dispatcher: Dispatcher<'a>,
            target: Option<RenderTexture>,
            size: Size,
        }

        let mut commands = buffer.list_commands().iter();
        loop {
            let mut dispatch = None;

            while let command = commands.next() {
                match command {
                    Some(Command::SetRenderTarget { texture, size }) => {
                        if dispatch.is_some() {
                            break;
                        }

                        dispatch = Some(DispatchGroup {
                            dispatcher: Dispatcher::new(&self.owner.arena),
                            target: *texture,
                            size: *size,
                        });
                    }
                    Some(Command::ClearBuffer { bounds }) => {
                        dispatch
                            .as_mut()
                            .expect("render target is not set")
                            .dispatcher
                            .write_clear(*bounds);
                    }
                    Some(Command::BeginQuad { shader, bounds }) => {
                        let shader = self
                            .owner
                            .shaders
                            .get(KeyData::from_ffi(shader.0).into())
                            .expect("unknown shader id");

                        dispatch
                            .as_mut()
                            .expect("render target is not set")
                            .dispatcher
                            .write_start(*bounds, &shader);
                    }
                    Some(Command::EndQuad) => {
                        dispatch
                            .as_mut()
                            .expect("render target is not set")
                            .dispatcher
                            .write_end();
                    }
                    Some(Command::WriteFloat(x)) => {
                        dispatch
                            .as_mut()
                            .expect("render target is not set")
                            .dispatcher
                            .write_data(&[VMSlot { float: *x }]);
                    }
                    Some(Command::WriteInt(x)) => {
                        dispatch
                            .as_mut()
                            .expect("render target is not set")
                            .dispatcher
                            .write_data(&[VMSlot { int: *x }]);
                    }
                    Some(Command::WriteStaticTexture(tex)) => {
                        let tex = self
                            .owner
                            .textures
                            .get(KeyData::from_ffi(tex.0).into())
                            .expect("unknown texture id");

                        dispatch
                            .as_mut()
                            .expect("render target is not set")
                            .dispatcher
                            .write_texture(tex.as_ref());
                    }
                    Some(Command::WriteRenderTexture(tex)) => {
                        let tex = self
                            .owner
                            .buffers
                            .get(KeyData::from_ffi(tex.0).into())
                            .expect("unknown render texture id");

                        dispatch
                            .as_mut()
                            .expect("render target is not set")
                            .dispatcher
                            .write_texture(tex.as_ref());
                    }

                    None => {
                        if dispatch.is_some() {
                            break;
                        } else {
                            return;
                        }
                    }
                }
            }

            if let Some(dispatch) = dispatch.take() {
                let target = match dispatch.target {
                    Some(target) => todo!(),
                    None => {
                        assert!(
                            (self.screen.width() == dispatch.size.width as usize)
                                || (self.screen.height() == dispatch.size.height as usize),
                            "screen buffer size mismatch"
                        );

                        self.screen.reborrow()
                    }
                };

                dispatch.dispatcher.dispatch(&mut self.owner.thread_pool, target);
            }

            drop(dispatch);
            self.owner.arena.reset();
        }
    }
}
