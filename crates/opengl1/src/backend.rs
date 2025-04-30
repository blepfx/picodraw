use crate::{
    compiler::{
        self,
        serialize::{QuadDescriptorStruct, write_uninit},
    },
    raw::*,
};
use picodraw_core::*;
use slotmap::{DefaultKey, Key, KeyData, SecondaryMap, SlotMap};
use std::{
    ffi::{CStr, c_void},
    time::Duration,
};

pub use crate::raw::GlInfo as OpenGlInfo;
pub struct OpenGlContext<'a>(&'a mut OpenGlBackend);

/// A `picodraw` backend that uses OpenGL.
pub struct OpenGlBackend {
    shaders: SlotMap<DefaultKey, ResourceShader>,
    textures: SlotMap<DefaultKey, ResourceTexture>,
    framebuffers: SlotMap<DefaultKey, ResourceFramebuffer>,
    program: Option<CompiledProgram>,

    gl_bindings: GlBindings,
    gl_buffer: GlUniformBuffer,
    gl_vao: GlVertexArrayObject,
    gl_info: GlInfo,
    gl_query: GlQuery,

    scratch_quads: Vec<compiler::serialize::QuadDescriptorStruct>,
    scratch_textures: compiler::serialize::ShaderTextureAllocator,
}

#[derive(Debug, Clone)]
pub enum OpenGlError {
    InvalidBinding { name: &'static CStr },
    InvalidVersion { info: OpenGlInfo },
    InvalidInfo,
}

unsafe impl Send for OpenGlBackend {}

impl OpenGlBackend {
    /// Creates a new OpenGL backend.
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
    /// - If the version is not supported [`OpenGlError::InvalidVersion`] is returned.
    /// - If one of the required procedure pointers is not valid [`OpenGlError::InvalidBinding`] is returned.
    /// - If querying the current OpenGL version, extensions or limits fails [`OpenGlError::InvalidInfo`] is returned.
    /// - If the OpenGL context is not active for the current thread, the behavior is undefined.
    ///
    /// #### Safety
    /// This function should be called only if the OpenGL context is currently active for the current thread.
    pub unsafe fn new(proc_addr: &dyn Fn(&CStr) -> *const c_void) -> Result<Self, OpenGlError> {
        unsafe {
            let gl_bindings = GlBindings::load_from(proc_addr).map_err(|name| OpenGlError::InvalidBinding { name })?;
            let (gl_info, gl_buffer, gl_vao, gl_query) = GlContext::within(&gl_bindings, |gl| {
                clear_error(gl);

                let info = GlInfo::get(gl).ok_or(OpenGlError::InvalidInfo)?;
                let supported = {
                    let query_timer = info.version >= (3, 3) || info.extensions.contains("ARB_timer_query");
                    let buffer_texture = info.version >= (3, 1)
                        || info.extensions.contains("ARB_texture_buffer_object")
                        || info.extensions.contains("EXT_texture_buffer");
                    let shader_bit_encoding =
                        info.version >= (3, 3) || info.extensions.contains("ARB_shader_bit_encoding");

                    query_timer && buffer_texture && shader_bit_encoding && info.version >= (3, 0)
                };

                if !supported {
                    return Err(OpenGlError::InvalidVersion { info });
                }

                let gl_buffer = GlUniformBuffer::new(gl, info.max_texture_buffer_size.min(262144));
                let gl_vao = GlVertexArrayObject::new(gl);
                let gl_query = GlQuery::new(gl);

                Ok((info, gl_buffer, gl_vao, gl_query))
            })?;

            Ok(Self {
                scratch_quads: vec![],
                scratch_textures: compiler::serialize::ShaderTextureAllocator::new(
                    (gl_info.max_texture_image_units - 1) as u32,
                ),

                shaders: SlotMap::new(),
                framebuffers: SlotMap::new(),
                textures: SlotMap::new(),
                program: None,

                gl_bindings,
                gl_buffer,
                gl_query,
                gl_info,
                gl_vao,
            })
        }
    }

    /// Get a [`Context`](picodraw_core::Context) for the OpenGL backend.
    ///
    /// #### Safety
    /// This function should be called only if the OpenGL context is currently active for the current thread.
    pub unsafe fn open(&mut self) -> OpenGlContext {
        OpenGlContext(self)
    }

    /// Delete all the resources associated with the OpenGL backend.
    ///
    /// #### Safety
    /// This function should be called only if the OpenGL context is currently active for the current thread.
    pub unsafe fn delete(self) {
        unsafe {
            GlContext::within(&self.gl_bindings, |gl| {
                self.gl_buffer.delete(gl);
                self.gl_query.delete(gl);
                self.gl_vao.delete(gl);

                for (_, fb) in self.framebuffers.into_iter() {
                    fb.framebuffer.delete(gl);
                }

                for (_, tx) in self.textures.into_iter() {
                    tx.texture.delete(gl);
                }

                if let Some(program) = self.program {
                    program.program.delete(gl);
                }
            });
        }
    }
}

impl<'a> OpenGlContext<'a> {
    /// Take a screenshot of a region of a current back buffer.
    /// Useful for debugging and testing.
    pub fn screenshot(&self, bounds: impl Into<Bounds>) -> Vec<u32> {
        let bounds = bounds.into();

        unsafe {
            GlContext::within(&self.0.gl_bindings, |gl| {
                GlFramebuffer::bind_default(gl);
                screenshot_rect(
                    gl,
                    bounds.left as _,
                    bounds.top as _,
                    bounds.width() as _,
                    bounds.height() as _,
                )
            })
        }
    }

    /// Get the total render time of one of the previous draw calls.
    /// Does not necessarily correspond to the time of the last draw call (there is a small delay due to the asynchronous nature of GPUs).
    pub fn gpu_time(&self) -> Duration {
        Duration::from_nanos(self.0.gl_query.query() as u64)
    }
}

impl<'a> Context for OpenGlContext<'a> {
    fn create_texture_render(&mut self) -> RenderTexture {
        unsafe {
            GlContext::within(&self.0.gl_bindings, |gl| {
                let id = self.0.framebuffers.insert(ResourceFramebuffer {
                    framebuffer: GlFramebuffer::new(gl, 1, 1),
                    width: 1,
                    height: 1,
                });

                RenderTexture(id.data().as_ffi())
            })
        }
    }

    fn create_texture_static(&mut self, data: ImageData) -> Texture {
        unsafe {
            GlContext::within(&self.0.gl_bindings, |gl| {
                let id = self.0.textures.insert(ResourceTexture {
                    texture: GlTexture::new(gl, data),
                });

                Texture(id.data().as_ffi())
            })
        }
    }

    fn create_shader(&mut self, graph: Graph) -> Shader {
        unsafe {
            GlContext::within(&self.0.gl_bindings, |gl| {
                if let Some(program) = self.0.program.take() {
                    program.program.delete(gl);
                }
            })
        }

        let id = self.0.shaders.insert(ResourceShader { graph });
        Shader(id.data().as_ffi())
    }

    fn delete_texture_render(&mut self, id: RenderTexture) -> bool {
        unsafe {
            GlContext::within(&self.0.gl_bindings, |gl| {
                match self.0.framebuffers.remove(KeyData::from_ffi(id.0).into()) {
                    Some(fb) => {
                        fb.framebuffer.delete(gl);
                        true
                    }
                    _ => false,
                }
            })
        }
    }

    fn delete_texture_static(&mut self, id: Texture) -> bool {
        unsafe {
            GlContext::within(&self.0.gl_bindings, |gl| {
                match self.0.textures.remove(KeyData::from_ffi(id.0).into()) {
                    Some(tx) => {
                        tx.texture.delete(gl);
                        true
                    }
                    _ => false,
                }
            })
        }
    }

    fn delete_shader(&mut self, id: Shader) -> bool {
        match self.0.shaders.remove(KeyData::from_ffi(id.0).into()) {
            Some(_) => true,
            _ => false,
        }
    }

    fn draw(&mut self, buffer: &CommandBuffer) {
        unsafe {
            GlContext::within(&self.0.gl_bindings, |gl| {
                clear_error(gl);

                let program = self.0.program.get_or_insert_with(|| {
                    CompiledProgram::compile(
                        gl,
                        &self.0.gl_info,
                        self.0
                            .shaders
                            .iter()
                            .map(|(id, shader)| (Shader(id.data().as_ffi()), &shader.graph)),
                    )
                });

                self.0.gl_query.wrap(gl, || {
                    program.bind(gl);
                    self.0.gl_vao.bind(gl);
                    self.0.gl_buffer.bind_texture(gl, 0);
                    enable_blend_normal(gl);

                    self.0.scratch_quads.clear();
                    self.0.scratch_textures.clear();

                    let mut state_size = Size { width: 0, height: 0 };
                    let mut state_screen = false;
                    let mut commands = CommandStream(buffer.list_commands());
                    while commands.peek().is_some() {
                        let quads_to_draw = self.0.gl_buffer.update(gl, |writer| {
                            let offset_data_start = writer.pointer();

                            'main: while let Some(cmd) = commands.peek() {
                                match cmd {
                                    Command::SetRenderTarget { texture: None, size } => {
                                        if !self.0.scratch_quads.is_empty() {
                                            break 'main;
                                        }

                                        GlFramebuffer::bind_default(gl);
                                        viewport(gl, 0, 0, size.width as u32, size.height as u32);
                                        uniform_1i(gl, program.uni_frame_screen, 1);
                                        uniform_2f(
                                            gl,
                                            program.uni_frame_resolution,
                                            [size.width as f32, size.height as f32],
                                        );

                                        state_screen = true;
                                        state_size = size;
                                        commands.pop();
                                    }

                                    Command::SetRenderTarget {
                                        texture: Some(texture),
                                        size,
                                    } => {
                                        if !self.0.scratch_quads.is_empty() {
                                            break 'main;
                                        }

                                        let fb_data = self
                                            .0
                                            .framebuffers
                                            .get_mut(KeyData::from_ffi(texture.0).into())
                                            .expect("invalid id");

                                        if size.width != fb_data.width || size.height != fb_data.height {
                                            std::mem::replace(
                                                &mut fb_data.framebuffer,
                                                GlFramebuffer::new(gl, size.width, size.height),
                                            )
                                            .delete(gl);

                                            fb_data.width = size.width;
                                            fb_data.height = size.height;
                                        }

                                        fb_data.framebuffer.bind(gl);
                                        viewport(gl, 0, 0, size.width as u32, size.height as u32);
                                        uniform_1i(gl, program.uni_frame_screen, 0);
                                        uniform_2f(
                                            gl,
                                            program.uni_frame_resolution,
                                            [size.width as f32, size.height as f32],
                                        );

                                        state_screen = false;
                                        state_size = size;
                                        commands.pop();
                                    }

                                    Command::ClearBuffer { bounds } => {
                                        if !self.0.scratch_quads.is_empty() {
                                            break 'main;
                                        }

                                        if state_screen {
                                            clear_rect(
                                                gl,
                                                bounds.left as _,
                                                (state_size.height as i32 - bounds.bottom as i32) as _,
                                                bounds.width() as _,
                                                bounds.height() as _,
                                            );
                                        } else {
                                            clear_rect(
                                                gl,
                                                bounds.left as _,
                                                bounds.top as _,
                                                bounds.width() as _,
                                                bounds.height() as _,
                                            );
                                        }

                                        commands.pop();
                                    }

                                    Command::BeginQuad { shader, bounds } => {
                                        let layout = program
                                            .layouts
                                            .get(KeyData::from_ffi(shader.0).into())
                                            .expect("invalid id");

                                        {
                                            let mut range = layout.textures.iter().copied();
                                            for cmd in commands.peek_quad() {
                                                let slot = match cmd {
                                                    Command::WriteStaticTexture(x) => {
                                                        compiler::serialize::ShaderTextureSlot::Static(*x)
                                                    }
                                                    Command::WriteRenderTexture(x) => {
                                                        compiler::serialize::ShaderTextureSlot::Render(*x)
                                                    }
                                                    _ => continue,
                                                };

                                                let index = range.next().expect("malformed command stream");
                                                match self.0.scratch_textures.try_allocate(index, slot) {
                                                    Ok(true) => match slot {
                                                        compiler::serialize::ShaderTextureSlot::Static(x) => {
                                                            self.0
                                                                .textures
                                                                .get(KeyData::from_ffi(x.0).into())
                                                                .expect("invalid id")
                                                                .texture
                                                                .bind_texture(gl, index + 1);
                                                        }
                                                        compiler::serialize::ShaderTextureSlot::Render(x) => {
                                                            self.0
                                                                .framebuffers
                                                                .get(KeyData::from_ffi(x.0).into())
                                                                .expect("invalid id")
                                                                .framebuffer
                                                                .bind_texture(gl, index + 1);
                                                        }
                                                    },
                                                    Ok(false) => {}
                                                    Err(_) => break 'main,
                                                }
                                            }

                                            if range.next().is_some() {
                                                panic!("malformed command stream")
                                            }
                                        }

                                        if (self.0.scratch_quads.len() + 1) * GlUniformBuffer::TEXEL_SIZE_BYTES
                                            + (layout.size as usize)
                                            > writer.space_left()
                                        {
                                            writer.invalidate();
                                            break 'main;
                                        }

                                        let offset_quad_start = writer.pointer();
                                        let mut encoder = compiler::serialize::ShaderDataEncoder::new(
                                            &layout,
                                            writer.request(layout.size as usize),
                                        );

                                        commands.pop();
                                        loop {
                                            match commands.pop() {
                                                Some(Command::WriteFloat(x)) => encoder.write_f32(x),
                                                Some(Command::WriteInt(x)) => encoder.write_i32(x),

                                                Some(Command::WriteRenderTexture(_)) => {}
                                                Some(Command::WriteStaticTexture(_)) => {}

                                                Some(Command::EndQuad) => break,
                                                _ => panic!("malformed command stream"),
                                            }
                                        }

                                        encoder.finish();

                                        self.0.scratch_quads.push(compiler::serialize::QuadDescriptorStruct {
                                            left: bounds.left.try_into().unwrap_or(u16::MAX),
                                            top: bounds.top.try_into().unwrap_or(u16::MAX),
                                            right: bounds.right.try_into().unwrap_or(u16::MAX),
                                            bottom: bounds.bottom.try_into().unwrap_or(u16::MAX),
                                            shader: layout.branch_id,
                                            offset: (offset_quad_start - offset_data_start) as u32,
                                        });
                                    }

                                    _ => panic!("malformed command stream"),
                                }
                            }

                            if !self.0.scratch_quads.is_empty() {
                                let offset_quads_start = writer.pointer();
                                let quads_count = self.0.scratch_quads.len();

                                {
                                    let quad_data_bytes =
                                        QuadDescriptorStruct::as_byte_slice(self.0.scratch_quads.as_slice());
                                    let dst = writer.request(quad_data_bytes.len());
                                    write_uninit(dst, quad_data_bytes);
                                }

                                self.0.scratch_textures.clear();
                                self.0.scratch_quads.clear();

                                uniform_1i(gl, program.uni_buffer_offset_instance, offset_quads_start as i32);
                                uniform_1i(gl, program.uni_buffer_offset_data, offset_data_start as i32);

                                quads_count
                            } else {
                                0
                            }
                        });

                        if quads_to_draw != 0 {
                            draw_arrays_triangles(gl, quads_to_draw * 6);
                        }
                    }

                    check_error(gl);
                });
            });
        }
    }
}

struct CompiledProgram {
    program: GlProgram,
    layouts: SecondaryMap<DefaultKey, compiler::serialize::ShaderDataLayout>,

    uni_buffer_offset_instance: GlUniformLoc,
    uni_buffer_offset_data: GlUniformLoc,
    uni_frame_resolution: GlUniformLoc,
    uni_frame_screen: GlUniformLoc,
}

impl CompiledProgram {
    fn compile<'a>(
        gl: GlContext<'a>,
        info: &'a GlInfo,
        shaders: impl IntoIterator<Item = (Shader, &'a Graph)>,
    ) -> Self {
        let compiled = {
            let mut compiler = compiler::Compiler::new(compiler::CompilerOptions {
                glsl_version: info.glsl_version(),
                texture_units: info.max_texture_image_units as u32,
            });

            for (id, shader) in shaders.into_iter() {
                compiler.put_shader(id, shader);
            }

            compiler.compile()
        };

        let program = GlProgram::new(gl, &compiled.shader_vertex, &compiled.shader_fragment);
        program.bind(gl);

        uniform_1i(
            gl,
            program.get_uniform_loc(gl, compiler::UNIFORM_BUFFER_OBJECT),
            0, //texture location 0
        );

        for i in 1..info.max_texture_image_units {
            uniform_1i(
                gl,
                program.get_uniform_loc_array(gl, compiler::UNIFORM_TEXTURE_SAMPLERS, i - 1),
                i as _,
            );
        }

        Self {
            uni_buffer_offset_instance: program.get_uniform_loc(gl, compiler::UNIFORM_BUFFER_OFFSET_INSTANCE),
            uni_buffer_offset_data: program.get_uniform_loc(gl, compiler::UNIFORM_BUFFER_OFFSET_DATA),
            uni_frame_resolution: program.get_uniform_loc(gl, compiler::UNIFORM_FRAME_RESOLUTION),
            uni_frame_screen: program.get_uniform_loc(gl, compiler::UNIFORM_FRAME_SCREEN),

            program,
            layouts: compiled
                .shader_layout
                .into_iter()
                .map(|(shader, layout)| (DefaultKey::from(KeyData::from_ffi(shader.0)), layout))
                .collect(),
        }
    }

    fn bind(&self, gl: GlContext) {
        self.program.bind(gl);
    }

    fn set_uniforms(&self, gl: GlContext, offset_quads_start: u32, offset_data_start: u32) {
        uniform_1i(gl, self.uni_buffer_offset_instance, offset_quads_start as i32);
        uniform_1i(gl, self.uni_buffer_offset_data, offset_data_start as i32);
    }
}

struct ResourceShader {
    graph: Graph,
}

struct ResourceTexture {
    texture: GlTexture,
}

struct ResourceFramebuffer {
    framebuffer: GlFramebuffer,
    width: u32,
    height: u32,
}

struct CommandStream<'a>(&'a [Command]);
impl<'a> CommandStream<'a> {
    fn peek(&self) -> Option<Command> {
        self.0.first().copied()
    }

    fn pop(&mut self) -> Option<Command> {
        let next = self.0.first().copied()?;
        self.0 = &self.0[1..];
        Some(next)
    }

    fn peek_quad(&self) -> &[Command] {
        match self.0.iter().position(|x| matches!(x, Command::EndQuad)) {
            Some(p) => &self.0[..p],
            None => &self.0,
        }
    }
}
