use crate::{
    types::GlType, Bounds, Float, Float2, Int, Shader, ShaderData, ShaderDataWriter, ShaderVars,
    Texture,
};
use std::{ops::Range, sync::Arc};

pub const BUILTIN_POSITION: usize = usize::MAX;
pub const BUILTIN_RESOLUTION: usize = usize::MAX - 1;
pub const BUILTIN_BOUNDS: usize = usize::MAX - 2;

pub struct InputStructure {
    pub inputs: Vec<InputField>,
    pub textures: Vec<Arc<dyn Fn() -> image::DynamicImage>>,
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
    inputs: Vec<InputField>,
    textures: Vec<Arc<dyn Fn() -> image::DynamicImage>>,
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
            inputs: vec![],
            textures: vec![],
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

    fn register(&mut self, repr: InputRepr) -> usize {
        let id = self.inputs.len();
        let (size, align) = match repr {
            InputRepr::Int8 => (1, 1),
            InputRepr::Int16 => (2, 2),
            InputRepr::Int32 => (4, 4),
            InputRepr::UInt8 => (1, 1),
            InputRepr::UInt16 => (2, 2),
            InputRepr::UInt32 => (4, 4),
            InputRepr::Float32 => (4, 4),
        };

        self.inputs.push(InputField {
            offset: take_offset(&mut self.bitmap, size, align),
            repr: repr.clone(),
        });

        id
    }
}

impl ShaderVars for InputCollector {
    fn read_int8(&mut self) -> Int {
        Int::input_raw(self.register(InputRepr::Int8))
    }

    fn read_int16(&mut self) -> Int {
        Int::input_raw(self.register(InputRepr::Int16))
    }

    fn read_int32(&mut self) -> Int {
        Int::input_raw(self.register(InputRepr::Int32))
    }

    fn read_uint8(&mut self) -> Int {
        Int::input_raw(self.register(InputRepr::UInt8))
    }

    fn read_uint16(&mut self) -> Int {
        Int::input_raw(self.register(InputRepr::UInt16))
    }

    fn read_uint32(&mut self) -> Int {
        Int::input_raw(self.register(InputRepr::UInt32))
    }

    fn read_float(&mut self) -> Float {
        Float::input_raw(self.register(InputRepr::Float32))
    }

    fn texture(&mut self, tex: Arc<dyn Fn() -> image::DynamicImage>) -> Texture {
        let id = self.textures.len();
        self.textures.push(tex);
        Texture::input_raw(id)
    }

    fn resolution(&mut self) -> Float2 {
        Float2::input_raw(BUILTIN_RESOLUTION)
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
    resolution: (f32, f32),
    data: &'a mut [[u32; 4]],
    structure: &'a InputStructure,
    pointer: usize,
}

impl<'a> ShaderDataWriter for InputEncoder<'a> {
    fn write_int(&mut self, x: i32) {
        let field = match self.structure.inputs.get(self.pointer) {
            Some(field) => field,
            None => panic!("invalid shader data structure: writes more than reads"),
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
            InputRepr::UInt8 => {
                let bytes = bytemuck::cast_slice_mut::<_, u8>(self.data);
                bytes[field.offset as usize] = x as u8;
            }
            InputRepr::UInt16 => {
                let shorts = bytemuck::cast_slice_mut::<_, u16>(self.data);
                shorts[(field.offset / 2) as usize] = x as u16;
            }
            InputRepr::UInt32 => {
                let ints = bytemuck::cast_slice_mut::<_, u32>(self.data);
                ints[(field.offset / 4) as usize] = x as u32;
            }
            _ => panic!("invalid shader data structure: write/read type mismatch"),
        }

        self.pointer += 1;
    }

    fn write_float(&mut self, x: f32) {
        let field = match self.structure.inputs.get(self.pointer) {
            Some(field) => field,
            None => panic!("invalid shader data structure: writes more than reads"),
        };

        match field.repr {
            InputRepr::Float32 => {
                let ints = bytemuck::cast_slice_mut::<_, u32>(self.data);
                ints[(field.offset / 4) as usize] = f32::to_bits(x);
            }

            _ => panic!("invalid shader data structure: write/read type mismatch"),
        }

        self.pointer += 1;
    }

    fn resolution(&self) -> (f32, f32) {
        self.resolution
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
        bounds: Bounds,
        input: &InputStructure,
        width: f32,
        height: f32,
    ) {
        let bounds = [
            bounds.left.min(width.ceil() as u16),
            bounds.top.min(height.ceil() as u16),
            bounds.right.min(width.ceil() as u16),
            bounds.bottom.min(height.ceil() as u16),
        ];

        if bounds[0] != bounds[2] && bounds[1] != bounds[3] {
            let data_start = self.data.len();
            self.data
                .resize(self.data.len() + input.size.div_ceil(16) as usize, [0; 4]);

            draw.write(&mut InputEncoder {
                data: &mut self.data[data_start..],
                structure: input,
                resolution: (width, height),
                pointer: 0,
            });

            self.quads.push(QuadEncoded {
                bounds,
                shader_id,
                data_range: data_start..self.data.len(),
            });
        }
    }

    pub fn size_texels(&self) -> usize {
        self.quads.len() + self.data.len()
    }

    pub fn total_area(&self) -> u64 {
        self.quads
            .iter()
            .map(|quad| {
                let quad_width = quad.bounds[2].abs_diff(quad.bounds[0]) as u64;
                let quad_height = quad.bounds[3].abs_diff(quad.bounds[1]) as u64;
                quad_width * quad_height
            })
            .sum()
    }
}
