pub mod codegen;
pub mod serialize;

use std::collections::HashMap;

pub const UNIFORM_TEXTURE_SAMPLERS: &str = "uTextures";
pub const UNIFORM_FRAME_RESOLUTION: &str = "uResolution";
pub const UNIFORM_FRAME_SCREEN: &str = "uScreenTarget";

pub const UNIFORM_BUFFER_UNIFORM_F32: &str = "uBufferF32";
pub const UNIFORM_BUFFER_UNIFORM_U32: &str = "uBufferU32";
pub const UNIFORM_BUFFER_TEXTURE: &str = "uBuffer";

pub const UNIFORM_BUFFER_LIST_OFFSET: &str = "uBufferListOffset";
pub const UNIFORM_BUFFER_DATA_OFFSET: &str = "uBufferDataOffset";

pub fn compile_glsl<'a>(
    options: CompilerOptions,
    shaders: impl IntoIterator<Item = (picodraw_core::Shader, &'a picodraw_core::Graph)>,
) -> CompilerResult {
    let shaders = shaders.into_iter().collect::<HashMap<_, _>>();

    let shader_layout = {
        let mut layouts = HashMap::new();
        let mut textures = 0;
        let mut branch_ids = 1;

        for (id, graph) in shaders.iter() {
            let layout = serialize::ShaderDataLayout::new(graph, branch_ids, textures, options.texture_units);

            branch_ids += 1;
            textures += layout.textures.len() as u32;
            layouts.insert(*id, layout);
        }

        layouts
    };

    let shader_vertex = codegen::generate_vertex_shader(&options);
    let shader_fragment = {
        let mut codegen = codegen::FragmentCodegen::new(&options);

        for (id, graph) in shaders.iter() {
            let layout = shader_layout.get(id).unwrap();

            codegen.emit_graph_begin(layout.branch_id);

            for texture in layout.textures.iter() {
                codegen.emit_graph_texture(*texture);
            }

            for (index, _) in layout.inputs.iter() {
                codegen.emit_graph_input(*index);
            }

            for op in graph.iter() {
                codegen.emit_atom(graph, op);
            }
            codegen.emit_graph_end(graph);
        }

        codegen.finish()
    };

    CompilerResult {
        vertex: shader_vertex,
        fragment: shader_fragment,
        layout: shader_layout,
    }
}

#[derive(Clone, Copy)]
pub enum CompilerBufferMode {
    UniformBlock { size_bytes: u32 },
    TextureBuffer,
}

#[derive(Clone, Copy)]
pub struct CompilerOptions {
    pub glsl_version: u32,
    pub texture_units: u32,
    pub buffer_mode: CompilerBufferMode,
}

pub struct CompilerResult {
    pub vertex: String,
    pub fragment: String,
    pub layout: HashMap<picodraw_core::Shader, serialize::ShaderDataLayout>,
}
