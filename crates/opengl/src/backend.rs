use crate::{
    compiler,
    dispatch::{Dispatcher, DispatcherScratch},
    opengl::{
        GlFramebufferBinding, GlProfiler, GlProgram, GlStreamBuffer, GlTextureRender, GlTextureStatic, GlVertexArray,
        enable_blend_normal, enable_debug,
    },
};
use glow::HasContext;
use picodraw_core::*;
use slotmap::{DefaultKey, Key, KeyData, SecondaryMap, SlotMap};
use std::{ffi::CStr, time::Duration};

pub use crate::opengl::GlInfo as OpenGlInfo;
pub struct OpenGlContext<'a, T: HasContext>(&'a mut OpenGlBackend<T>);

#[cfg(any(not(target_arch = "wasm32"), target_os = "emscripten"))]
pub type Native = glow::Context;

#[derive(Debug, Clone, Default)]
pub struct OpenGlStats {
    /// Total GPU render time of one of the previous draw calls.
    /// Does not necessarily correspond to the time of the last draw call (there is a small delay due to the asynchronous nature of GPUs).
    pub gpu_time: Option<Duration>,

    /// Number of GPU draw calls/context switches
    pub draw_calls: u32,

    /// Total number of bytes sent to the GPU, including quad lists and quad data
    pub bytes_sent: u64,

    /// Number of quads sent to the GPU
    pub total_quads: u32,
}

/// A `picodraw` backend that uses OpenGL.
pub struct OpenGlBackend<T: HasContext> {
    shaders: SlotMap<DefaultKey, Graph>,
    textures: SlotMap<DefaultKey, GlTextureStatic<T>>,
    framebuffers: SlotMap<DefaultKey, Option<GlTextureRender<T>>>,
    program: Option<CompiledProgram<T>>,

    gl_context: T,
    gl_info: OpenGlInfo,
    gl_profiler: GlProfiler<T>,
    gl_vertex: GlVertexArray<T>,
    gl_buffer: GlStreamBuffer<T>,

    scratch: DispatcherScratch<T>,
    stats: OpenGlStats,
}

#[derive(Debug, Clone)]
pub enum OpenGlError {
    UnsupportedVersion { info: OpenGlInfo },
}

unsafe impl<T: HasContext> Send for OpenGlBackend<T> {}

impl OpenGlBackend<glow::Context> {
    /// Creates a new OpenGL backend from a given loader function
    /// (a function that takes a GL function name and returns a pointer to that function).
    ///
    /// The `proc_addr` function is used to load a pointer to an OpenGL procedure given it's name.
    ///
    /// #### Requirements
    /// `picodraw` requires at least OpenGL v3.3.
    /// It is possible that the backend can be created with an OpenGL v3.0 if the following extensions are present:
    /// - `ARB_texture_buffer_object` or `EXT_texture_buffer`
    /// - `ARB_shader_bit_encoding`
    /// - `ARB_timer_query`
    ///
    /// #### Error Conditions
    /// - If the version is not supported [`OpenGlError::UnsupportedVersion`] is returned.
    ///
    /// #### Safety
    /// This function should be called only if the OpenGL context is currently active for the current thread.
    pub unsafe fn new<F>(loader: F) -> Result<Self, OpenGlError>
    where
        F: FnMut(&CStr) -> *const std::os::raw::c_void,
    {
        unsafe { Self::from_glow(glow::Context::from_loader_function_cstr(loader)) }
    }
}

impl<T: HasContext> OpenGlBackend<T> {
    /// Creates a new OpenGL backend from a given `glow` context.
    ///
    /// See [`OpenGlBackend::new`] for more details.
    pub unsafe fn from_glow(mut gl_context: T) -> Result<Self, OpenGlError> {
        let gl_info = OpenGlInfo::query(&gl_context);

        if !gl_info.is_baseline_supported() {
            return Err(OpenGlError::UnsupportedVersion { info: gl_info });
        }

        let gl_vertex = GlVertexArray::new(&gl_context);

        let gl_profiler = if gl_info.is_timer_query_supported() {
            GlProfiler::new(&gl_context)
        } else {
            GlProfiler::dummy()
        };

        let gl_buffer = if gl_info.prefer_tbo_over_ubo() {
            GlStreamBuffer::new_tbo(&gl_context, gl_info.target_tbo_size())
        } else {
            GlStreamBuffer::new_ubo(&gl_context, gl_info.target_ubo_size())
        };

        if cfg!(debug_assertions) {
            enable_debug(&mut gl_context);
        }

        Ok(Self {
            scratch: DispatcherScratch::default(),
            stats: OpenGlStats::default(),

            shaders: SlotMap::with_key(),
            textures: SlotMap::with_key(),
            framebuffers: SlotMap::with_key(),
            program: None,

            gl_context,
            gl_info,
            gl_profiler,
            gl_vertex,
            gl_buffer,
        })
    }

    /// Get a [`Context`](picodraw_core::Context) for the OpenGL backend.
    ///
    /// #### Safety
    /// This function should be called only if the OpenGL context is currently active for the current thread.
    pub unsafe fn open(&mut self) -> OpenGlContext<T> {
        OpenGlContext(self)
    }

    /// Delete all the resources associated with the OpenGL backend.
    ///
    /// #### Safety
    /// This function should be called only if the OpenGL context is currently active for the current thread.
    pub unsafe fn delete(self) {
        self.gl_buffer.delete(&self.gl_context);
        self.gl_vertex.delete(&self.gl_context);
        self.gl_profiler.delete(&self.gl_context);

        for (_, texture) in self.textures.into_iter() {
            texture.delete(&self.gl_context);
        }

        for (_, framebuffer) in self.framebuffers.into_iter() {
            if let Some(framebuffer) = framebuffer {
                framebuffer.delete(&self.gl_context);
            }
        }

        if let Some(program) = self.program {
            program.program.delete(&self.gl_context);
        }
    }
}

impl<'a, T: HasContext> OpenGlContext<'a, T> {
    /// Take a screenshot of a region of a buffer.
    /// Useful for debugging and testing.
    pub fn screenshot(&self, buffer: Option<RenderTexture>, bounds: impl Into<Bounds>) -> Vec<u8> {
        let bounds = bounds.into();

        let buffer = match buffer {
            Some(buffer) => {
                let framebuffer = self
                    .0
                    .framebuffers
                    .get(KeyData::from_ffi(buffer.0).into())
                    .expect("invalid render texture id");

                if let Some(framebuffer) = framebuffer {
                    framebuffer.bind(&self.0.gl_context)
                } else {
                    panic!("render texture is in use");
                }
            }
            None => GlFramebufferBinding::default(&self.0.gl_context),
        };

        buffer.screenshot(
            bounds.left as _,
            bounds.top as _,
            bounds.width() as _,
            bounds.height() as _,
        )
    }

    /// Returns the statistics information for the last frame
    pub fn stats(&self) -> OpenGlStats {
        self.0.stats.clone()
    }
}

impl<'a, T: HasContext> Context for OpenGlContext<'a, T> {
    fn create_texture_render(&mut self) -> RenderTexture {
        let id = self
            .0
            .framebuffers
            .insert(Some(GlTextureRender::new(&self.0.gl_context, 1, 1)));

        RenderTexture(id.data().as_ffi())
    }

    fn create_texture_static(&mut self, data: ImageData) -> Texture {
        let id = self.0.textures.insert(GlTextureStatic::new(&self.0.gl_context, data));

        Texture(id.data().as_ffi())
    }

    fn create_shader(&mut self, graph: Graph) -> Shader {
        if let Some(program) = self.0.program.take() {
            program.program.delete(&self.0.gl_context);
        }

        let id = self.0.shaders.insert(graph);
        Shader(id.data().as_ffi())
    }

    fn delete_texture_render(&mut self, id: RenderTexture) -> bool {
        match self.0.framebuffers.remove(KeyData::from_ffi(id.0).into()) {
            Some(fb) => {
                if let Some(framebuffer) = fb {
                    framebuffer.delete(&self.0.gl_context);
                }

                true
            }
            _ => false,
        }
    }

    fn delete_texture_static(&mut self, id: Texture) -> bool {
        match self.0.textures.remove(KeyData::from_ffi(id.0).into()) {
            Some(texture) => {
                texture.delete(&self.0.gl_context);
                true
            }
            _ => false,
        }
    }

    fn delete_shader(&mut self, id: Shader) -> bool {
        match self.0.shaders.remove(KeyData::from_ffi(id.0).into()) {
            Some(_) => true,
            _ => false,
        }
    }

    fn draw(&mut self, buffer: &CommandBuffer) {
        self.0.stats = OpenGlStats::default();

        let gl = &self.0.gl_context;
        let program = self.0.program.get_or_insert_with(|| {
            let options = if self.0.gl_info.prefer_tbo_over_ubo() {
                compiler::CompilerOptions {
                    glsl_version: self.0.gl_info.glsl_version(),
                    texture_units: self.0.gl_info.max_texture_units - 1,
                    buffer_mode: compiler::CompilerBufferMode::TextureBuffer,
                }
            } else {
                compiler::CompilerOptions {
                    glsl_version: self.0.gl_info.glsl_version(),
                    texture_units: self.0.gl_info.max_texture_units,
                    buffer_mode: compiler::CompilerBufferMode::UniformBlock {
                        size_bytes: self.0.gl_info.target_ubo_size(),
                    },
                }
            };

            let result = compiler::compile_glsl(
                options,
                self.0
                    .shaders
                    .iter()
                    .map(|(id, shader)| (Shader(id.data().as_ffi()), shader)),
            );

            let program = GlProgram::compile(gl, &result.vertex, &result.fragment);
            let bind_program = program.bind(gl);

            if self.0.gl_info.prefer_tbo_over_ubo() {
                bind_program.set_texture_sampler_binding(gl, compiler::UNIFORM_BUFFER_TEXTURE, 0);

                for i in 0..options.texture_units {
                    bind_program.set_texture_sampler_binding(
                        gl,
                        &format!("{}[{}]", compiler::UNIFORM_TEXTURE_SAMPLERS, i),
                        i + 1,
                    );
                }
            } else {
                bind_program.set_uniform_block_binding(gl, compiler::UNIFORM_BUFFER_UNIFORM_F32, 0);
                bind_program.set_uniform_block_binding(gl, compiler::UNIFORM_BUFFER_UNIFORM_U32, 0); //funny aliasing trick

                for i in 0..options.texture_units {
                    bind_program.set_texture_sampler_binding(
                        gl,
                        &format!("{}[{}]", compiler::UNIFORM_TEXTURE_SAMPLERS, i),
                        i,
                    );
                }
            }

            bind_program.set_uniform_binding(gl, compiler::UNIFORM_FRAME_RESOLUTION, 0);
            bind_program.set_uniform_binding(gl, compiler::UNIFORM_FRAME_SCREEN, 1);
            bind_program.set_uniform_binding(gl, compiler::UNIFORM_BUFFER_DATA_OFFSET, 2);
            bind_program.set_uniform_binding(gl, compiler::UNIFORM_BUFFER_LIST_OFFSET, 3);

            CompiledProgram {
                program,
                layouts: result
                    .layout
                    .into_iter()
                    .map(|(shader, layout)| (DefaultKey::from(KeyData::from_ffi(shader.0)), layout))
                    .collect(),
            }
        });

        self.0.gl_profiler.wrap(gl, || {
            enable_blend_normal(gl);

            let mut commands = buffer.list_commands().iter().copied();
            let bind_program = program.program.bind(gl);
            let bind_vertex_array = self.0.gl_vertex.bind(gl);

            let mut target = match commands.next() {
                Some(Command::SetRenderTarget { texture, size }) => Some((texture, size)),
                None => None,
                _ => panic!("render target is not set"),
            };

            loop {
                let (target_buffer, target_size) = match target {
                    Some((Some(texture), size)) => {
                        let framebuffer = self
                            .0
                            .framebuffers
                            .get_mut(KeyData::from_ffi(texture.0).into())
                            .expect("invalid render texture id")
                            .take()
                            .expect("render texture is in use");

                        let framebuffer = if framebuffer.size() != (size.width, size.height) {
                            framebuffer.delete(gl);
                            GlTextureRender::new(&self.0.gl_context, size.width as _, size.height as _)
                        } else {
                            framebuffer
                        };

                        (Some((texture, framebuffer)), size)
                    }

                    Some((None, size)) => (None, size),
                    None => return,
                };

                let mut dispatcher = Dispatcher::new(
                    &mut self.0.scratch,
                    &self.0.gl_context,
                    &bind_program,
                    &bind_vertex_array,
                    &self.0.gl_buffer,
                );

                match target_buffer.as_ref() {
                    Some((_, framebuffer)) => {
                        dispatcher.set_target_texture(framebuffer, target_size);
                    }
                    None => {
                        dispatcher.set_target_backbuffer(target_size);
                    }
                }

                loop {
                    match commands.next() {
                        Some(Command::SetRenderTarget { texture, size }) => {
                            target = Some((texture, size));
                            break;
                        }

                        Some(Command::ClearBuffer { bounds }) => {
                            dispatcher.clear_rect(bounds);
                        }

                        Some(Command::BeginQuad { shader, bounds }) => {
                            let layout = program
                                .layouts
                                .get(KeyData::from_ffi(shader.0).into())
                                .expect("invalid shader id");

                            dispatcher.quad_start(layout, bounds);
                        }

                        Some(Command::EndQuad) => {
                            dispatcher.quad_end();
                        }

                        Some(Command::WriteFloat(x)) => {
                            dispatcher.quad_data(x.to_bits());
                        }

                        Some(Command::WriteInt(x)) => {
                            dispatcher.quad_data(x as u32);
                        }

                        Some(Command::WriteStaticTexture(x)) => {
                            let texture = self
                                .0
                                .textures
                                .get(KeyData::from_ffi(x.0).into())
                                .expect("invalid static texture id");

                            dispatcher.quad_texture(texture.texture());
                        }

                        Some(Command::WriteRenderTexture(x)) => {
                            let framebuffer = self
                                .0
                                .framebuffers
                                .get(KeyData::from_ffi(x.0).into())
                                .expect("invalid render texture id")
                                .as_ref()
                                .expect("render texture is currently in use");

                            dispatcher.quad_texture(framebuffer.texture());
                        }

                        None => {
                            target = None;
                            break;
                        }
                    }
                }

                dispatcher.flush();

                self.0.stats.draw_calls += dispatcher.total_drawcalls_issued;
                self.0.stats.bytes_sent += dispatcher.total_bytes_written;
                self.0.stats.total_quads += dispatcher.total_quads_written;

                if let Some((texture, framebuffer)) = target_buffer {
                    self.0
                        .framebuffers
                        .get_mut(KeyData::from_ffi(texture.0).into())
                        .expect("invalid render texture id")
                        .replace(framebuffer);
                }
            }
        });

        self.0.stats.gpu_time = self.0.gl_profiler.query().map(|x| Duration::from_nanos(x as u64));
    }
}

struct CompiledProgram<T: HasContext> {
    program: GlProgram<T>,
    layouts: SecondaryMap<DefaultKey, compiler::serialize::ShaderDataLayout>,
}
