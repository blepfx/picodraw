mod bindings;

use bindings::*;
use std::{
    cell::Cell,
    collections::HashSet,
    ffi::{CStr, CString},
    mem::{MaybeUninit, forget, size_of},
    ops::Deref,
    ptr::null,
};

pub use bindings::GlBindings;

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

    pub fn get_uniform_loc_array(&self, gl: GlContext, name: &str, index: usize) -> GlUniformLoc {
        unsafe {
            let cname = CString::new(format!("{}[{}]", name, index)).unwrap();
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
                ptr: Cell::new(size),
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
                    | MAP_FLUSH_EXPLICIT_BIT
                    | (if needs_invalidation {
                        MAP_INVALIDATE_BUFFER_BIT
                    } else {
                        MAP_UNSYNCHRONIZED_BIT
                    }),
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
        (self.owner.size - self.owner.ptr.get()) * GlTextureBuffer::TEXEL_SIZE_BYTES
    }

    pub fn mark_full(&self) {
        self.owner.ptr.set(self.owner.size);
    }

    pub fn request(&self, byte_size: usize) -> &mut [MaybeUninit<u8>] {
        let texel_size = byte_size.div_ceil(GlTextureBuffer::TEXEL_SIZE_BYTES);
        if self.space_left() >= texel_size {
            unsafe {
                let ptr = self.buffer.add(self.owner.ptr.get() - self.start);
                self.owner.ptr.set(self.owner.ptr.get() + texel_size);

                std::slice::from_raw_parts_mut(ptr as *mut MaybeUninit<u8>, byte_size)
            }
        } else {
            panic!("overwrite");
        }
    }
}

pub struct GlTexture {
    texture: GLuint,
}

impl GlTexture {
    pub fn new(gl: GlContext, data: picodraw_core::ImageData) -> Self {
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
                match data.format {
                    picodraw_core::ImageFormat::R8 => R8,
                    picodraw_core::ImageFormat::RGB8 => RGB8,
                    picodraw_core::ImageFormat::RGBA8 => RGBA8,
                },
                data.width as _,
                data.height as _,
                0,
                match data.format {
                    picodraw_core::ImageFormat::R8 => RED,
                    picodraw_core::ImageFormat::RGB8 => RGB,
                    picodraw_core::ImageFormat::RGBA8 => RGBA,
                },
                UNSIGNED_BYTE,
                data.data.as_ptr() as *const _,
            );

            check_error(gl);
            forget(texture_drop);

            Self { texture }
        }
    }

    pub fn bind_texture(&self, gl: GlContext, id: u32) {
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

pub struct GlFramebuffer {
    framebuffer: GLuint,
    texture: GLuint,
}

impl GlFramebuffer {
    pub fn new(gl: GlContext, width: u32, height: u32) -> Self {
        unsafe {
            let mut texture = 0;
            let mut framebuffer = 0;

            gl.gen_textures(1, &mut texture);
            check_error(gl);

            let texture_drop = Defer(move || gl.delete_textures(1, &texture));

            gl.gen_framebuffers(1, &mut framebuffer);
            check_error(gl);

            let framebuffer_drop = Defer(move || gl.delete_framebuffers(1, &framebuffer));

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
                null(),
            );
            check_error(gl);

            gl.bind_framebuffer(FRAMEBUFFER, framebuffer);
            check_error(gl);

            gl.framebuffer_texture_2d(FRAMEBUFFER, COLOR_ATTACHMENT0, TEXTURE_2D, texture, 0);
            check_error(gl);

            if gl.check_framebuffer_status(FRAMEBUFFER) != FRAMEBUFFER_COMPLETE {
                panic!("picodraw internal error: framebuffer incomplete");
            }

            gl.bind_framebuffer(FRAMEBUFFER, 0);
            check_error(gl);

            forget(texture_drop);
            forget(framebuffer_drop);

            Self {
                texture,
                framebuffer,
            }
        }
    }

    pub fn bind(&self, gl: GlContext) {
        unsafe {
            gl.bind_framebuffer(FRAMEBUFFER, self.framebuffer);
        }
        check_error(gl);
    }

    pub fn bind_texture(&self, gl: GlContext, id: u32) {
        unsafe {
            gl.active_texture(TEXTURE0 + id);
            check_error(gl);

            gl.bind_texture(TEXTURE_2D, self.texture);
            check_error(gl);
        }
    }

    pub fn bind_default(gl: GlContext) {
        unsafe {
            gl.bind_framebuffer(FRAMEBUFFER, 0);
        }
        check_error(gl);
    }

    pub fn delete(self, gl: GlContext) {
        unsafe {
            gl.delete_framebuffers(1, &self.framebuffer);
            gl.delete_textures(1, &self.texture);
        }
    }
}

pub struct GlInfo {
    pub version: (i32, i32),
    pub extensions: HashSet<String>,
    pub max_texture_buffer_size: usize,
    pub max_texture_image_units: usize,
}

impl GlInfo {
    pub fn get(gl: GlContext) -> Option<Self> {
        unsafe {
            let version = {
                let mut version = (0, 0);
                gl.get_integer_v(MAJOR_VERSION, &mut version.0);
                gl.get_integer_v(MINOR_VERSION, &mut version.1);

                if gl.get_error() != NO_ERROR {
                    let vstr = CStr::from_ptr(gl.get_string(VERSION) as *const _).to_string_lossy();
                    let mut vstr = vstr.split(&[' ', '.']);

                    version.0 = vstr.next().and_then(|x| x.parse::<i32>().ok()).unwrap_or(1);
                    version.1 = vstr.next().and_then(|x| x.parse::<i32>().ok()).unwrap_or(0);
                }

                if version.0 < 1 {
                    return None;
                }

                version
            };

            let extensions = if version >= (3, 0) {
                let mut extensions = HashSet::new();
                let mut ext_num = 0;

                gl.get_integer_v(NUM_EXTENSIONS, &mut ext_num);
                for i in 0..ext_num {
                    extensions.insert(
                        CStr::from_ptr(gl.get_stringi(EXTENSIONS, i as u32) as *const _)
                            .to_string_lossy()
                            .to_string(),
                    );
                }

                extensions
            } else {
                CStr::from_ptr(gl.get_string(EXTENSIONS) as *const _)
                    .to_string_lossy()
                    .split(' ')
                    .map(|x| x.to_string())
                    .collect()
            };

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
                extensions,
                max_texture_image_units: max_texture_image_units as usize,
                max_texture_buffer_size: max_texture_buffer_size as usize,
            })
        }
    }

    pub fn glsl_version(&self) -> u32 {
        if self.version >= (3, 3) {
            (self.version.0 * 100 + self.version.1 * 10) as u32
        } else if self.version >= (3, 2) {
            150
        } else if self.version >= (3, 1) {
            140
        } else if self.version >= (3, 0) {
            130
        } else if self.version >= (2, 1) {
            120
        } else if self.version >= (2, 0) {
            110
        } else {
            100
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
        gl.scissor(x as _, y as _, w as _, h as _);
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

pub fn clear_rect(gl: GlContext, x: i32, y: i32, w: u32, h: u32) {
    unsafe {
        gl.enable(SCISSOR_TEST);
        gl.scissor(x as _, y as _, w as _, h as _);
        gl.clear_color(0.0, 0.0, 0.0, 0.0);
        gl.clear(COLOR_BUFFER_BIT);
        gl.disable(SCISSOR_TEST);
    }
    check_error(gl);
}

pub fn screenshot_rect(gl: GlContext, x: i32, y: i32, w: u32, h: u32) -> Vec<u32> {
    unsafe {
        let mut data = vec![0; (w * h) as usize];
        gl.read_pixels(
            x as _,
            y as _,
            w as _,
            h as _,
            RGBA,
            UNSIGNED_BYTE,
            data.as_mut_ptr() as *mut _,
        );
        data
    }
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
