use super::{GlFramebufferBinding, GlStreamUBO};
use glow::{HasContext, TEXTURE_2D, TEXTURE0, TRIANGLES, UNIFORM_BUFFER};
use std::array::from_fn;

pub struct GlProgram<T: HasContext> {
    program: T::Program,
    uniforms: Vec<Option<T::UniformLocation>>,
}

pub struct GlProgramBinding<'a, T: HasContext> {
    program: &'a GlProgram<T>,
    gl: &'a T,
}

pub struct GlVertexArray<T: HasContext> {
    array: T::VertexArray,
}

pub struct GlVertexArrayBinding<'a, T: HasContext> {
    _array: &'a GlVertexArray<T>,
    gl: &'a T,
}

impl<T: HasContext> GlProgram<T> {
    pub fn compile(gl: &T, vertex_shader: &str, fragment_shader: &str) -> Self {
        unsafe {
            let program = gl.create_program().unwrap();
            let shader_sources = [
                ("vertex", glow::VERTEX_SHADER, vertex_shader),
                ("fragment", glow::FRAGMENT_SHADER, fragment_shader),
            ];

            let shaders: [T::Shader; 2] = from_fn(|i| {
                let (shader_type_str, shader_type, shader_source) = shader_sources[i];

                let shader = gl.create_shader(shader_type).expect("Cannot create shader");
                gl.shader_source(shader, shader_source);
                gl.compile_shader(shader);

                if !gl.get_shader_compile_status(shader) {
                    panic!("{} {}", shader_type_str, gl.get_shader_info_log(shader));
                }

                gl.attach_shader(program, shader);
                shader
            });

            gl.link_program(program);
            if !gl.get_program_link_status(program) {
                panic!("linking {}", gl.get_program_info_log(program));
            }

            for shader in shaders {
                gl.detach_shader(program, shader);
                gl.delete_shader(shader);
            }

            Self {
                program,
                uniforms: Vec::new(),
            }
        }
    }

    pub fn set_uniform_binding(&mut self, gl: &T, name: &str, binding_index: u32) {
        unsafe {
            if let Some(location) = gl.get_uniform_location(self.program, name) {
                if self.uniforms.len() <= binding_index as usize {
                    self.uniforms.resize(binding_index as usize + 1, None);
                }

                self.uniforms[binding_index as usize] = Some(location);
            }
        }
    }

    pub fn set_uniform_block_binding(&self, gl: &T, name: &str, binding_index: u32) {
        unsafe {
            if let Some(location) = gl.get_uniform_block_index(self.program, name) {
                gl.uniform_block_binding(self.program, location, binding_index);
            }
        }
    }

    pub fn set_texture_sampler_binding(&self, gl: &T, name: &str, binding_index: u32) {
        unsafe {
            if let Some(location) = gl.get_uniform_location(self.program, name) {
                gl.uniform_1_i32(Some(&location), binding_index as i32);
            }
        }
    }

    pub fn bind<'a>(&'a self, gl: &'a T) -> GlProgramBinding<'a, T> {
        unsafe {
            gl.use_program(Some(self.program));
        }

        GlProgramBinding { program: self, gl }
    }

    pub fn delete(self, gl: &T) {
        unsafe {
            gl.delete_program(self.program);
        }
    }
}

impl<T: HasContext> GlVertexArray<T> {
    pub fn new(gl: &T) -> Self {
        unsafe {
            let array = gl.create_vertex_array().unwrap();
            Self { array }
        }
    }

    pub fn bind<'a>(&'a self, gl: &'a T) -> GlVertexArrayBinding<'a, T> {
        unsafe {
            gl.bind_vertex_array(Some(self.array));
        }

        GlVertexArrayBinding { _array: self, gl }
    }

    pub fn delete(self, gl: &T) {
        unsafe {
            gl.delete_vertex_array(self.array);
        }
    }
}

impl<'a, T: HasContext> GlProgramBinding<'a, T> {
    pub fn set_uniform_i32(&self, binding_index: u32, value: i32) {
        unsafe {
            if let Some(Some(location)) = self.program.uniforms.get(binding_index as usize) {
                self.gl.uniform_1_i32(Some(location), value);
            }
        }
    }

    pub fn set_uniform_f32x2(&self, binding_index: u32, value0: f32, value1: f32) {
        unsafe {
            if let Some(Some(location)) = self.program.uniforms.get(binding_index as usize) {
                self.gl.uniform_2_f32(Some(location), value0, value1);
            }
        }
    }

    pub fn set_uniform_block_range(&self, binding_index: u32, buffer: &'a GlStreamUBO<T>) {
        unsafe {
            self.gl
                .bind_buffer_base(UNIFORM_BUFFER, binding_index, Some(buffer.ubo_buffer));
        }
    }

    pub fn set_sampler_texture(&self, sampler_index: u32, texture: T::Texture) {
        unsafe {
            self.gl.active_texture(TEXTURE0 + sampler_index);
            self.gl.bind_texture(TEXTURE_2D, Some(texture));
        }
    }
}

impl<'a, T: HasContext> GlVertexArrayBinding<'a, T> {
    pub fn draw_triangles(&self, _fb: &GlFramebufferBinding<'a, T>, _program: &GlProgramBinding<'a, T>, count: u32) {
        unsafe {
            self.gl.draw_arrays(TRIANGLES, 0, count as _);
        }
    }
}
