mod ir;

use super::{REGISTER_COUNT, VMOp, VMOpcode};
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

impl CompiledShader {
    pub fn compile(arena: &Bump, graph: &Graph) -> Self {
        let mut slots_input = 0;
        let mut slots_texture = 0;

        let builder = ir::IRBuilder::from_graph(arena, graph, |builder, addr, input| match input {
            OpInput::TextureRender | OpInput::TextureStatic => {
                builder.set_texture(addr, slots_texture);
                slots_texture += 1;
            }

            _ => {
                builder.set_graph(addr, 0, ir::IR::new(arena, VMOp::Read(slots_input, ())));
                slots_input += 1;
            }
        });

        let program = builder.extract_program(graph.output(), 4);
        let program = ir::optimize_peephole(&program, arena);
        let program = ir::optimize_hashcons(&program, arena);

        let (program_static, program_dynamic) = ir::split_static_dynamic(&program, arena);

        let program_static = ir::lower_to_opcodes(&program_static, arena);
        let program_dynamic = ir::lower_to_opcodes(&program_dynamic, arena);

        assert!(
            program_static.registers <= REGISTER_COUNT as u8 && program_dynamic.registers <= REGISTER_COUNT as u8,
            "too many registers used"
        );

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

    pub fn static_opcodes(&self) -> &[VMOpcode] {
        &self.static_opcodes
    }

    pub fn dynamic_opcodes(&self) -> &[VMOpcode] {
        &self.dynamic_opcodes
    }

    pub fn static_registers(&self) -> u8 {
        self.static_registers
    }

    pub fn dynamic_registers(&self) -> u8 {
        self.dynamic_registers
    }

    pub fn static_outputs(&self) -> &[u8] {
        &self.static_outputs
    }

    pub fn dynamic_outputs(&self) -> &[u8; 4] {
        &self.dynamic_outputs
    }

    pub fn input_slots(&self) -> u32 {
        self.slots_input
    }

    pub fn texture_slots(&self) -> u8 {
        self.slots_texture
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
        let graph = Graph::collect(|| {
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
