use std::time::Instant;

use crate::{
    compiler::CompilerShader,
    opengl::{
        BUFFER_ALIGNMENT, GlFramebufferBinding, GlProgramBinding, GlStreamBuffer, GlStreamBufferResource, GlTexture,
        GlVertexArrayBinding, viewport,
    },
};
use glow::HasContext;
use picodraw_core::{Bounds, Color, Size};

pub struct DispatcherScratch<T: HasContext> {
    drawcall_data: Vec<u8>,
    drawcall_quads: Vec<GpuQuadDescriptor>,
    drawcall_textures: Vec<Option<T::Texture>>,

    object_data: Vec<u8>,
    object_rects: Vec<Bounds>,
    object_textures: Vec<T::Texture>,
}

impl<T: HasContext> Default for DispatcherScratch<T> {
    fn default() -> Self {
        Self {
            drawcall_data: Vec::new(),
            drawcall_quads: Vec::new(),
            drawcall_textures: Vec::new(),
            object_data: Vec::new(),
            object_rects: Vec::new(),
            object_textures: Vec::new(),
        }
    }
}

pub struct Dispatcher<'a, T: HasContext> {
    global_context: &'a T,
    global_program: GlProgramBinding<'a, T>,
    global_vertex_array: GlVertexArrayBinding<'a, T>,
    global_buffer: &'a GlStreamBuffer<T>,

    target_framebuffer: GlFramebufferBinding<'a, T>,
    target_framebuffer_screen: bool,
    target_framebuffer_size: Size,

    drawcall_data: &'a mut Vec<u8>,
    drawcall_quads: &'a mut Vec<GpuQuadDescriptor>,
    drawcall_textures: &'a mut Vec<Option<T::Texture>>,

    object_data: &'a mut Vec<u8>,
    object_rects: &'a mut Vec<Bounds>,
    object_textures: &'a mut Vec<T::Texture>,

    pub total_bytes_written: u64,
    pub total_quads_written: u32,
    pub total_objects_written: u32,
    pub total_pixels_written: u64,
    pub total_drawcalls_issued: u32,

    pub build_time_begin: Instant,
}

impl<'a, T: HasContext> Dispatcher<'a, T> {
    pub fn new(
        scratch: &'a mut DispatcherScratch<T>,
        global_context: &'a T,
        global_program: GlProgramBinding<'a, T>,
        global_vertex_array: GlVertexArrayBinding<'a, T>,
        global_buffer: &'a GlStreamBuffer<T>,
    ) -> Self {
        scratch.drawcall_data.clear();
        scratch.drawcall_quads.clear();
        scratch.drawcall_textures.clear();
        scratch.object_data.clear();
        scratch.object_rects.clear();
        scratch.object_textures.clear();

        Self {
            global_context,
            global_program,
            global_vertex_array,
            global_buffer,

            target_framebuffer: GlFramebufferBinding::default(global_context),
            target_framebuffer_screen: true,
            target_framebuffer_size: Size { width: 0, height: 0 },

            drawcall_data: &mut scratch.drawcall_data,
            drawcall_quads: &mut scratch.drawcall_quads,
            drawcall_textures: &mut scratch.drawcall_textures,

            object_data: &mut scratch.object_data,
            object_rects: &mut scratch.object_rects,
            object_textures: &mut scratch.object_textures,

            total_bytes_written: 0,
            total_quads_written: 0,
            total_objects_written: 0,
            total_pixels_written: 0,
            total_drawcalls_issued: 0,

            build_time_begin: Instant::now(),
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

    pub fn set_target_texture(&mut self, texture: &'a GlTexture<T>) {
        self.flush();

        let (width, height) = texture.size();

        self.target_framebuffer = texture.bind_framebuffer(self.global_context);
        self.target_framebuffer_screen = false;
        self.target_framebuffer_size = Size { width, height };

        self.global_program.set_uniform_f32x2(0, width as f32, height as f32);
        self.global_program.set_uniform_i32(1, 0);

        viewport(self.global_context, 0, 0, width, height);
    }

    pub fn push_clear(&mut self, bounds: Bounds, color: Color) {
        self.flush();

        if self.target_framebuffer_screen {
            self.target_framebuffer.clear(
                bounds.left as _,
                (self.target_framebuffer_size.height as i32 - bounds.bottom as i32) as _,
                bounds.width() as _,
                bounds.height() as _,
                color,
            );
        } else {
            self.target_framebuffer.clear(
                bounds.left as _,
                bounds.top as _,
                bounds.width() as _,
                bounds.height() as _,
                color,
            );
        }
    }

    pub fn push_object_texture(&mut self, texture: T::Texture) {
        self.object_textures.push(texture);
    }

    pub fn push_object_rect(&mut self, rect: Bounds) {
        self.object_rects.push(rect);
    }

    pub fn push_object_data(&mut self, data: &[u8]) {
        self.object_data.extend_from_slice(data);
    }

    pub fn push_object(&mut self, shader: &CompilerShader) {
        let object_data_size_aligned = self.object_data.len().next_multiple_of(16);
        if self.object_data.len() != object_data_size_aligned {
            self.object_data.resize(object_data_size_aligned, 0);
        }

        let buffer_fits = (self.drawcall_data.len() + self.object_data.len())
            + (self.drawcall_quads.len() + self.object_rects.len()) * GpuQuadDescriptor::SIZE
            <= self.global_buffer.bytes_left() as usize;

        let can_bind_textures = self
            .object_textures
            .iter()
            .zip(shader.texture_slots.iter())
            .all(|(tex, slot)| match self.drawcall_textures.get(*slot as usize) {
                Some(Some(existing_texture)) => *tex == *existing_texture,
                _ => true,
            });

        if !buffer_fits || !can_bind_textures {
            self.flush();
        }

        let offset = self.drawcall_data.len();
        self.drawcall_data.extend_from_slice(self.object_data);

        for bounds in self.object_rects.iter() {
            self.total_pixels_written += bounds.width() as u64 * bounds.height() as u64;
            self.drawcall_quads.push(GpuQuadDescriptor {
                left: bounds.left.try_into().unwrap_or(u16::MAX),
                top: bounds.top.try_into().unwrap_or(u16::MAX),
                right: bounds.right.try_into().unwrap_or(u16::MAX),
                bottom: bounds.bottom.try_into().unwrap_or(u16::MAX),
                shader: shader.index,
                offset: offset as u32 / BUFFER_ALIGNMENT,
            });
        }

        for (tex, slot) in self.object_textures.iter().zip(shader.texture_slots.iter()) {
            if self.drawcall_textures.len() <= *slot as usize {
                self.drawcall_textures.resize(*slot as usize + 1, None);
            }

            self.drawcall_textures[*slot as usize] = Some(*tex);
        }

        self.object_data.clear();
        self.object_rects.clear();
        self.object_textures.clear();
        self.total_objects_written += 1;
    }

    pub fn flush(&mut self) {
        if self.drawcall_quads.is_empty() {
            return;
        }

        let (start_data, start_list) = {
            let range_data = self.global_buffer.write(self.global_context, self.drawcall_data);
            let range_quad = self.global_buffer.write(
                self.global_context,
                GpuQuadDescriptor::as_byte_slice(self.drawcall_quads.as_slice()),
            );

            (range_data.start, range_quad.start)
        };

        self.global_program
            .set_uniform_i32(2, (start_data / BUFFER_ALIGNMENT) as i32);
        self.global_program
            .set_uniform_i32(3, (start_list / BUFFER_ALIGNMENT) as i32);

        match self.global_buffer.resource() {
            GlStreamBufferResource::Texture(texture) => {
                self.global_program.set_buffer_texture(0, texture);

                for (index, texture) in self.drawcall_textures.iter().enumerate() {
                    if let Some(texture) = texture {
                        self.global_program.set_sampler_texture(index as u32 + 1, *texture);
                    }
                }
            }
            GlStreamBufferResource::UniformBlock(buffer) => {
                self.global_program.set_uniform_block(0, buffer);

                for (index, texture) in self.drawcall_textures.iter().enumerate() {
                    if let Some(texture) = texture {
                        self.global_program.set_sampler_texture(index as u32, *texture);
                    }
                }
            }
        }

        self.global_vertex_array.draw_triangles(
            &self.target_framebuffer,
            &self.global_program,
            (self.drawcall_quads.len() * 6) as u32,
        );

        self.total_bytes_written += (self.drawcall_quads.len() * GpuQuadDescriptor::SIZE) as u64;
        self.total_bytes_written += self.drawcall_data.len() as u64;
        self.total_quads_written += self.drawcall_quads.len() as u32;
        self.total_drawcalls_issued += 1;

        self.drawcall_data.clear();
        self.drawcall_quads.clear();
        self.drawcall_textures.clear();
    }
}

#[repr(C)]
struct GpuQuadDescriptor {
    pub left: u16,
    pub top: u16,
    pub right: u16,
    pub bottom: u16,
    pub shader: u32,
    pub offset: u32,
}

impl GpuQuadDescriptor {
    pub const SIZE: usize = std::mem::size_of::<Self>();

    pub fn as_byte_slice(slice: &[Self]) -> &[u8] {
        let len = Self::SIZE * slice.len();
        let ptr = slice.as_ptr() as *const u8;
        unsafe { std::slice::from_raw_parts(ptr, len) }
    }
}
