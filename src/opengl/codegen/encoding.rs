use crate::{
    types::GlType, Float, Float2, Int, Shader, ShaderData, ShaderDataWriter, ShaderVars, Texture,
};
use rustc_hash::FxHashMap;
use std::{ops::Range, sync::Arc};

pub struct InputStructure {
    pub inputs: FxHashMap<String, InputField>,
    pub textures: FxHashMap<String, Arc<dyn Fn() -> image::DynamicImage>>,
    pub size: u32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum InputRepr {
    UInt8,
    UInt16,
    UInt32,
    Int8,
    Int16,
    Int32,
    Float32,
}

pub struct InputField {
    pub offset: u32,
    pub repr: InputRepr,
}

struct InputCollector {
    inputs: FxHashMap<String, InputField>,
    textures: FxHashMap<String, Arc<dyn Fn() -> image::DynamicImage>>,
    bitmap: Vec<bool>,
}

impl InputStructure {
    pub fn of<T: ShaderData>() -> (Self, T::ShaderVars) {
        let mut collector = InputCollector::new();
        let vars = T::shader_vars(&mut collector);
        (collector.finish(), vars)
    }
}

impl InputCollector {
    fn new() -> Self {
        Self {
            inputs: FxHashMap::default(),
            textures: FxHashMap::default(),
            bitmap: vec![],
        }
    }

    fn finish(self) -> InputStructure {
        InputStructure {
            inputs: self.inputs,
            textures: self.textures,
            size: self.bitmap.len() as u32,
        }
    }

    fn register(&mut self, id: &str, repr: InputRepr) {
        if self.inputs.contains_key(id) {
            return;
        }

        let (size, align) = match repr {
            InputRepr::Int8 => (1, 1),
            InputRepr::Int16 => (2, 2),
            InputRepr::Int32 => (4, 4),
            InputRepr::UInt8 => (1, 1),
            InputRepr::UInt16 => (2, 2),
            InputRepr::UInt32 => (4, 4),
            InputRepr::Float32 => (4, 4),
        };

        self.inputs.insert(
            id.to_owned(),
            InputField {
                offset: take_offset(&mut self.bitmap, size, align),
                repr: repr.clone(),
            },
        );
    }
}

impl ShaderVars for InputCollector {
    fn int8(&mut self, id: &str) -> Int {
        self.register(id, InputRepr::Int8);
        Int::input_raw(id.to_owned())
    }

    fn int16(&mut self, id: &str) -> Int {
        self.register(id, InputRepr::Int16);
        Int::input_raw(id.to_owned())
    }

    fn int32(&mut self, id: &str) -> Int {
        self.register(id, InputRepr::Int32);
        Int::input_raw(id.to_owned())
    }

    fn uint8(&mut self, id: &str) -> Int {
        self.register(id, InputRepr::UInt8);
        Int::input_raw(id.to_owned())
    }

    fn uint16(&mut self, id: &str) -> Int {
        self.register(id, InputRepr::UInt16);
        Int::input_raw(id.to_owned())
    }

    fn uint32(&mut self, id: &str) -> Int {
        self.register(id, InputRepr::UInt32);
        Int::input_raw(id.to_owned())
    }

    fn float(&mut self, id: &str) -> Float {
        self.register(id, InputRepr::Float32);
        Float::input_raw(id.to_owned())
    }

    fn texture(&mut self, tex: Arc<dyn Fn() -> image::DynamicImage>) -> Texture {
        let id = format!("@tex/{}", self.textures.len());
        self.textures.insert(id.clone(), tex);
        Texture::input_raw(id)
    }

    fn position(&mut self) -> Float2 {
        Float2::input_raw("@pos".to_owned())
    }

    fn resolution(&mut self) -> Float2 {
        Float2::input_raw("@res".to_owned())
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

pub struct InputEncoder<'a> {
    data: &'a mut [[u32; 4]],
    structure: &'a InputStructure,
}

impl<'a> ShaderDataWriter for InputEncoder<'a> {
    fn write_int(&mut self, location: &str, x: i32) {
        let field = match self.structure.inputs.get(location) {
            Some(field) => field,
            None => panic!("unknown field '{}'", location),
        };

        match field.repr {
            InputRepr::Int8 => {
                let bytes = bytemuck::cast_slice_mut::<_, u8>(self.data);
                bytes[field.offset as usize] = x as i8 as u8;
            }
            InputRepr::Int16 => {
                let shorts = bytemuck::cast_slice_mut::<_, u16>(self.data);
                shorts[(field.offset / 2) as usize] = x as i16 as u16;
            }
            InputRepr::Int32 => {
                let ints = bytemuck::cast_slice_mut::<_, u32>(self.data);
                ints[(field.offset / 4) as usize] = x as u32;
            }
            _ => panic!("wrong type for '{}'", location),
        }
    }

    fn write_float(&mut self, location: &str, x: f32) {
        let field = match self.structure.inputs.get(location) {
            Some(field) => field,
            None => panic!("unknown field '{}'", location),
        };

        match field.repr {
            InputRepr::Float32 => {
                let ints = bytemuck::cast_slice_mut::<_, u32>(self.data);
                ints[(field.offset / 4) as usize] = f32::to_bits(x);
            }

            _ => panic!("wrong type for '{}'", location),
        }
    }
}

pub struct QuadEncoder {
    pub quads: Vec<QuadEncoded>,
    pub data: Vec<[u32; 4]>,
}

pub struct QuadEncoded {
    pub bounds: [u16; 4],
    pub shader_id: u32,
    pub data_range: Range<usize>,
}

impl QuadEncoder {
    pub fn new() -> Self {
        Self {
            quads: vec![],
            data: vec![],
        }
    }

    pub fn clear(&mut self) {
        self.quads.clear();
        self.data.clear();
    }

    pub fn push<T: Shader>(
        &mut self,
        draw: &T,
        shader_id: u32,
        input: &InputStructure,
        width: u32,
        height: u32,
    ) {
        let data_start = self.data.len();
        self.data
            .resize(self.data.len() + input.size.div_ceil(16) as usize, [0; 4]);

        draw.write(&mut InputEncoder {
            data: &mut self.data[data_start..],
            structure: input,
        });

        let bounds = draw.bounds();

        self.quads.push(QuadEncoded {
            bounds: [
                (bounds[0] / width as f32 * 65535.0).floor() as u16,
                (bounds[1] / height as f32 * 65535.0).floor() as u16,
                (bounds[2] / width as f32 * 65535.0).ceil() as u16,
                (bounds[3] / height as f32 * 65535.0).ceil() as u16,
            ],
            shader_id,
            data_range: data_start..self.data.len(),
        })
    }
}
