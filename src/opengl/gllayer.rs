use super::bindings::*;
use std::{
    cell::Cell,
    ffi::{c_void, CStr, CString, OsStr},
    marker::PhantomData,
    mem::{forget, size_of},
    ops::Deref,
    ptr::{copy_nonoverlapping, null},
};

#[derive(Clone, Copy)]
pub struct GlContext<'a>(&'a GlBindings);

impl<'a> GlContext<'a> {
    pub unsafe fn within<R>(
        bindings: &'a GlBindings,
        c: impl for<'x> FnOnce(GlContext<'x>) -> R,
    ) -> R {
        c(GlContext(bindings))
    }
}

impl<'a> Deref for GlContext<'a> {
    type Target = GlBindings;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Copy)]
pub struct GlUniformLoc(GLint);
pub struct GlProgram {
    program: GLuint,
}

impl GlProgram {
    pub fn new(gl: GlContext, vertex: &str, fragment: &str) -> GlProgram {
        unsafe {
            unsafe fn new_shader(gl: GlContext, source: &str, type_: GLenum) -> GLuint {
                unsafe {
                    let shader = gl.create_shader(type_);

                    let shader_drop = Defer(|| gl.delete_shader(shader));

                    gl.shader_source(
                        shader,
                        1,
                        &(source.as_ptr() as *const GLchar),
                        &(source.len() as GLint),
                    );

                    gl.compile_shader(shader);

                    let mut success = 0;
                    gl.get_shader_iv(shader, COMPILE_STATUS, &mut success);

                    if success == 0 {
                        let mut max_length = 0;
                        gl.get_shader_iv(shader, INFO_LOG_LENGTH, &mut max_length);

                        let mut buffer = vec![0u8; max_length as usize];
                        gl.get_shader_info_log(
                            shader,
                            max_length,
                            &mut max_length,
                            buffer.as_mut_ptr() as *mut _,
                        );

                        panic!(
                            "picodraw opengl internal error ({} shader compilation)\n {}",
                            if type_ == VERTEX_SHADER {
                                "vertex"
                            } else {
                                "fragment"
                            },
                            String::from_utf8_lossy(&buffer)
                        );
                    }

                    forget(shader_drop);
                    shader
                }
            }

            let shader_vs = new_shader(gl, vertex, VERTEX_SHADER);
            let _shaders_fg_drop = Defer(|| gl.delete_shader(shader_vs));

            let shader_fg = new_shader(gl, fragment, FRAGMENT_SHADER);
            let _shaders_fg_drop = Defer(|| gl.delete_shader(shader_fg));

            let program = gl.create_program();

            let program_drop = Defer(|| gl.delete_program(program));

            gl.attach_shader(program, shader_vs);

            gl.attach_shader(program, shader_fg);

            gl.link_program(program);

            let mut success = 0;
            gl.get_program_iv(program, LINK_STATUS, &mut success);
            if success == 0 {
                let mut max_length = 0;
                gl.get_program_iv(program, INFO_LOG_LENGTH, &mut max_length);

                let mut buffer = vec![0u8; max_length as usize];
                gl.get_program_info_log(
                    program,
                    max_length,
                    &mut max_length,
                    buffer.as_mut_ptr() as *mut _,
                );

                panic!(
                    "picodraw opengl internal error (shader linking)\n {}",
                    String::from_utf8_lossy(&buffer)
                );
            }

            forget(program_drop);
            GlProgram { program }
        }
    }

    pub fn bind(&self, gl: GlContext) {
        unsafe {
            gl.use_program(self.program);
        }
    }

    pub fn get_uniform_loc(&self, gl: GlContext, name: &str) -> GlUniformLoc {
        unsafe {
            let cname = CString::new(name).unwrap();
            let loc = gl.get_uniform_location(self.program, cname.as_ptr() as _);

            GlUniformLoc(loc)
        }
    }

    pub fn delete(self, gl: GlContext) {
        unsafe {
            gl.delete_program(self.program);
        }
    }
}

pub struct GlVertexArrayObject {
    vao: GLuint,
}

impl GlVertexArrayObject {
    pub fn new(gl: GlContext) -> Self {
        unsafe {
            let mut vao = 0;
            gl.gen_vertex_arrays(1, &mut vao);

            Self { vao }
        }
    }

    pub fn bind(&self, gl: GlContext) {
        unsafe {
            gl.bind_vertex_array(self.vao);
        }
    }

    pub fn delete(self, gl: GlContext) {
        unsafe {
            gl.delete_vertex_arrays(1, &self.vao);
        }
    }
}

pub struct GlTextureBuffer {
    tbo_buffer: GLuint,
    tbo_texture: GLuint,

    size: usize,
    ptr: Cell<usize>,
}

impl GlTextureBuffer {
    pub const TEXEL_SIZE_BYTES: usize = size_of::<[u32; 4]>();

    pub fn new(gl: GlContext, size: usize) -> Self {
        unsafe {
            let mut tbo_buffer = 0;
            gl.gen_buffers(1, &mut tbo_buffer);

            let tbo_buffer_drop = Defer(move || gl.delete_buffers(1, &tbo_buffer));

            gl.bind_buffer(TEXTURE_BUFFER, tbo_buffer);
            gl.buffer_data(
                TEXTURE_BUFFER,
                (Self::TEXEL_SIZE_BYTES * size) as _,
                null(),
                STREAM_DRAW,
            );

            let mut tbo_texture = 0;
            gl.gen_textures(1, &mut tbo_texture);

            let tbo_texture_drop = Defer(move || gl.delete_textures(1, &tbo_texture));

            gl.bind_texture(TEXTURE_BUFFER, tbo_texture);

            gl.tex_buffer(TEXTURE_BUFFER, RGBA32UI, tbo_buffer);

            forget(tbo_texture_drop);
            forget(tbo_buffer_drop);

            Self {
                tbo_buffer,
                tbo_texture,
                size,
                ptr: Cell::new(0),
            }
        }
    }

    pub fn bind_texture(&self, gl: GlContext, id: u32) {
        unsafe {
            gl.active_texture(TEXTURE0 + id);

            gl.bind_texture(TEXTURE_BUFFER, self.tbo_texture);
        }
    }

    pub fn update<R>(
        &self,
        gl: GlContext,
        c: impl for<'a> FnOnce(GlTextureBufferWriter<'a>) -> R,
    ) -> R {
        unsafe {
            gl.bind_buffer(TEXTURE_BUFFER, self.tbo_buffer);

            let needs_invalidation = if self.ptr.get() == self.size {
                self.ptr.set(0);
                true
            } else {
                false
            };

            let range_start = self.ptr.get();
            let range_mapped = gl.map_buffer_range(
                TEXTURE_BUFFER,
                (range_start * Self::TEXEL_SIZE_BYTES) as _,
                ((self.size - range_start) * Self::TEXEL_SIZE_BYTES) as _,
                MAP_WRITE_BIT
                    | MAP_UNSYNCHRONIZED_BIT
                    | MAP_FLUSH_EXPLICIT_BIT
                    | (if needs_invalidation {
                        MAP_INVALIDATE_BUFFER_BIT
                    } else {
                        0
                    }),
            ) as *mut [u32; 4];

            let result = c(GlTextureBufferWriter {
                owner: self,
                start: range_start,
                buffer: range_mapped,
            });

            gl.flush_mapped_buffer_range(
                TEXTURE_BUFFER,
                0,
                (Self::TEXEL_SIZE_BYTES * (self.ptr.get() - range_start)) as _,
            );

            gl.unmap_buffer(TEXTURE_BUFFER);

            result
        }
    }

    pub fn delete(self, gl: GlContext) {
        unsafe {
            gl.delete_textures(1, &self.tbo_texture);

            gl.delete_buffers(1, &self.tbo_buffer);
        }
    }
}

pub struct GlTextureBufferWriter<'a> {
    owner: &'a GlTextureBuffer,
    start: usize,
    buffer: *mut [u32; 4],
}

impl<'a> GlTextureBufferWriter<'a> {
    pub fn pointer(&self) -> usize {
        self.owner.ptr.get()
    }

    pub fn space_left(&self) -> usize {
        self.owner.size - self.owner.ptr.get()
    }

    pub fn mark_full(&self) {
        self.owner.ptr.set(self.owner.size);
    }

    pub fn write(&self, data: &[[u32; 4]]) {
        if self.owner.ptr.get() + data.len() <= self.owner.size {
            unsafe {
                copy_nonoverlapping(
                    data.as_ptr(),
                    self.buffer.add(self.owner.ptr.get() - self.start),
                    data.len(),
                );
            }

            self.owner.ptr.set(self.owner.ptr.get() + data.len());
        } else {
            panic!("overwrite");
        }
    }
}

pub struct GlTexture {
    texture: GLuint,
}

impl GlTexture {
    pub fn new(gl: GlContext, width: u32, height: u32, data: &[u8]) -> Self {
        unsafe {
            let mut texture = 0;

            gl.gen_textures(1, &mut texture);

            let texture_drop = Defer(move || gl.delete_textures(1, &texture));

            gl.bind_texture(TEXTURE_2D, texture);

            gl.tex_parameteri(TEXTURE_2D, TEXTURE_MIN_FILTER, LINEAR);
            gl.tex_parameteri(TEXTURE_2D, TEXTURE_MAG_FILTER, LINEAR);

            gl.tex_image_2d(
                TEXTURE_2D,
                0,
                RGBA8,
                width as _,
                height as _,
                0,
                RGBA,
                UNSIGNED_BYTE,
                data.as_ptr() as *const _,
            );

            forget(texture_drop);

            Self { texture }
        }
    }

    pub fn bind(&self, gl: GlContext, id: u32) {
        unsafe {
            gl.active_texture(TEXTURE0 + id);
            gl.bind_texture(TEXTURE_2D, self.texture);
        }
    }

    pub fn delete(self, gl: GlContext) {
        unsafe {
            gl.delete_textures(1, &self.texture);
        }
    }
}

pub struct GlQuery {
    query: GLuint,
    waiting: Cell<u8>,
    phantom: PhantomData<*const ()>,
}

impl GlQuery {
    pub fn new(gl: GlContext) -> Self {
        unsafe {
            let mut query = 0;
            gl.gen_queries(1, &mut query);

            Self {
                query,
                waiting: Cell::new(255),
                phantom: PhantomData,
            }
        }
    }

    pub fn time_elapsed(&self, gl: GlContext, c: impl FnOnce()) -> Option<u64> {
        unsafe {
            if self.waiting.get() == 255 {
                gl.begin_query(TIME_ELAPSED, self.query);

                c();

                gl.end_query(TIME_ELAPSED);

                self.waiting.set(3);
                None
            } else if self.waiting.get() == 0 {
                c();

                let mut available = 0;

                gl.get_query_object_iv(self.query, QUERY_RESULT_AVAILABLE, &mut available);

                if available != 0 {
                    let mut elapsed = 0;

                    gl.get_query_object_ui64v(self.query, QUERY_RESULT, &mut elapsed);
                    gl.get_error();

                    self.waiting.set(255);
                    Some(elapsed)
                } else {
                    None
                }
            } else {
                c();
                self.waiting.set(self.waiting.get() - 1);
                None
            }
        }
    }

    pub fn delete(self, gl: GlContext) {
        unsafe {
            gl.delete_queries(1, &self.query);
        }
    }
}

pub struct GlInfo {
    pub version: (i32, i32),
    pub max_texture_size: usize,
    pub max_texture_buffer_size: usize,
    pub ext_khr_debug: bool,
}

impl GlInfo {
    pub fn get(gl: GlContext) -> Option<Self> {
        unsafe {
            let mut version = (0, 0);
            gl.get_error(); //clear errors
            gl.get_integer_v(MAJOR_VERSION, &mut version.0);
            gl.get_integer_v(MINOR_VERSION, &mut version.1);
            if gl.get_error() != NO_ERROR {
                // failed to get the current opengl version??
                return None;
            }

            if version <= (3, 0) {
                return None;
            }

            let mut ext_count = 0;
            let mut ext_khr_debug = false;

            gl.get_integer_v(NUM_EXTENSIONS, &mut ext_count);
            for i in 0..ext_count {
                let str = CStr::from_ptr(gl.get_string_i(EXTENSIONS, i) as *const _);
                if str == c"GL_KHR_debug" {
                    ext_khr_debug = true;
                }
            }

            // all of this is probably not necessary because of opengls min guarantees but its nice to have
            let mut max_texture_size = 0;
            let mut max_texture_image_units = 0;
            let mut max_texture_image_units_combined = 0;
            let mut max_texture_buffer_size = 0;

            gl.get_integer_v(MAX_TEXTURE_BUFFER_SIZE, &mut max_texture_buffer_size);
            gl.get_integer_v(MAX_TEXTURE_SIZE, &mut max_texture_size);
            gl.get_integer_v(MAX_TEXTURE_IMAGE_UNITS, &mut max_texture_image_units);
            gl.get_integer_v(
                MAX_COMBINED_TEXTURE_IMAGE_UNITS,
                &mut max_texture_image_units_combined,
            );

            // sanity checks
            if max_texture_image_units <= 0
                || max_texture_image_units_combined <= 0
                || max_texture_size <= 0
                || max_texture_buffer_size <= 0
            {
                return None;
            }

            Some(Self {
                version,
                max_texture_buffer_size: max_texture_buffer_size as usize,
                max_texture_size: max_texture_size as usize,
                ext_khr_debug,
            })
        }
    }
}

pub fn draw_arrays_triangles(gl: GlContext, count: usize) {
    unsafe {
        gl.draw_arrays(TRIANGLES, 0, count as _);
    }
}

pub fn uniform_1i(gl: GlContext, uni: GlUniformLoc, value: i32) {
    unsafe {
        gl.uniform_1i(uni.0, value);
    }
}

pub fn uniform_2f(gl: GlContext, uni: GlUniformLoc, value: [f32; 2]) {
    unsafe {
        gl.uniform_2f(uni.0, value[0], value[1]);
    }
}

pub fn viewport(gl: GlContext, x: i32, y: i32, w: u32, h: u32) {
    unsafe {
        gl.viewport(x as _, y as _, w as _, h as _);
    }
}

pub fn enable_blend_normal(gl: GlContext) {
    unsafe {
        gl.enable(BLEND);
        gl.blend_func_separate(SRC_ALPHA, ONE_MINUS_SRC_ALPHA, ONE, ONE_MINUS_SRC_ALPHA);
    }
}

pub fn enable_framebuffer_srgb(gl: GlContext) {
    unsafe {
        gl.enable(FRAMEBUFFER_SRGB);
    }
}

pub fn disable_framebuffer_srgb(gl: GlContext) {
    unsafe {
        gl.disable(FRAMEBUFFER_SRGB);
    }
}

pub fn clear_color(gl: GlContext) {
    unsafe {
        gl.clear(COLOR_BUFFER_BIT);
    }
}

pub fn bind_default_framebuffer(gl: GlContext) {
    unsafe {
        gl.bind_framebuffer(FRAMEBUFFER, 0);
    }
}

pub fn enable_debug_panics(gl: GlContext) {
    extern "system" fn callback(
        _source: GLenum,
        _gltype: GLenum,
        _id: GLuint,
        severity: GLenum,
        length: GLsizei,
        message: *const GLchar,
        _user_param: *mut c_void,
    ) {
        if severity != DEBUG_SEVERITY_HIGH {
            return;
        }

        unsafe {
            let message = OsStr::from_encoded_bytes_unchecked(std::slice::from_raw_parts(
                message as *const u8,
                length as usize,
            ))
            .to_string_lossy();

            panic!("OpenGL Error: {}", message);
        }
    }

    unsafe {
        gl.debug_message_callback(Some(callback), null());
        gl.enable(DEBUG_OUTPUT);
        gl.enable(DEBUG_OUTPUT_SYNCHRONOUS);
    }
}

struct Defer<F: FnMut() -> ()>(F);
impl<F: FnMut() -> ()> Drop for Defer<F> {
    fn drop(&mut self) {
        (self.0)()
    }
}
