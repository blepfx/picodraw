use crate::{graph::ShaderGraph, types::GlType, Float2, Float4, Shader};
use encoding::InputStructure;
use rustc_hash::FxHashMap;

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
    shaders: FxHashMap<String, ShaderData>,
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
        let mut input = None;
        let graph = ShaderGraph::collect(|| {
            let (structure, vars) = InputStructure::of::<T>();
            input = Some(structure);
            T::draw(Float2::input_raw("@pos".to_string()), vars)
        });

        let id = T::id();
        if self.shaders.contains_key(id) {
            panic!("shader id duplicate '{}'", id);
        }

        self.dirty = true;
        self.shaders.insert(
            id.to_owned(),
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
                    .map(move |(name, generator)| (data.id, name.as_str(), generator()))
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
        value: &T,
        width: u32,
        height: u32,
    ) {
        let data = self
            .shaders
            .get(T::id())
            .expect("register the drawable first");

        encoder.push(value, data.id, &data.input, width, height);
    }
}