mod ir;

use super::{VMOp, VMOpcode};
use bumpalo::Bump;
use picodraw_core::{Graph, graph::OpInput};

#[derive(Debug)]
pub struct CompiledShader {
    slots_input: u32,
    slots_texture: u8,

    static_opcodes: Vec<VMOpcode>,
    static_outputs: Vec<u8>,
    static_registers: u8,

    dynamic_opcodes: Vec<VMOpcode>,
    dynamic_outputs: [u8; 4],
    dynamic_registers: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct CompiledProgram<'a> {
    opcodes: &'a [VMOpcode],
    outputs: &'a [u8],
    registers: u8,
}

impl CompiledShader {
    pub fn compile(arena: &Bump, graph: &Graph) -> Self {
        let mut slots_input = 0;
        let mut slots_texture = 0;

        let builder = ir::IRBuilder::from_graph(arena, graph, |builder, addr, input| match input {
            OpInput::Texture => {
                builder.set_texture(addr, slots_texture);
                slots_texture += 1;
            }

            _ => {
                builder.set_graph(addr, 0, ir::IR::new(arena, VMOp::Read(slots_input, ())));
                slots_input += 1;
            }
        });

        let program = builder.extract_program(graph.output(), 4);
        let program = ir::optimize_peephole(&program, arena, ir::peeper_generic);
        let program = ir::optimize_hashcons(&program, arena);

        let (program_static, program_dynamic) = ir::split_static_dynamic(&program, arena);

        let program_static = ir::lower_to_opcodes(&program_static, arena);
        let program_dynamic = ir::lower_to_opcodes(&program_dynamic, arena);

        Self {
            slots_input,
            slots_texture,

            static_opcodes: program_static.opcodes.to_vec(),
            static_registers: program_static.registers,
            static_outputs: program_static.outputs.to_vec(),

            dynamic_opcodes: program_dynamic.opcodes.to_vec(),
            dynamic_registers: program_dynamic.registers,
            dynamic_outputs: [
                program_dynamic.outputs[0],
                program_dynamic.outputs[1],
                program_dynamic.outputs[2],
                program_dynamic.outputs[3],
            ],
        }
    }

    pub fn static_program(&self) -> CompiledProgram<'_> {
        unsafe { CompiledProgram::new_unchecked(&self.static_opcodes, &self.static_outputs, self.static_registers) }
    }

    pub fn dynamic_program(&self) -> CompiledProgram<'_> {
        unsafe { CompiledProgram::new_unchecked(&self.dynamic_opcodes, &self.dynamic_outputs, self.dynamic_registers) }
    }

    pub fn input_slots(&self) -> usize {
        self.slots_input as usize
    }

    pub fn texture_slots(&self) -> usize {
        self.slots_texture as usize
    }
}

impl<'a> CompiledProgram<'a> {
    pub unsafe fn new_unchecked(opcodes: &'a [VMOpcode], outputs: &'a [u8], registers: u8) -> Self {
        Self {
            opcodes,
            outputs,
            registers,
        }
    }

    pub fn opcodes(&self) -> &'a [VMOpcode] {
        self.opcodes
    }

    pub fn output_registers(&self) -> &'a [u8] {
        self.outputs
    }

    #[allow(unused)]
    pub fn used_registers(&self) -> usize {
        self.registers as usize
    }
}

#[cfg(test)]
mod tests {
    use crate::vm::CompiledShader;
    use bumpalo::Bump;
    use picodraw_core::{
        Graph,
        shader::{float2, float4, io},
    };

    #[test]
    fn test() {
        let graph = Graph::trace(|| {
            let z = io::read::<f32>();

            let y = io::resolution().x() * z;
            let x = io::resolution().x() * z;

            let p = io::position() / io::resolution();
            let d = p - float2((0.5, 0.5));
            let d = d.len();

            float4((d, d + (y * 2.0 + x), d * z, 1.0))
        });
        let arena = Bump::new();
        let shader = CompiledShader::compile(&arena, &graph);

        dbg!(shader);
    }
}
