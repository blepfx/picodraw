use crate::{graph::ShaderGraph, types::GlType, Bounds, Float2, Float4, Shader, ShaderContext};
use encoding::{InputStructure, BUILTIN_BOUNDS, BUILTIN_POSITION, BUILTIN_RESOLUTION};
use rustc_hash::FxHashMap;
use std::any::{type_name, TypeId};

mod atlas;
mod encoding;
mod glsl;

pub use atlas::TextureAtlas;
pub use encoding::QuadEncoder;
pub use glsl::VERTEX_SHADER;

struct ShaderData {
    id: u32,
    graph: ShaderGraph<Float4>,
    input: InputStructure,
}

pub struct ShaderMap {
    shaders: FxHashMap<TypeId, ShaderData>,
    dirty: bool,
}

impl ShaderMap {
    pub fn new() -> Self {
        Self {
            shaders: FxHashMap::default(),
            dirty: false,
        }
    }

    pub fn register<T: Shader>(&mut self) {
        let id = T::id();
        if self.shaders.contains_key(&id) {
            return;
        }

        let mut input = None;
        let graph = ShaderGraph::collect(|| {
            let (structure, vars) = InputStructure::of::<T>();
            input = Some(structure);
            T::draw(ShaderContext {
                vars: &vars,
                position: Float2::input_raw(BUILTIN_POSITION),
                resolution: Float2::input_raw(BUILTIN_RESOLUTION),
                bounds: Float4::input_raw(BUILTIN_BOUNDS),
            })
        });

        self.dirty = true;
        self.shaders.insert(
            id,
            ShaderData {
                id: self.shaders.len() as u32,
                graph,
                input: input.unwrap(),
            },
        );
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn recompile(&mut self, max_texture_size: u32) -> (String, TextureAtlas) {
        self.dirty = false;

        let atlas = TextureAtlas::pack(
            self.shaders.values().flat_map(|data| {
                data.input
                    .textures
                    .iter()
                    .enumerate()
                    .map(move |(id, generator)| (data.id, id as u32, generator()))
            }),
            max_texture_size,
        );

        let fragment_src = glsl::generate_fragment_shader(
            self.shaders
                .values()
                .map(|data| (data.id, &data.graph, &data.input)),
            &atlas,
        );

        (fragment_src, atlas)
    }

    pub fn write<T: Shader>(
        &mut self,
        encoder: &mut QuadEncoder,
        bounds: Bounds,
        value: &T,
        width: u32,
        height: u32,
    ) {
        let data = self.shaders.get(&T::id()).unwrap_or_else(|| {
            if cfg!(debug_assertions) {
                panic!("register the drawable first ({})", type_name::<T>())
            } else {
                panic!("register the drawable first")
            }
        });

        encoder.push(
            value,
            data.id,
            bounds,
            &data.input,
            width as f32,
            height as f32,
        );
    }
}
