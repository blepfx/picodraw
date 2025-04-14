use super::bindings::*;
use std::{
    cell::Cell,
    ffi::CString,
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
                    check_error(gl);
                    let shader_drop = Defer(|| gl.delete_shader(shader));

                    gl.shader_source(
                        shader,
                        1,
                        &(source.as_ptr() as *const GLchar),
                        &(source.len() as GLint),
                    );
                    check_error(gl);

                    gl.compile_shader(shader);
                    check_error(gl);

                    let mut success = 0;
                    gl.get_shader_iv(shader, COMPILE_STATUS, &mut success);
                    check_error(gl);

                    if success == 0 {
                        let mut max_length = 0;
                        gl.get_shader_iv(shader, INFO_LOG_LENGTH, &mut max_length);
                        check_error(gl);

                        let mut buffer = vec![0u8; max_length as usize];
                        gl.get_shader_info_log(
                            shader,
                            max_length,
                            &mut max_length,
                            buffer.as_mut_ptr() as *mut _,
                        );
                        check_error(gl);

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
            check_error(gl);
            let program_drop = Defer(|| gl.delete_program(program));

            gl.attach_shader(program, shader_vs);
            check_error(gl);

            gl.attach_shader(program, shader_fg);
            check_error(gl);

            gl.link_program(program);
            check_error(gl);

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

            check_error(gl);
            forget(program_drop);
            GlProgram { program }
        }
    }

    pub fn bind(&self, gl: GlContext) {
        unsafe {
            gl.use_program(self.program);
            check_error(gl);
        }
    }

    pub fn get_uniform_loc(&self, gl: GlContext, name: &str) -> GlUniformLoc {
        unsafe {
            let cname = CString::new(name).unwrap();
            let loc = gl.get_uniform_location(self.program, cname.as_ptr() as _);
            check_error(gl);
            GlUniformLoc(loc)
        }
    }

    pub fn delete(self, gl: GlContext) {
        unsafe {
            gl.delete_program(self.program);
            check_error(gl);
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
            check_error(gl);
            Self { vao }
        }
    }

    pub fn bind(&self, gl: GlContext) {
        unsafe {
            gl.bind_vertex_array(self.vao);
            check_error(gl);
        }
    }

    pub fn delete(self, gl: GlContext) {
        unsafe {
            gl.delete_vertex_arrays(1, &self.vao);
            check_error(gl);
        }
    }
}

pub struct GlTextureBuffer {
    tbo_buffer: GLuint,
    tbo_texture: GLuint,

    size: usize,
    ptr: Cell<usize>,
    invalidate: Cell<bool>,
}

impl GlTextureBuffer {
    pub const TEXEL_SIZE_BYTES: usize = size_of::<[u32; 4]>();

    pub fn new(gl: GlContext, size: usize) -> Self {
        unsafe {
            let mut tbo_buffer = 0;
            gl.gen_buffers(1, &mut tbo_buffer);
            check_error(gl);
            let tbo_buffer_drop = Defer(move || gl.delete_buffers(1, &tbo_buffer));

            gl.bind_buffer(TEXTURE_BUFFER, tbo_buffer);
            gl.buffer_data(
                TEXTURE_BUFFER,
                (Self::TEXEL_SIZE_BYTES * size) as _,
                null(),
                DYNAMIC_DRAW,
            );
            check_error(gl);

            let mut tbo_texture = 0;
            gl.gen_textures(1, &mut tbo_texture);
            check_error(gl);
            let tbo_texture_drop = Defer(move || gl.delete_textures(1, &tbo_texture));

            gl.bind_texture(TEXTURE_BUFFER, tbo_texture);
            check_error(gl);
            gl.tex_buffer(TEXTURE_BUFFER, RGBA32UI, tbo_buffer);
            check_error(gl);

            forget(tbo_texture_drop);
            forget(tbo_buffer_drop);

            Self {
                tbo_buffer,
                tbo_texture,
                size,
                ptr: Cell::new(0),
                invalidate: Cell::new(false),
            }
        }
    }

    pub fn bind_texture(&self, gl: GlContext, id: u32) {
        unsafe {
            gl.active_texture(TEXTURE0 + id);
            check_error(gl);

            gl.bind_texture(TEXTURE_BUFFER, self.tbo_texture);
            check_error(gl);
        }
    }

    pub fn update<R>(
        &mut self,
        gl: GlContext,
        c: impl for<'a> FnOnce(GlTextureBufferWriter<'a>) -> R,
    ) -> R {
        unsafe {
            gl.bind_buffer(TEXTURE_BUFFER, self.tbo_buffer);
            check_error(gl);

            if self.invalidate.replace(false) {
                self.ptr.set(0);
            }

            let range_start = self.ptr.get();
            let range_mapped = gl.map_buffer_range(
                TEXTURE_BUFFER,
                (range_start * Self::TEXEL_SIZE_BYTES) as _,
                ((self.size - range_start) * Self::TEXEL_SIZE_BYTES) as _,
                MAP_WRITE_BIT | MAP_FLUSH_EXPLICIT_BIT,
            ) as *mut [u32; 4];
            check_error(gl);

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
            check_error(gl);

            gl.unmap_buffer(TEXTURE_BUFFER);
            check_error(gl);

            result
        }
    }

    pub fn delete(self, gl: GlContext) {
        unsafe {
            gl.delete_textures(1, &self.tbo_texture);
            check_error(gl);

            gl.delete_buffers(1, &self.tbo_buffer);
            check_error(gl);
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

    pub fn invalidate(&self) {
        self.owner.invalidate.set(true);
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
            check_error(gl);

            let texture_drop = Defer(move || gl.delete_textures(1, &texture));

            gl.bind_texture(TEXTURE_2D, texture);
            check_error(gl);

            gl.tex_parameteri(TEXTURE_2D, TEXTURE_MIN_FILTER, LINEAR);
            gl.tex_parameteri(TEXTURE_2D, TEXTURE_MAG_FILTER, LINEAR);
            check_error(gl);

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

            check_error(gl);
            forget(texture_drop);

            Self { texture }
        }
    }

    pub fn bind(&self, gl: GlContext, id: u32) {
        unsafe {
            gl.active_texture(TEXTURE0 + id);
            gl.bind_texture(TEXTURE_2D, self.texture);
            check_error(gl);
        }
    }

    pub fn delete(self, gl: GlContext) {
        unsafe {
            gl.delete_textures(1, &self.texture);
            check_error(gl);
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
            check_error(gl);

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
                check_error(gl);

                c();

                gl.end_query(TIME_ELAPSED);
                check_error(gl);

                self.waiting.set(3);
                None
            } else if self.waiting.get() == 0 {
                c();

                let mut available = 0;

                gl.get_query_object_iv(self.query, QUERY_RESULT_AVAILABLE, &mut available);
                check_error(gl);

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
            check_error(gl);
        }
    }
}

pub struct GlInfo {
    pub version: (i32, i32),
    pub max_texture_size: usize,
    pub max_texture_buffer_size: usize,
}

impl GlInfo {
    pub fn get(gl: GlContext) -> Option<Self> {
        unsafe {
            clear_error(gl);

            let mut version = (0, 0);
            gl.get_integer_v(MAJOR_VERSION, &mut version.0);
            gl.get_integer_v(MINOR_VERSION, &mut version.1);
            if gl.get_error() != NO_ERROR {
                // failed to get the current opengl version??
                return None;
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
            check_error(gl);

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
            })
        }
    }
}

pub fn draw_arrays_triangles(gl: GlContext, count: usize) {
    unsafe {
        gl.draw_arrays(TRIANGLES, 0, count as _);
    }
    check_error(gl);
}

pub fn uniform_1i(gl: GlContext, uni: GlUniformLoc, value: i32) {
    unsafe {
        gl.uniform_1i(uni.0, value);
    }
    check_error(gl);
}

pub fn uniform_2f(gl: GlContext, uni: GlUniformLoc, value: [f32; 2]) {
    unsafe {
        gl.uniform_2f(uni.0, value[0], value[1]);
    }
    check_error(gl);
}

pub fn viewport(gl: GlContext, x: i32, y: i32, w: u32, h: u32) {
    unsafe {
        gl.viewport(x as _, y as _, w as _, h as _);
    }
    check_error(gl);
}

pub fn enable_blend_normal(gl: GlContext) {
    unsafe {
        gl.enable(BLEND);
        gl.blend_func_separate(SRC_ALPHA, ONE_MINUS_SRC_ALPHA, ONE, ONE_MINUS_SRC_ALPHA);
    }
    check_error(gl);
}

pub fn enable_framebuffer_srgb(gl: GlContext) {
    unsafe {
        gl.enable(FRAMEBUFFER_SRGB);
    }
    check_error(gl);
}

pub fn disable_framebuffer_srgb(gl: GlContext) {
    unsafe {
        gl.disable(FRAMEBUFFER_SRGB);
    }
    check_error(gl);
}

pub fn clear_color(gl: GlContext) {
    unsafe {
        gl.clear(COLOR_BUFFER_BIT);
    }
    check_error(gl);
}

pub fn bind_default_framebuffer(gl: GlContext) {
    unsafe {
        gl.bind_framebuffer(FRAMEBUFFER, 0);
    }
    check_error(gl);
}

pub fn clear_error(gl: GlContext) {
    unsafe { while gl.get_error() != NO_ERROR {} }
}

#[track_caller]
pub fn check_error(gl: GlContext) {
    if cfg!(not(debug_assertions)) {
        return;
    }

    unsafe {
        let err = match gl.get_error() {
            NO_ERROR => return,
            INVALID_ENUM => "GL_INVALID_ENUM",
            INVALID_VALUE => "GL_INVALID_VALUE",
            INVALID_OPERATION => "GL_INVALID_OPERATION",
            INVALID_INDEX => "GL_INVALID_INDEX",
            INVALID_FRAMEBUFFER_OPERATION => "GL_INVALID_FRAMEBUFFER_OPERATION",
            STACK_OVERFLOW => "GL_STACK_OVERFLOW",
            STACK_UNDERFLOW => "GL_STACK_UNDERFLOW",
            OUT_OF_MEMORY => panic!("picodraw opengl error: out of memory"),
            _ => "GL_UNKNOWN",
        };

        panic!("picodraw opengl internal error ({})", err);
    }
}

struct Defer<F: FnMut() -> ()>(F);
impl<F: FnMut() -> ()> Drop for Defer<F> {
    fn drop(&mut self) {
        (self.0)()
    }
}
