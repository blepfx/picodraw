use crate::opengl::BUFFER_ALIGNMENT;
use picodraw_core::graph::*;

#[derive(Debug)]
pub struct ShaderDataLayout {
    pub inputs: Vec<(u32, OpInput)>,
    pub textures: Vec<u32>,
    pub size: u32,
    pub branch_id: u32,
}

impl ShaderDataLayout {
    pub fn new(graph: &Graph, uid: u32, textures_start: u32, textures_limit: u32) -> Self {
        let mut bitmap = vec![];
        let mut fields = vec![];
        let mut textures = vec![];

        for op in graph.iter() {
            if let OpValue::Input(input) = graph.value_of(op) {
                let (size, align) = match input {
                    OpInput::F32 => (4, 4),
                    OpInput::I32 => (4, 4),
                    OpInput::I16 => (2, 2),
                    OpInput::I8 => (1, 1),
                    OpInput::U16 => (2, 2),
                    OpInput::U8 => (1, 1),

                    OpInput::TextureRender | OpInput::TextureStatic => {
                        textures.push((textures_start + textures.len() as u32) % textures_limit);
                        continue;
                    }
                };

                let offset = take_offset(&mut bitmap, size, align);
                fields.push((offset, input));
            }
        }

        Self {
            branch_id: uid,
            inputs: fields,
            textures,
            size: (bitmap.len() as u32).next_multiple_of(BUFFER_ALIGNMENT),
        }
    }
}

fn take_offset(bitmap: &mut Vec<bool>, size: u32, align: u32) -> u32 {
    if size == 0 {
        return 0;
    }

    let mut offset = 0;
    loop {
        if (offset..offset + size).all(|i| !bitmap.get(i as usize).copied().unwrap_or_default()) {
            break;
        }

        offset += align;
    }

    for i in offset..offset + size {
        while i as usize >= bitmap.len() {
            bitmap.push(false);
        }

        bitmap[i as usize] = true;
    }

    offset
}

pub fn encode(dst: &mut [u8], layout: &ShaderDataLayout, stream: impl IntoIterator<Item = u32>) -> Result<(), ()> {
    let mut pointer = 0;
    for value in stream {
        let (offset, input) = layout.inputs.get(pointer).copied().ok_or(())?;
        let offset = offset as usize;

        match input {
            OpInput::I32 | OpInput::F32 => {
                dst[offset..offset + 4].copy_from_slice(&(value as u32).to_ne_bytes());
            }
            OpInput::I16 | OpInput::U16 => {
                dst[offset..offset + 2].copy_from_slice(&(value as u16).to_ne_bytes());
            }
            OpInput::I8 | OpInput::U8 => {
                dst[offset..offset + 1].copy_from_slice(&(value as u8).to_ne_bytes());
            }
            _ => return Err(()),
        }

        pointer += 1;
    }

    if pointer != layout.inputs.len() {
        return Err(());
    }

    Ok(())
}

#[repr(C)]
#[derive(Debug)]
pub struct QuadDescriptorStruct {
    pub left: u16,
    pub top: u16,
    pub right: u16,
    pub bottom: u16,
    pub shader: u32,
    pub offset: u32,
}

impl QuadDescriptorStruct {
    pub const SIZE: usize = std::mem::size_of::<Self>();

    pub fn as_byte_slice(slice: &[Self]) -> &[u8] {
        let len = Self::SIZE * slice.len();
        let ptr = slice.as_ptr() as *const u8;
        unsafe { std::slice::from_raw_parts(ptr, len) }
    }
}
