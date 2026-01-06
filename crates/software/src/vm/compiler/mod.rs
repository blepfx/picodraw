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

    #[allow(unused)]
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
        trace::{float1, float4},
    };

    #[test]
    fn test() {
        let graph = ShaderData::trace(|| {
            let p = float1::read_f32(0);

            let z0 = p.sin();
            let z1 = p.cos();
            let z2 = p.tan();
            let z3 = p.asin();
            let z4 = p.acos();

            let z0 = z0.abs();
            let z1 = z1.abs();
            let z2 = z2.abs();
            let z3 = z3.abs();
            let z4 = z4.abs();

            let u = ((z4 + z3) + z2) + z1 + z0;

            float4(u)
        });

        let arena = bumpalo::Bump::new();
        let compiled = super::CompiledShader::compile(&arena, &graph).unwrap();

        dbg!(&compiled.static_program());
        dbg!(&compiled.dynamic_program());
    }
}
