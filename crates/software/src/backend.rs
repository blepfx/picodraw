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
    buffers: SlotMap<DefaultKey, Option<Buffer>>,

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

    pub fn open<'a>(&'a mut self, screen: BufferMut<'a>) -> SoftwareContext<'a> {
        SoftwareContext { owner: self, screen }
    }
}

impl<'a> Context for SoftwareContext<'a> {
    fn create_texture_render(&mut self) -> RenderTexture {
        let id = self.owner.buffers.insert(Some(Buffer::new(0, 0)));
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
        let mut commands = buffer.list_commands().iter();
        let mut target = match commands.next() {
            Some(Command::SetRenderTarget { texture, size }) => Some((*texture, *size)),
            None => None,
            _ => panic!("render target is not set"),
        };

        loop {
            let target_buffer = match target {
                Some((Some(texture), Size { width, height })) => {
                    let mut buffer = self
                        .owner
                        .buffers
                        .get_mut(KeyData::from_ffi(texture.0).into())
                        .expect("unknown texture id")
                        .take()
                        .expect("render texture is currently in use");

                    if buffer.width() != width as usize || buffer.height() != height as usize {
                        buffer.resize(width as usize, height as usize);
                    }

                    Some((buffer, texture))
                }

                Some((None, Size { width, height })) => {
                    assert!(
                        self.screen.width() == width as usize && self.screen.height() == height as usize,
                        "screen size mismatch"
                    );

                    None
                }

                None => return,
            };

            let mut dispatcher = Dispatcher::new(&self.owner.arena);
            loop {
                match commands.next() {
                    Some(Command::SetRenderTarget { texture, size }) => {
                        target = Some((*texture, *size));
                        break;
                    }
                    Some(Command::ClearBuffer { bounds }) => {
                        dispatcher.write_clear(*bounds);
                    }
                    Some(Command::BeginQuad { shader, bounds }) => {
                        let shader = self
                            .owner
                            .shaders
                            .get(KeyData::from_ffi(shader.0).into())
                            .expect("unknown shader id");

                        dispatcher.write_start(*bounds, &shader);
                    }
                    Some(Command::EndQuad) => {
                        dispatcher.write_end();
                    }
                    Some(Command::WriteFloat(x)) => {
                        dispatcher.write_data(&[VMSlot { float: *x }]);
                    }
                    Some(Command::WriteInt(x)) => {
                        dispatcher.write_data(&[VMSlot { int: *x }]);
                    }
                    Some(Command::WriteStaticTexture(tex)) => {
                        let tex = self
                            .owner
                            .textures
                            .get(KeyData::from_ffi(tex.0).into())
                            .expect("unknown texture id");

                        dispatcher.write_texture(tex.as_ref());
                    }
                    Some(Command::WriteRenderTexture(tex)) => {
                        let tex = self
                            .owner
                            .buffers
                            .get(KeyData::from_ffi(tex.0).into())
                            .expect("unknown render texture id")
                            .as_ref()
                            .expect("render texture is currently in use");

                        dispatcher.write_texture(tex.as_ref());
                    }

                    None => {
                        target = None;
                        break;
                    }
                }
            }

            match target_buffer {
                Some((mut buffer, id)) => {
                    dispatcher.dispatch(&mut self.owner.thread_pool, buffer.as_mut());
                    *self.owner.buffers.get_mut(KeyData::from_ffi(id.0).into()).unwrap() = Some(buffer);
                    self.owner.arena.reset();
                }
                None => {
                    dispatcher.dispatch(&mut self.owner.thread_pool, self.screen.reborrow());
                    self.owner.arena.reset();
                }
            };
        }
    }
}
