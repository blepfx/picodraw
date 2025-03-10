use crate::shader::interpreter::{PIXEL_COUNT, VMSlot};
use fxhash::FxHashMap;
use picodraw_core::{Bounds, Shader};
use std::ops::Range;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct TileCoord(u32, u32);

struct DispatchObject {
    bounds: Bounds,
    shader: u32,
    data: Range<u32>,
}

struct DispatchShader {}

struct DispatchOutput {
    tile: TileCoord,
    data: Box<[u8; PIXEL_COUNT * 4]>,
}

pub struct Dispatcher {
    objects: Vec<DispatchObject>,
    data: Vec<VMSlot>,
    tiles: FxHashMap<TileCoord, Vec<u32>>,
    shaders: Vec<DispatchShader>,
}

impl Dispatcher {
    pub fn new() -> Self {
        Dispatcher {
            objects: Vec::new(),
            data: Vec::new(),
            tiles: FxHashMap::default(),
            shaders: Vec::new(),
        }
    }

    pub fn begin_quad(&mut self, bounds: Bounds, shader: u32) {
        self.objects.push(DispatchObject {
            bounds,
            shader,
            data: self.data.len() as u32..self.data.len() as u32,
        });
    }

    pub fn add_data(&mut self, data: &[VMSlot]) {
        self.data.extend_from_slice(data);
    }

    pub fn end_quad(&mut self) {
        let obj = self
            .objects
            .last_mut()
            .expect("end_quad called without begin_quad");
        obj.data.end = self.data.len() as u32;
    }
}
