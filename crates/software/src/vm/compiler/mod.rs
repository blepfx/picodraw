mod ir;

use super::VMOpcode;
use bumpalo::Bump;
use picodraw_core::{ShaderData, ShaderError};
use std::fmt::Debug;

pub struct CompiledShader {
    static_opcodes: Vec<VMOpcode>,
    static_outputs: Vec<u8>,
    static_registers: u8,

    dynamic_opcodes: Vec<VMOpcode>,
    dynamic_outputs: [u8; 4],
    dynamic_registers: u8,
}

#[derive(Clone, Copy)]
pub struct CompiledProgram<'a> {
    opcodes: &'a [VMOpcode],
    outputs: &'a [u8],
    registers: u8,
}

impl CompiledShader {
    pub fn compile(arena: &Bump, data: &ShaderData) -> Result<Self, ShaderError> {
        let program = ir::lower_to_ir(arena, data)?;
        let program = ir::optimize_peephole(&program, arena, ir::peeper_split);
        let program = ir::optimize_peephole(&program, arena, ir::peeper_const);
        let program = ir::optimize_hashcons(&program, arena);

        let (program_static, program_dynamic) = ir::split_static_dynamic(&program, arena);
        let program_static = ir::optimize_peephole(&program_static, arena, ir::peeper_join);
        let program_dynamic = ir::optimize_peephole(&program_dynamic, arena, ir::peeper_join);

        let program_static = ir::lower_to_opcodes(&program_static, arena)?;
        let program_dynamic = ir::lower_to_opcodes(&program_dynamic, arena)?;

        Ok(Self {
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
        })
    }

    pub fn static_program(&self) -> CompiledProgram<'_> {
        unsafe { CompiledProgram::new_unchecked(&self.static_opcodes, &self.static_outputs, self.static_registers) }
    }

    pub fn dynamic_program(&self) -> CompiledProgram<'_> {
        unsafe { CompiledProgram::new_unchecked(&self.dynamic_opcodes, &self.dynamic_outputs, self.dynamic_registers) }
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

    pub fn used_registers(&self) -> usize {
        self.registers as usize
    }
}

impl<'a> Debug for CompiledProgram<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[")?;

        for op in self.opcodes {
            writeln!(f, "   {:?}", op)?;
        }

        for output in self.outputs {
            writeln!(f, "   Output({})", output)?;
        }

        write!(f, "]")
    }
}

#[cfg(test)]
mod tests {
    use picodraw_core::{
        ShaderData,
        trace::{float1, float2, float4},
    };

    fn shader_rect() -> float4 {
        let center = float2((float1::read_f32(0), float1::read_f32(4)));
        let angle = float1::read_f32(8);
        let extents = float2((float1::read_f32(12), float1::read_f32(16)));
        let radius = float1::read_f32(20);
        let color = float4((
            float1::read_f32(24),
            float1::read_f32(28),
            float1::read_f32(32),
            float1::read_f32(36),
        ));

        let p = float2::position() - center;
        let p = float2((
            p.x() * angle.cos() - p.y() * angle.sin(),
            p.x() * angle.sin() + p.y() * angle.cos(),
        ));

        let q = p.abs() - extents + radius;
        let d = q.x().max(q.y()).min(0.0) + q.max(0.0).len() - radius;

        let mask = (0.5 - d * 0.707).clamp(0.0, 1.0);
        float4((color.x(), color.y(), color.z(), color.w() * mask))
    }

    #[test]
    fn test() {
        let graph = ShaderData::trace(shader_rect);

        let arena = bumpalo::Bump::new();
        let compiled = super::CompiledShader::compile(&arena, &graph).unwrap();

        dbg!(&compiled.static_program());
        dbg!(&compiled.dynamic_program());
    }
}
