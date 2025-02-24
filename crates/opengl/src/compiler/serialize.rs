use picodraw_core::graph::*;

#[derive(Debug)]
pub struct ShaderDataLayout {
    pub fields: Vec<(u32, OpInput)>,
    pub textures: Vec<u32>,
    pub size: u32,
    pub branch_id: u32,
}

pub struct ShaderDataEncoder<'a> {
    layout: &'a ShaderDataLayout,
    data: &'a mut [u8],
    pointer: usize,
}

impl ShaderDataLayout {
    pub fn new(graph: &Graph, uid: u32, textures_start: u32, textures_limit: u32) -> Self {
        let mut bitmap = vec![];
        let mut fields = vec![];
        let mut textures = vec![];

        for (_, op) in graph.iter() {
            if let Op::Input(input) = op {
                let (size, align) = match input {
                    OpInput::F32 => (4, 4),
                    OpInput::I32 => (4, 4),
                    OpInput::I16 => (2, 2),
                    OpInput::I8 => (1, 1),
                    OpInput::U32 => (4, 4),
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
            fields,
            textures,
            size: bitmap.len() as u32,
        }
    }
}

impl<'a> ShaderDataEncoder<'a> {
    pub fn new(layout: &'a ShaderDataLayout, data: &'a mut [u8]) -> Self {
        Self {
            layout,
            data,
            pointer: 0,
        }
    }

    pub fn write_i32(&mut self, value: i32) {
        let (offset, input) = self.layout.fields.get(self.pointer).expect("malformed write stream");
        let offset = *offset as usize;

        match input {
            OpInput::I32 | OpInput::U32 => {
                self.data[offset..offset + 4].copy_from_slice(&value.to_ne_bytes());
            }
            OpInput::I16 | OpInput::U16 => {
                self.data[offset..offset + 2].copy_from_slice(&(value as i16).to_ne_bytes());
            }
            OpInput::I8 | OpInput::U8 => {
                self.data[offset..offset + 1].copy_from_slice(&(value as u8).to_ne_bytes());
            }
            _ => panic!("malformed write stream"),
        }

        self.pointer += 1;
    }

    pub fn write_f32(&mut self, value: f32) {
        let (offset, input) = self.layout.fields.get(self.pointer).expect("malformed write stream");
        let offset = *offset as usize;

        match input {
            OpInput::F32 => {
                self.data[offset..offset + 4].copy_from_slice(&value.to_ne_bytes());
            }
            _ => panic!("malformed write stream"),
        }

        self.pointer += 1;
    }

    pub fn finish(self) {
        if self.pointer != self.layout.fields.len() {
            panic!("malformed write stream");
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
    pub fn as_bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self as *const _ as *const _, std::mem::size_of::<Self>()) }
    }
}

pub struct ShaderTextureAllocator {
    data: Vec<Option<ShaderTextureSlot>>,
}

impl ShaderTextureAllocator {
    pub fn new(max_samplers: u32) -> Self {
        Self {
            data: vec![None; max_samplers as usize],
        }
    }

    pub fn clear(&mut self) {
        self.data.fill(None);
    }

    pub fn try_allocate(&mut self, id: u32, slot: ShaderTextureSlot) -> Result<bool, ()> {
        if self.data[id as usize] == None {
            self.data[id as usize] = Some(slot);
            Ok(true)
        } else if self.data[id as usize] == Some(slot) {
            Ok(false)
        } else {
            Err(())
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ShaderTextureSlot {
    Static(picodraw_core::Texture),
    Render(picodraw_core::RenderTexture),
}
