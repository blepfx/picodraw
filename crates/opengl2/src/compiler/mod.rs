mod analysis;
mod codegen;

use picodraw_core2::ShaderData;
use std::collections::HashMap;

pub const UNIFORM_TEXTURE_SAMPLERS: &str = "uTextures";
pub const UNIFORM_FRAME_RESOLUTION: &str = "uResolution";
pub const UNIFORM_FRAME_SCREEN: &str = "uScreenTarget";

pub const UNIFORM_BUFFER_UNIFORM_F32: &str = "uBufferF32";
pub const UNIFORM_BUFFER_UNIFORM_U32: &str = "uBufferU32";
pub const UNIFORM_BUFFER_TEXTURE: &str = "uBuffer";

pub const UNIFORM_BUFFER_LIST_OFFSET: &str = "uBufferListOffset";
pub const UNIFORM_BUFFER_DATA_OFFSET: &str = "uBufferDataOffset";

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
}

pub struct CompilerShader {
    pub index: u32,
    pub texture_slots: Box<[u32]>,
}

#[derive(Debug)]
pub enum CompilerError {
    TooManyTextures,
    InvalidInstructionType { index: u32 },
    InvalidInstructionReference { index: u32 },
    InvalidOutput { index: u32 },
}

pub struct GlslCompiler {
    options: CompilerOptions,
    shaders: HashMap<u32, Box<str>>,

    dispatch_alloc: u32,
    texture_alloc: u32,
}

impl GlslCompiler {
    pub fn new(options: CompilerOptions) -> Self {
        Self {
            options,
            shaders: HashMap::new(),
            dispatch_alloc: 0,
            texture_alloc: u32::MAX,
        }
    }

    pub fn options(&self) -> &CompilerOptions {
        &self.options
    }

    pub fn add_shader(&mut self, shader: ShaderData) -> Result<CompilerShader, CompilerError> {
        let structure = analysis::shader_analysis(shader)?;
        if structure.num_textures > self.options.texture_units {
            return Err(CompilerError::TooManyTextures);
        }

        let shader = CompilerShader {
            index: self.dispatch_alloc,
            texture_slots: (0..structure.num_textures)
                .map(|_| {
                    self.texture_alloc = self.texture_alloc.wrapping_add(1) % self.options.texture_units;
                    self.texture_alloc
                })
                .collect(),
        };

        let mut buffer = String::new();
        codegen::emit_shader_function(&mut buffer, &shader, &structure);

        self.shaders.insert(self.dispatch_alloc, buffer.into());
        self.dispatch_alloc = self.dispatch_alloc.wrapping_add(1);

        Ok(shader)
    }

    pub fn remove_shader(&mut self, index: u32) {
        self.shaders.remove(&index);
    }

    pub fn compile(&mut self) -> CompilerResult {
        let mut vertex = String::new();
        let mut fragment = String::new();

        codegen::emit_vertex_program(&mut vertex, &self.options);
        codegen::emit_fragment_header(&mut fragment, &self.options);

        for (_, chunk) in self.shaders.iter() {
            fragment.push_str(&chunk);
        }

        codegen::emit_fragment_dispatch(&mut fragment, self.shaders.iter().map(|(id, _)| *id));

        CompilerResult { vertex, fragment }
    }
}

#[cfg(test)]
mod test {
    use picodraw_core2::{
        ShaderData,
        trace::{TraceGraph, float2, float4},
    };

    use crate::compiler::{CompilerOptions, GlslCompiler};

    #[test]
    fn test() {
        let graph = TraceGraph::new(|| {
            let a = float2::position().x().sin();
            let b = float2::position().y().cos();
            let d = float2::position().y().cos();
            let c = a.gt(0.4).select(a - b + a.sin() + d + d, a + b);
            float4((a, b, c, c))
        });

        let mut compiler = GlslCompiler::new(CompilerOptions {
            glsl_version: 330,
            texture_units: 16,
            buffer_mode: super::CompilerBufferMode::TextureBuffer,
        });

        compiler.add_shader(ShaderData::from(&graph)).unwrap();

        println!("{}", compiler.compile().fragment);
    }
}
