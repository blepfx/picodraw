pub mod codegen;
pub mod serialize;

use std::collections::HashMap;

pub const UNIFORM_BUFFER_OBJECT: &str = "uBuffer";
pub const UNIFORM_TEXTURE_SAMPLERS: &str = "uTextures";
pub const UNIFORM_BUFFER_OFFSET_INSTANCE: &str = "uBufferOffsetInstance";
pub const UNIFORM_BUFFER_OFFSET_DATA: &str = "uBufferOffsetData";
pub const UNIFORM_FRAME_RESOLUTION: &str = "uResolution";
pub const UNIFORM_FRAME_SCREEN: &str = "uScreenTarget";

pub struct Compiler<'a> {
    options: CompilerOptions,
    shaders: HashMap<picodraw_core::Shader, &'a picodraw_core::Graph>,
}

pub struct CompilerOptions {
    pub glsl_version: u32,
    pub texture_units: u32,
}

pub struct CompilerResult {
    pub shader_vertex: String,
    pub shader_fragment: String,
    pub shader_layout: HashMap<picodraw_core::Shader, serialize::ShaderDataLayout>,
}

impl<'a> Compiler<'a> {
    pub fn new(options: CompilerOptions) -> Self {
        Self {
            options,
            shaders: HashMap::new(),
        }
    }

    pub fn put_shader(&mut self, id: picodraw_core::Shader, graph: &'a picodraw_core::Graph) {
        self.shaders.insert(id, graph);
    }

    pub fn compile(self) -> CompilerResult {
        let shader_layout = {
            let mut layouts = HashMap::new();
            let mut textures = 0;
            let mut branch_ids = 1;

            for (id, graph) in self.shaders.iter() {
                let layout = serialize::ShaderDataLayout::new(
                    graph,
                    branch_ids,
                    textures,
                    self.options.texture_units - 1,
                );

                branch_ids += 1;
                textures += layout.textures.len() as u32;
                layouts.insert(*id, layout);
            }

            layouts
        };

        let codegen_options = codegen::CodegenOptions {
            glsl_version: self.options.glsl_version,
            texture_samplers: self.options.texture_units - 1,
        };

        let shader_vertex = codegen::generate_vertex_shader(&codegen_options);
        let shader_fragment = {
            let mut codegen = codegen::FragmentCodegen::new(&codegen_options);

            for (id, graph) in self.shaders.iter() {
                let layout = shader_layout.get(id).unwrap();

                codegen.emit_begin_graph(&layout);
                for op in graph.iter() {
                    codegen.emit_atom(graph, op);
                }
                codegen.emit_end_graph(graph);
            }

            codegen.finish()
        };

        CompilerResult {
            shader_vertex,
            shader_fragment,
            shader_layout,
        }
    }
}
