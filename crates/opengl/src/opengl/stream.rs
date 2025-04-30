use glow::{DYNAMIC_DRAW, HasContext, RGBA32UI, TEXTURE_BUFFER, UNIFORM_BUFFER};
use std::{cell::Cell, ops::Range};

pub const BUFFER_ALIGNMENT: u32 = 16;

pub enum GlStreamBufferResource<T: HasContext> {
    Texture(T::Texture),
    UniformBlock(T::Buffer),
}

pub struct GlStreamBuffer<T: HasContext> {
    buffer: BufferImpl<T>,
    size: u32,
    ptr: Cell<u32>,
}

enum BufferImpl<T: HasContext> {
    Uniform { buffer: T::Buffer },
    Texture { buffer: T::Buffer, texture: T::Texture },
}

impl<T: HasContext> GlStreamBuffer<T> {
    pub fn new_ubo(gl: &T, size: u32) -> Self {
        unsafe {
            let ubo_buffer = gl.create_buffer().unwrap();
            gl.bind_buffer(UNIFORM_BUFFER, Some(ubo_buffer));
            gl.buffer_data_size(UNIFORM_BUFFER, size as i32, DYNAMIC_DRAW);

            Self {
                buffer: BufferImpl::Uniform { buffer: ubo_buffer },
                size,
                ptr: Cell::new(0),
            }
        }
    }

    pub fn new_tbo(gl: &T, size: u32) -> Self {
        unsafe {
            let tbo_buffer = gl.create_buffer().unwrap();
            gl.bind_buffer(UNIFORM_BUFFER, Some(tbo_buffer));
            gl.buffer_data_size(UNIFORM_BUFFER, size as i32, DYNAMIC_DRAW);

            let tbo_texture = gl.create_texture().unwrap();
            gl.bind_texture(TEXTURE_BUFFER, Some(tbo_texture));
            gl.tex_buffer(TEXTURE_BUFFER, RGBA32UI, Some(tbo_buffer));

            Self {
                buffer: BufferImpl::Texture {
                    buffer: tbo_buffer,
                    texture: tbo_texture,
                },
                size,
                ptr: Cell::new(0),
            }
        }
    }

    pub fn bytes_left(&self) -> u32 {
        self.size - self.ptr.get()
    }

    pub fn write(&self, gl: &T, data: &[u8]) -> Range<u32> {
        debug_assert!(
            data.len() % BUFFER_ALIGNMENT as usize == 0,
            "data length must be aligned to {} bytes",
            BUFFER_ALIGNMENT
        );

        if data.len() > self.bytes_left() as usize {
            self.ptr.set(0);
            assert!(
                data.len() <= self.bytes_left() as usize,
                "not enough space in the ring buffer"
            );
        }

        let ptr = self.ptr.get();

        unsafe {
            match &self.buffer {
                BufferImpl::Uniform { buffer } => {
                    gl.bind_buffer(UNIFORM_BUFFER, Some(*buffer));
                    gl.buffer_sub_data_u8_slice(UNIFORM_BUFFER, ptr as i32, data);
                }
                BufferImpl::Texture { buffer, .. } => {
                    gl.bind_buffer(TEXTURE_BUFFER, Some(*buffer));
                    gl.buffer_sub_data_u8_slice(TEXTURE_BUFFER, ptr as i32, data);
                }
            };
        }

        self.ptr.set(ptr + data.len() as u32);

        ptr..self.ptr.get()
    }

    pub fn resource(&self) -> GlStreamBufferResource<T> {
        match &self.buffer {
            BufferImpl::Uniform { buffer } => GlStreamBufferResource::UniformBlock(*buffer),
            BufferImpl::Texture { texture, .. } => GlStreamBufferResource::Texture(*texture),
        }
    }

    pub fn delete(self, gl: &T) {
        unsafe {
            match self.buffer {
                BufferImpl::Uniform { buffer } => gl.delete_buffer(buffer),
                BufferImpl::Texture { buffer, texture } => {
                    gl.delete_buffer(buffer);
                    gl.delete_texture(texture);
                }
            }
        }
    }
}
