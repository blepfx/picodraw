use crate::{
    buffer::{Buffer, BufferMut},
    dispatch::Dispatcher,
    util::{SimdDispatcher, ThreadPool},
    vm::{CompiledShader, VMSlot},
};
use bumpalo::Bump;
use picodraw_core::{
    Command, Context, DrawError, Graph, QuadData, ShaderId, Size, TextureData, TextureFormat, TextureId,
};
use slotmap::{DefaultKey, Key, KeyData, SlotMap};

pub struct SoftwareBackend {
    shaders: SlotMap<DefaultKey, CompiledShader>,
    buffers: SlotMap<DefaultKey, Option<Buffer>>,

    arena: Bump,
    thread_pool: ThreadPool,
    simd_dispatch: SimdDispatcher,
}

pub struct SoftwareContext<'a> {
    owner: &'a mut SoftwareBackend,
    screen: BufferMut<'a>,
}

impl SoftwareBackend {
    pub fn new() -> Self {
        Self {
            arena: Bump::new(),
            thread_pool: ThreadPool::new(),
            simd_dispatch: SimdDispatcher::new(),

            shaders: SlotMap::new(),
            buffers: SlotMap::new(),
        }
    }

    pub fn open<'a>(&'a mut self, screen: BufferMut<'a>) -> SoftwareContext<'a> {
        SoftwareContext { owner: self, screen }
    }
}

impl<'a> SoftwareContext<'a> {
    pub fn reborrow(&mut self) -> SoftwareContext<'_> {
        SoftwareContext {
            owner: self.owner,
            screen: self.screen.reborrow(),
        }
    }

    fn draw_to_buffer(&mut self, commands: &[Command], buffer: Option<BufferMut>) -> Result<(), DrawError> {
        let mut dispatch = Dispatcher::new(&self.owner.arena);

        for command in commands {
            match *command {
                Command::Clear(bounds) => {
                    dispatch.write_clear(bounds);
                }
                Command::Begin(bounds, shader) => {
                    let shader = self
                        .owner
                        .shaders
                        .get(KeyData::from_ffi(shader.0).into())
                        .ok_or_else(|| DrawError::InvalidShader)?;

                    dispatch.write_start(bounds, &shader);
                }
                Command::End => {
                    dispatch.write_end()?;
                }
                Command::Data(QuadData::Float(float)) => {
                    dispatch.write_data(&[VMSlot { float }]);
                }
                Command::Data(QuadData::Int(int)) => {
                    dispatch.write_data(&[VMSlot { int }]);
                }
                Command::Data(QuadData::Texture(tex)) => {
                    let tex = self
                        .owner
                        .buffers
                        .get(KeyData::from_ffi(tex.0).into())
                        .ok_or_else(|| DrawError::InvalidTexture)?
                        .as_ref()
                        .ok_or_else(|| DrawError::TargetInUse)?;

                    dispatch.write_texture(tex.as_ref());
                }
            }
        }

        dispatch.dispatch(
            &mut self.owner.thread_pool,
            self.owner.simd_dispatch,
            buffer.unwrap_or(self.screen.reborrow()),
        );
        self.owner.arena.reset();

        Ok(())
    }
}

impl<'a> Context for SoftwareContext<'a> {
    fn create_texture(&mut self, size: Size, _: TextureFormat) -> TextureId {
        let id = self
            .owner
            .buffers
            .insert(Some(Buffer::new(size.width as _, size.height as _)));
        TextureId(id.data().as_ffi())
    }

    fn delete_texture(&mut self, id: TextureId) -> bool {
        self.owner.buffers.remove(KeyData::from_ffi(id.0).into()).is_some()
    }

    fn upload_texture(&mut self, id: TextureId, data: TextureData) -> bool {
        let buffer = self
            .owner
            .buffers
            .get_mut(KeyData::from_ffi(id.0).into())
            .map(|x| x.as_mut().expect("texture is invalid state"));

        if let Some(buffer) = buffer {
            buffer
                .as_mut()
                .subregion_mut(
                    data.bounds.left as usize,
                    data.bounds.top as usize,
                    data.bounds.width() as usize,
                    data.bounds.height() as usize,
                )
                .unpack_data(
                    data.bounds.width() as usize,
                    data.bounds.height() as usize,
                    data.format,
                    data.data,
                );

            return true;
        }

        false
    }

    fn create_shader(&mut self, graph: Graph) -> ShaderId {
        let compiled = CompiledShader::compile(&self.owner.arena, &graph);
        let key = self.owner.shaders.insert(compiled);
        self.owner.arena.reset();

        ShaderId(key.data().as_ffi())
    }

    fn delete_shader(&mut self, id: ShaderId) -> bool {
        self.owner.shaders.remove(KeyData::from_ffi(id.0).into()).is_some()
    }

    fn draw_screen(&mut self, commands: &[Command]) -> Result<(), DrawError> {
        self.draw_to_buffer(commands, None)
    }

    fn draw_texture(&mut self, target: TextureId, commands: &[Command]) -> Result<(), DrawError> {
        let mut buffer = self
            .owner
            .buffers
            .get_mut(KeyData::from_ffi(target.0).into())
            .ok_or_else(|| DrawError::InvalidTarget)?
            .take()
            .ok_or_else(|| DrawError::TargetInUse)?;

        self.draw_to_buffer(commands, Some(buffer.as_mut()))?;

        // FIXME: put back even if we panic
        *self.owner.buffers.get_mut(KeyData::from_ffi(target.0).into()).unwrap() = Some(buffer);

        Ok(())
    }
}
