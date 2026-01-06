use crate::{
    buffer::{Buffer, BufferMut},
    dispatch::Dispatcher,
    util::{SimdDispatcher, ThreadPool},
    vm::CompiledShader,
};
use bumpalo::Bump;
use picodraw_core::{
    Bounds, Color, Context, DrawTarget, FrameEncoder, ShaderData, ShaderError, Size, TextureData, TextureError,
    TextureFormat,
};

pub struct SoftwareBackend {
    arena: Bump,
    thread_pool: ThreadPool,
    simd_dispatch: SimdDispatcher,
}

pub struct SoftwareContext<'a> {
    owner: &'a mut SoftwareBackend,
    screen: BufferMut<'a>,
}

impl SoftwareBackend {
    pub fn single_threaded() -> Self {
        Self {
            arena: Bump::new(),
            simd_dispatch: SimdDispatcher::new(),
            thread_pool: ThreadPool::with_threads(1),
        }
    }

    pub fn multi_threaded() -> Self {
        Self {
            arena: Bump::new(),
            simd_dispatch: SimdDispatcher::new(),
            thread_pool: ThreadPool::with_threads(std::thread::available_parallelism().map(|x| x.get()).unwrap_or(1)),
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
}

impl<'a> Context for SoftwareContext<'a> {
    type Shader = CompiledShader;
    type Texture = Buffer;

    fn create_texture(&mut self, size: Size, _: TextureFormat) -> Result<Buffer, TextureError> {
        Ok(Buffer::new(size.width as usize, size.height as usize))
    }

    fn delete_texture(&mut self, texture: Buffer) {
        drop(texture);
    }

    fn upload_texture(&mut self, texture: &mut Buffer, data: TextureData) -> Result<(), TextureError> {
        texture
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
            )
    }

    fn create_shader(&mut self, data: &ShaderData) -> Result<CompiledShader, ShaderError> {
        self.owner.arena.reset();
        CompiledShader::compile(&self.owner.arena, data)
    }

    fn delete_shader(&mut self, shader: CompiledShader) {
        drop(shader);
    }

    fn draw<'s>(
        &'s mut self,
        target: DrawTarget<Self::Texture>,
        f: impl FnOnce(&mut dyn FrameEncoder<'s, Shader = Self::Shader, Texture = Self::Texture>),
    ) {
        self.owner.arena.reset();
        let mut dispatcher = Dispatcher::new(&self.owner.arena);
        f(&mut dispatcher);

        dispatcher.rasterize(
            &mut self.owner.thread_pool,
            self.owner.simd_dispatch,
            match target {
                DrawTarget::Screen => self.screen.reborrow(),
                DrawTarget::Texture(texture) => texture.as_mut(),
            },
        );
    }
}

impl<'a> FrameEncoder<'a> for Dispatcher<'a> {
    type Shader = CompiledShader;
    type Texture = Buffer;

    fn clear(&mut self, bounds: Bounds, color: Color) {
        self.push_clear(bounds, color);
    }

    fn draw(&mut self, shader: &'a Self::Shader) {
        self.push_object(shader);
    }

    fn add_rect(&mut self, rect: Bounds) {
        self.push_object_rect(rect);
    }

    fn add_data(&mut self, data: &[u8]) {
        self.push_object_data(data);
    }

    fn add_texture(&mut self, texture: &'a Self::Texture) {
        self.push_object_texture(texture.as_ref());
    }
}
