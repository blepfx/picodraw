use crate::{
    compiler,
    opengl::{
        BUFFER_ALIGNMENT, GlFramebufferBinding, GlProgramBinding, GlStreamUBO, GlTextureRender, GlVertexArrayBinding,
        viewport,
    },
};
use glow::HasContext;
use picodraw_core::{Bounds, Size};

pub struct DispatcherScratch<T: HasContext> {
    drawcall_data: Vec<u8>,
    drawcall_quads: Vec<compiler::serialize::QuadDescriptorStruct>,
    drawcall_textures: Vec<Option<T::Texture>>,
    quad_queue_data: Vec<u32>,
    quad_queue_textures: Vec<(u32, T::Texture)>,
}

impl<T: HasContext> Default for DispatcherScratch<T> {
    fn default() -> Self {
        Self {
            drawcall_data: Vec::new(),
            drawcall_quads: Vec::new(),
            drawcall_textures: Vec::new(),
            quad_queue_data: Vec::new(),
            quad_queue_textures: Vec::new(),
        }
    }
}

pub struct Dispatcher<'a, T: HasContext> {
    pub global_context: &'a T,
    pub global_program: &'a GlProgramBinding<'a, T>,
    pub global_vertex_array: &'a GlVertexArrayBinding<'a, T>,
    pub global_buffer_quadlist: &'a GlStreamUBO<T>,
    pub global_buffer_quaddata: &'a GlStreamUBO<T>,

    pub target_framebuffer: GlFramebufferBinding<'a, T>,
    pub target_framebuffer_screen: bool,
    pub target_framebuffer_size: Size,

    pub drawcall_data: &'a mut Vec<u8>,
    pub drawcall_quads: &'a mut Vec<compiler::serialize::QuadDescriptorStruct>,
    pub drawcall_textures: &'a mut Vec<Option<T::Texture>>,

    pub quad_queue_data: &'a mut Vec<u32>,
    pub quad_queue_textures: &'a mut Vec<(u32, T::Texture)>,
    pub quad_layout: Option<&'a compiler::serialize::ShaderDataLayout>,
    pub quad_bounds: Bounds,

    pub total_bytes_written: u64,
    pub total_quads_written: u32,
    pub total_drawcalls_issued: u32,
}

impl<'a, T: HasContext> Dispatcher<'a, T> {
    pub fn new(
        scratch: &'a mut DispatcherScratch<T>,
        global_context: &'a T,
        global_program: &'a GlProgramBinding<'a, T>,
        global_vertex_array: &'a GlVertexArrayBinding<'a, T>,
        global_buffer_quadlist: &'a GlStreamUBO<T>,
        global_buffer_quaddata: &'a GlStreamUBO<T>,
    ) -> Self {
        let target_framebuffer = GlFramebufferBinding::default(global_context);
        let target_framebuffer_screen = true;
        let target_framebuffer_size = Size { width: 0, height: 0 };

        let drawcall_data = &mut scratch.drawcall_data;
        let drawcall_quads = &mut scratch.drawcall_quads;
        let drawcall_textures = &mut scratch.drawcall_textures;

        let quad_queue_data = &mut scratch.quad_queue_data;
        let quad_queue_textures = &mut scratch.quad_queue_textures;

        Self {
            global_context,
            global_program,
            global_vertex_array,
            global_buffer_quadlist,
            global_buffer_quaddata,

            target_framebuffer,
            target_framebuffer_screen,
            target_framebuffer_size,

            drawcall_data,
            drawcall_quads,
            drawcall_textures,

            quad_queue_data,
            quad_queue_textures,
            quad_layout: None,
            quad_bounds: Bounds {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            },

            total_bytes_written: 0,
            total_quads_written: 0,
            total_drawcalls_issued: 0,
        }
    }

    pub fn set_target_backbuffer(&mut self, size: Size) {
        self.flush();

        self.target_framebuffer = GlFramebufferBinding::default(self.global_context);
        self.target_framebuffer_screen = true;
        self.target_framebuffer_size = size;

        self.global_program
            .set_uniform_f32x2(0, size.width as f32, size.height as f32);
        self.global_program.set_uniform_i32(1, 1);

        viewport(self.global_context, 0, 0, size.width, size.height);
    }

    pub fn set_target_texture(&mut self, texture: &'a GlTextureRender<T>, size: Size) {
        self.flush();

        self.target_framebuffer = texture.bind(self.global_context);
        self.target_framebuffer_screen = false;
        self.target_framebuffer_size = size;

        self.global_program
            .set_uniform_f32x2(0, size.width as f32, size.height as f32);
        self.global_program.set_uniform_i32(1, 0);

        viewport(self.global_context, 0, 0, size.width, size.height);
    }

    pub fn clear_rect(&mut self, bounds: Bounds) {
        self.flush();

        if self.target_framebuffer_screen {
            self.target_framebuffer.clear(
                bounds.left as _,
                (self.target_framebuffer_size.height as i32 - bounds.bottom as i32) as _,
                bounds.width() as _,
                bounds.height() as _,
            );
        } else {
            self.target_framebuffer.clear(
                bounds.left as _,
                bounds.top as _,
                bounds.width() as _,
                bounds.height() as _,
            );
        }
    }

    pub fn quad_start(&mut self, layout: &'a compiler::serialize::ShaderDataLayout, bounds: Bounds) {
        self.quad_layout = Some(layout);
        self.quad_bounds = bounds;
        self.quad_queue_data.clear();
        self.quad_queue_textures.clear();
    }

    pub fn quad_end(&mut self) {
        let layout = self.quad_layout.expect("quad_end called without quad_start");
        let bounds = self.quad_bounds;

        let buffer_quaddata_fits =
            self.drawcall_data.len() + layout.size as usize <= self.global_buffer_quaddata.bytes_left() as usize;
        let buffer_quadlist_fits = (self.drawcall_quads.len() + 1) * compiler::serialize::QuadDescriptorStruct::SIZE
            <= self.global_buffer_quadlist.bytes_left() as usize;
        let can_bind_textures =
            self.quad_queue_textures
                .iter()
                .all(|(slot, tex)| match self.drawcall_textures.get(*slot as usize) {
                    Some(Some(existing_texture)) => *tex == *existing_texture,
                    Some(None) => true,
                    None => true,
                });

        if !buffer_quaddata_fits || !buffer_quadlist_fits || !can_bind_textures {
            self.flush();
        }

        let offset = self.drawcall_data.len();

        self.drawcall_quads.push(compiler::serialize::QuadDescriptorStruct {
            left: bounds.left.try_into().unwrap_or(u16::MAX),
            top: bounds.top.try_into().unwrap_or(u16::MAX),
            right: bounds.right.try_into().unwrap_or(u16::MAX),
            bottom: bounds.bottom.try_into().unwrap_or(u16::MAX),
            shader: layout.branch_id,
            offset: offset as u32 / BUFFER_ALIGNMENT,
        });

        self.drawcall_data.resize(offset + layout.size as usize, 0);

        compiler::serialize::encode(
            &mut self.drawcall_data[offset..],
            layout,
            self.quad_queue_data.drain(..),
        )
        .expect("malformed command stream");

        for (slot, texture) in self.quad_queue_textures.drain(..) {
            if self.drawcall_textures.len() <= slot as usize {
                self.drawcall_textures.resize(slot as usize + 1, None);
            }

            self.drawcall_textures[slot as usize] = Some(texture);
        }
    }

    pub fn quad_data(&mut self, data: u32) {
        self.quad_queue_data.push(data);
    }

    pub fn quad_texture(&mut self, texture: T::Texture) {
        let layout = self.quad_layout.expect("quad_texture called without quad_start");
        let slot = layout
            .textures
            .get(self.quad_queue_textures.len())
            .copied()
            .expect("malformed command stream");

        self.quad_queue_textures.push((slot, texture));
    }

    pub fn flush(&mut self) {
        if self.drawcall_quads.is_empty() {
            return;
        }

        let slice_quaddata = self.drawcall_data.as_slice();
        let slice_quadlist = compiler::serialize::QuadDescriptorStruct::as_byte_slice(self.drawcall_quads.as_slice());
        let range_quaddata = self.global_buffer_quaddata.write(self.global_context, slice_quaddata);
        let range_quadlist = self.global_buffer_quadlist.write(self.global_context, slice_quadlist);

        self.global_program
            .set_uniform_block_range(0, self.global_buffer_quaddata);
        self.global_program
            .set_uniform_block_range(1, self.global_buffer_quadlist);
        self.global_program
            .set_uniform_i32(2, (range_quaddata.start / BUFFER_ALIGNMENT) as i32);
        self.global_program
            .set_uniform_i32(3, (range_quadlist.start / BUFFER_ALIGNMENT) as i32);

        for (index, texture) in self.drawcall_textures.iter().enumerate() {
            if let Some(texture) = texture {
                self.global_program.set_sampler_texture(index as u32, *texture);
            }
        }

        self.global_vertex_array.draw_triangles(
            &self.target_framebuffer,
            &self.global_program,
            (self.drawcall_quads.len() * 6) as u32,
        );

        self.drawcall_data.clear();
        self.drawcall_quads.clear();
        self.drawcall_textures.clear();

        self.total_bytes_written += range_quaddata.len() as u64;
        self.total_bytes_written += range_quadlist.len() as u64;
        self.total_quads_written += self.drawcall_quads.len() as u32;
        self.total_drawcalls_issued += 1;
    }
}
