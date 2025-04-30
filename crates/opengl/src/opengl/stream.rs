use glow::{HasContext, STREAM_DRAW, UNIFORM_BUFFER};
use std::{cell::Cell, ops::Range};

pub const BUFFER_ALIGNMENT: u32 = 16;

pub struct GlStreamBuffer<T: HasContext> {
    pub(super) ubo_buffer: T::Buffer,

    size: u32,
    ptr: Cell<u32>,
}

impl<T: HasContext> GlStreamBuffer<T> {
    pub fn new(gl: &T, size: u32) -> Self {
        unsafe {
            let ubo_buffer = gl.create_buffer().unwrap();
            gl.bind_buffer(UNIFORM_BUFFER, Some(ubo_buffer));
            gl.buffer_data_size(UNIFORM_BUFFER, size as i32, STREAM_DRAW);

            Self {
                ubo_buffer,
                size,
                ptr: Cell::new(0),
            }
        }
    }

    pub fn bytes_left(&self) -> u32 {
        self.size - self.ptr.get()
    }

    pub fn write(&self, gl: &T, data: &[u8]) -> Range<u32> {
        if (self.bytes_left() as usize) < data.len() {
            //TODO: proper invalidation/orphaning

            self.ptr.set(0);
            assert!(
                data.len() <= self.bytes_left() as usize,
                "not enough space in the ring buffer"
            );
        }

        let ptr = self.ptr.get();

        unsafe {
            gl.bind_buffer(UNIFORM_BUFFER, Some(self.ubo_buffer));
            gl.buffer_sub_data_u8_slice(UNIFORM_BUFFER, ptr as i32, data);
        }

        self.ptr
            .set(ptr + (data.len() as u32).next_multiple_of(BUFFER_ALIGNMENT));

        ptr..self.ptr.get()
    }

    pub fn delete(self, gl: &T) {
        unsafe {
            gl.delete_buffer(self.ubo_buffer);
        }
    }
}
