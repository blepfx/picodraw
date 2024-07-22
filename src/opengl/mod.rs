mod bindings;
mod codegen;
mod gllayer;

use crate::{Bounds, Shader};
use bindings::GlBindings;
use codegen::{QuadEncoder, ShaderMap};
use gllayer::*;
use std::ffi::{c_void, CStr};

#[derive(Debug, Clone)]
pub struct GlStatistics {
    pub gpu_time_msec: f32,
    pub size_bytes: u64,
    pub area_pixels: u64,
    pub quads: u32,
    pub drawcalls: u32,
}

pub struct OpenGl {
    bindings: GlBindings,
    data: GlData,
}

struct GlData {
    program: Option<GlProgramData>,
    buffer: GlTextureBuffer,
    vao: GlVertexArrayObject,
    query: GlQuery,
    info: GlInfo,

    shaders: ShaderMap,
    pass_encoding: QuadEncoder,
    pass_viewport: Option<CurrentPass>,

    gpu_time: u64,
}

struct GlProgramData {
    program: GlProgram,
    atlas: GlTexture,

    uni_buffer_offset_instance: GlUniformLoc,
    uni_buffer_offset_data: GlUniformLoc,
    uni_resolution: GlUniformLoc,
}

pub struct OpenGlRenderer<'a> {
    data: &'a mut GlData,
}

impl OpenGl {
    pub unsafe fn new(f: &dyn Fn(&CStr) -> *const c_void) -> Self {
        let bindings = GlBindings::load_from(f);
        let data = GlContext::within(&bindings, |gl| GlData::new(gl));

        Self { bindings, data }
    }

    pub unsafe fn render(
        &mut self,
        width: u32,
        height: u32,
        c: impl for<'a> FnOnce(OpenGlRenderer<'a>),
    ) -> GlStatistics {
        GlContext::within(&self.bindings, |context| {
            self.data.begin_pass(width, height);
            c(OpenGlRenderer {
                data: &mut self.data,
            });
            self.data.end_pass(context)
        })
    }

    pub unsafe fn delete(self) {
        GlContext::within(&self.bindings, |gl| {
            self.data.delete(gl);
        })
    }
}

impl<'a> OpenGlRenderer<'a> {
    pub fn register<T: Shader>(&mut self) {
        self.data.shaders.register::<T>();
    }

    pub fn draw<T: Shader>(&mut self, drawable: &T, bounds: impl Into<Bounds>) {
        let pass = self
            .data
            .pass_viewport
            .as_ref()
            .expect("call begin_pass() first");

        self.data.shaders.write(
            &mut self.data.pass_encoding,
            bounds.into(),
            drawable,
            pass.width,
            pass.height,
        );
    }
}

impl GlData {
    fn begin_pass(&mut self, width: u32, height: u32) {
        self.pass_viewport = Some(CurrentPass { width, height });
    }

    fn end_pass(&mut self, gl: GlContext) -> GlStatistics {
        if self.shaders.is_dirty() {
            let (fragment_src, atlas) = self.shaders.recompile(self.info.max_texture_size as u32);

            if let Some(program) = self.program.take() {
                program.program.delete(gl);
                program.atlas.delete(gl);
            }

            let program = GlProgram::new(gl, codegen::VERTEX_SHADER, &fragment_src);
            program.bind(gl);

            gl_uniform_1i(
                gl,
                program.get_uniform_loc(gl, "uBuffer"),
                0, //texture location 0
            );

            gl_uniform_1i(
                gl,
                program.get_uniform_loc(gl, "uAtlas"),
                1, //texture location 0
            );

            let atlas_tex = atlas.create_image_rgba();
            let atlas = GlTexture::new(gl, atlas.size, atlas.size, &atlas_tex.as_raw());

            self.program = Some(GlProgramData {
                uni_buffer_offset_instance: program.get_uniform_loc(gl, "uBufferOffsetInstance"),
                uni_buffer_offset_data: program.get_uniform_loc(gl, "uBufferOffsetData"),
                uni_resolution: program.get_uniform_loc(gl, "uResolution"),
                program,
                atlas,
            })
        }

        let pass = self.pass_viewport.as_ref().unwrap();
        let program_data = self.program.as_ref().unwrap();
        program_data.program.bind(gl);
        program_data.atlas.bind(gl, 1);

        self.vao.bind(gl);
        self.buffer.bind_texture(gl, 0);

        gl_clear_color(gl);
        gl_enable_blend_normal(gl);
        gl_enable_framebuffer_srgb(gl);
        gl_viewport(gl, 0, 0, pass.width, pass.height);
        gl_uniform_2f(
            gl,
            program_data.uni_resolution,
            [pass.width as f32, pass.height as f32],
        );

        let mut stats_drawcalls = 0;
        let mut stats_quads = 0;

        self.gpu_time = self
            .query
            .time_elapsed(gl, || {
                let mut quads = 0;
                while quads < self.pass_encoding.quads.len() {
                    let quads_start = quads;

                    let (data_start, quad_data_start) = self.buffer.update(gl, |writer| {
                        let data_start = writer.pointer();
                        let local_data_start =
                            self.pass_encoding.quads[quads_start].data_range.start;
                        for quad in &self.pass_encoding.quads[quads_start..] {
                            if writer.space_left()
                                < quad.data_range.len() + 1 * (quads + 1 - quads_start)
                            {
                                break;
                            }

                            writer.write(&self.pass_encoding.data[quad.data_range.clone()]);
                            quads += 1;
                        }

                        let quad_data_start = writer.pointer();
                        if quads != quads_start {
                            for quad in &self.pass_encoding.quads[quads_start..quads] {
                                writer.write(&[[
                                    (quad.bounds[0] as u32) | ((quad.bounds[1] as u32) << 16),
                                    (quad.bounds[2] as u32) | ((quad.bounds[3] as u32) << 16),
                                    quad.shader_id,
                                    (quad.data_range.start - local_data_start) as u32,
                                ]]);
                            }
                        } else {
                            writer.mark_full();
                        }

                        (data_start, quad_data_start)
                    });

                    if quads != quads_start {
                        stats_quads += (quads - quads_start) as u32;
                        stats_drawcalls += 1;

                        gl_uniform_1i(
                            gl,
                            program_data.uni_buffer_offset_instance,
                            quad_data_start as i32,
                        );
                        gl_uniform_1i(gl, program_data.uni_buffer_offset_data, data_start as i32);
                        gl_draw_arrays_triangles(gl, (quads - quads_start) * 6);
                    }
                }
            })
            .unwrap_or(self.gpu_time);

        let stats = GlStatistics {
            gpu_time_msec: (self.gpu_time as f64 / 1e6) as f32,
            quads: stats_quads,
            drawcalls: stats_drawcalls,
            area_pixels: self.pass_encoding.total_area(),
            size_bytes: (self.pass_encoding.size_texels() * size_of::<[u32; 4]>()) as u64,
        };

        self.pass_viewport = None;
        self.pass_encoding.clear();

        stats
    }

    fn new(gl: GlContext) -> Self {
        let info = match GlInfo::get(gl) {
            Some(info) if info.version >= (3, 3) => info,
            _ => panic!("gl context is too old. target at least 3.3+"),
        };

        Self {
            gpu_time: 0,
            program: None,
            buffer: GlTextureBuffer::new(gl, info.max_texture_buffer_size.min(262144)),
            vao: GlVertexArrayObject::new(gl),
            query: GlQuery::new(gl),
            info,

            shaders: ShaderMap::new(),

            pass_encoding: QuadEncoder::new(),
            pass_viewport: None,
        }
    }

    fn delete(self, gl: GlContext) {
        if let Some(program) = self.program {
            program.program.delete(gl);
            program.atlas.delete(gl);
        }

        self.vao.delete(gl);
        self.buffer.delete(gl);
        self.query.delete(gl);
    }
}

struct CurrentPass {
    width: u32,
    height: u32,
}
