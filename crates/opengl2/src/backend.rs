use crate::{
    OpenGlError, OpenGlInfo, OpenGlStats,
    compiler::{self, CompilerError, CompilerShader, GlslCompiler},
    dispatch::{Dispatcher, DispatcherScratch},
    opengl::{
        GlFramebufferBinding, GlProfiler, GlProgram, GlStreamBuffer, GlTexture, GlVertexArray, enable_blend_normal,
        enable_debug,
    },
};
use glow::HasContext;
use picodraw_core2::*;
use std::{collections::HashMap, ffi::CStr, time::Duration};

#[cfg(not(target_arch = "wasm32"))]
pub type Native = glow::Context;

/// A `picodraw` backend that uses OpenGL.
pub struct OpenGlBackend<T: HasContext> {
    shader_compiler: GlslCompiler,

    gl_context: Box<T>,
    gl_info: OpenGlInfo,
    gl_profiler: GlProfiler<T>,
    gl_vertex: GlVertexArray<T>,
    gl_buffer: GlStreamBuffer<T>,
    gl_program: Option<GlProgram<T>>,
    gl_textures: HashMap<T::Texture, GlTexture<T>>,

    scratch: DispatcherScratch<T>,
    stats: OpenGlStats,

    viewport_size: Size,
}

pub struct OpenGlContext<'a, T: HasContext>(&'a mut OpenGlBackend<T>);

pub struct OpenGlShader {
    shader: CompilerShader,
    owner: usize,
}

pub struct OpenGlTexture<T: HasContext> {
    texture: T::Texture,
    owner: usize,
}

impl OpenGlBackend<Native> {
    /// Creates a new OpenGL backend from a given loader function
    /// (a function that takes a GL function name and returns a pointer to that function).
    ///
    /// The `proc_addr` function is used to load a pointer to an OpenGL procedure given it's name.
    ///
    /// #### Requirements
    /// `picodraw` requires at least OpenGL v3.3.
    /// It is possible that the backend can be created with OpenGL v3.0 if the following extensions are present:
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

        let compiler_opts = if gl_info.prefer_tbo_over_ubo() {
            compiler::CompilerOptions {
                glsl_version: gl_info.glsl_version(),
                texture_units: gl_info.max_texture_units - 1,
                buffer_mode: compiler::CompilerBufferMode::TextureBuffer,
            }
        } else {
            compiler::CompilerOptions {
                glsl_version: gl_info.glsl_version(),
                texture_units: gl_info.max_texture_units,
                buffer_mode: compiler::CompilerBufferMode::UniformBlock {
                    size_bytes: gl_info.target_ubo_size(),
                },
            }
        };

        if cfg!(debug_assertions) {
            enable_debug(&mut gl_context);
        }

        Ok(Self {
            scratch: DispatcherScratch::default(),
            stats: OpenGlStats::default(),

            gl_program: None,
            shader_compiler: GlslCompiler::new(compiler_opts),

            gl_context: Box::new(gl_context),
            gl_info,
            gl_profiler,
            gl_vertex,
            gl_buffer,
            gl_textures: HashMap::new(),

            viewport_size: Size { width: 1, height: 1 },
        })
    }

    /// Get a [`Context`](picodraw_core::Context) for the OpenGL backend.
    ///
    /// #### Safety
    /// This function should be called only if the OpenGL context is currently active for the current thread.
    pub unsafe fn open(&mut self) -> OpenGlContext<'_, T> {
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

        if let Some(program) = self.gl_program {
            program.delete(&self.gl_context);
        }

        for (_, texture) in self.gl_textures {
            texture.delete(&self.gl_context);
        }
    }
}

impl<'a, T: HasContext> OpenGlContext<'a, T> {
    pub fn reborrow(&'_ mut self) -> OpenGlContext<'_, T> {
        OpenGlContext(self.0)
    }

    /// Returns the statistics information for the last frame
    pub fn stats(&self) -> OpenGlStats {
        self.0.stats.clone()
    }

    /// Set the target screen size in physical pixel.
    pub fn set_viewport(&mut self, size: impl Into<Size>) {
        self.0.viewport_size = size.into();
    }

    /// Take a screenshot of a region of a buffer.
    /// Useful for debugging and testing.
    pub fn screenshot(&self, buffer: Option<&OpenGlTexture<T>>, bounds: impl Into<Bounds>) -> Vec<u8> {
        let bounds = bounds.into();

        let buffer = match buffer {
            Some(buffer) => {
                let texture = match self.0.gl_textures.get(&buffer.texture) {
                    Some(tex) if buffer.owner != self.unique_id() => tex,
                    _ => panic!("resource does not belong to this context"),
                };

                texture.bind_framebuffer(&self.0.gl_context)
            }
            None => GlFramebufferBinding::default(&*self.0.gl_context),
        };

        buffer.screenshot(
            bounds.left as _,
            bounds.top as _,
            bounds.width() as _,
            bounds.height() as _,
        )
    }

    pub fn unique_id(&self) -> usize {
        &*self.0.gl_context as *const _ as usize
    }
}

impl<'a, T: HasContext + 'static> Context for OpenGlContext<'a, T> {
    type Shader = OpenGlShader;
    type Texture = OpenGlTexture<T>;

    fn create_texture(&mut self, size: Size, format: TextureFormat) -> Result<Self::Texture, TextureError> {
        let texture = GlTexture::new(&*self.0.gl_context, size.width, size.height, format);
        let texture_id = texture.texture();

        self.0.gl_textures.insert(texture_id, texture);

        Ok(OpenGlTexture {
            texture: texture_id,
            owner: self.unique_id(),
        })
    }

    fn upload_texture(&mut self, texture: &mut Self::Texture, data: TextureData) -> Result<(), TextureError> {
        let texture = match self.0.gl_textures.get(&texture.texture) {
            Some(tex) if texture.owner == self.unique_id() => tex,
            _ => panic!("resource does not belong to this context"),
        };

        texture.upload_subregion(&self.0.gl_context, data)
    }

    fn delete_texture(&mut self, texture: Self::Texture) {
        assert!(
            self.unique_id() == texture.owner,
            "resource does not belong to this context"
        );

        if let Some(texture) = self.0.gl_textures.remove(&texture.texture) {
            texture.delete(&self.0.gl_context);
        }
    }

    fn create_shader(&mut self, shader: ShaderData) -> Result<Self::Shader, ShaderError> {
        if let Some(program) = self.0.gl_program.take() {
            program.delete(&self.0.gl_context);
        }

        let shader = match self.0.shader_compiler.add_shader(shader) {
            Ok(shader) => shader,
            Err(CompilerError::TooManyTextures) => return Err(ShaderError::TooComplex),
            _ => return Err(ShaderError::MalformedGraph),
        };

        Ok(OpenGlShader {
            shader,
            owner: self.unique_id(),
        })
    }

    fn delete_shader(&mut self, shader: Self::Shader) {
        assert!(
            self.unique_id() == shader.owner,
            "resource does not belong to this context"
        );

        self.0.shader_compiler.remove_shader(shader.shader.index);
    }

    fn draw(
        &mut self,
        target: DrawTarget<Self>,
        f: impl FnOnce(&mut dyn FrameEncoder<Shader = Self::Shader, Texture = Self::Texture>),
    ) {
        let gl = &*self.0.gl_context;

        self.0.gl_profiler.begin(gl);

        let unique_id = self.unique_id();
        let gl_program = self.0.gl_program.get_or_insert_with(|| {
            let gl = &*self.0.gl_context;

            let result = self.0.shader_compiler.compile();
            let texture_units = self.0.shader_compiler.options().texture_units;

            let program = GlProgram::compile(gl, &result.vertex, &result.fragment);
            let bind_program = program.bind(gl);

            if self.0.gl_info.prefer_tbo_over_ubo() {
                bind_program.set_texture_sampler_binding(gl, compiler::UNIFORM_BUFFER_TEXTURE, 0);

                for i in 0..texture_units {
                    bind_program.set_texture_sampler_binding(
                        gl,
                        &format!("{}[{}]", compiler::UNIFORM_TEXTURE_SAMPLERS, i),
                        i + 1,
                    );
                }
            } else {
                bind_program.set_uniform_block_binding(gl, compiler::UNIFORM_BUFFER_UNIFORM_F32, 0);
                bind_program.set_uniform_block_binding(gl, compiler::UNIFORM_BUFFER_UNIFORM_U32, 0); //funny aliasing trick

                for i in 0..texture_units {
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

            program
        });

        let bind_program = gl_program.bind(gl);
        let bind_vertex_array = self.0.gl_vertex.bind(gl);

        let mut dispatcher = Dispatcher::new(
            &mut self.0.scratch,
            &self.0.gl_context,
            bind_program,
            bind_vertex_array,
            &self.0.gl_buffer,
        );

        match target {
            DrawTarget::Screen => dispatcher.set_target_backbuffer(self.0.viewport_size),
            DrawTarget::Texture(texture) => {
                let texture = match self.0.gl_textures.get(&texture.texture) {
                    Some(tex) if texture.owner == unique_id => tex,
                    _ => panic!("resource does not belong to this context"),
                };

                dispatcher.set_target_texture(texture);
            }
        }

        enable_blend_normal(gl);

        f(&mut dispatcher);

        dispatcher.flush();

        self.0.gl_profiler.end(gl);

        self.0.stats.draw_calls = dispatcher.total_drawcalls_issued;
        self.0.stats.bytes_sent = dispatcher.total_bytes_written;
        self.0.stats.total_quads = dispatcher.total_quads_written;
        self.0.stats.total_objects = dispatcher.total_objects_written;
        self.0.stats.total_pixels = dispatcher.total_pixels_written;
        self.0.stats.gpu_time = self.0.gl_profiler.query().map(|x| Duration::from_nanos(x as u64));
        self.0.stats.cpu_time = Some(dispatcher.build_time_begin.elapsed());
    }
}

impl<'a, T: HasContext + 'static> FrameEncoder<'a> for Dispatcher<'a, T> {
    type Shader = OpenGlShader;
    type Texture = OpenGlTexture<T>;

    fn clear(&mut self, bounds: Bounds) {
        self.push_clear(bounds);
    }

    fn object(&mut self, shader: &OpenGlShader) {
        self.push_object(&shader.shader);
    }

    fn add_rect(&mut self, quad: Bounds) {
        self.push_object_rect(quad);
    }

    fn add_data(&mut self, buffer: &[u8]) {
        self.push_object_data(buffer);
    }

    fn add_texture(&mut self, texture: &<OpenGlContext<'a, T> as Context>::Texture) {
        self.push_object_texture(texture.texture);
    }
}
