use super::{IRProgram, IRVisit};
use crate::vm::CompiledProgram;
use bumpalo::{Bump, collections::Vec};
use picodraw_core::ShaderError;
use std::collections::HashMap;

/// register allocation and lowering to executable vm ops
pub fn lower_to_opcodes<'a>(program: &IRProgram<'a>, arena: &'a Bump) -> Result<CompiledProgram<'a>, ShaderError> {
    // collect ops in dfs post order and collected output edge counts
    let mut ops = Vec::new_in(arena);
    let mut edges = HashMap::new();

    // TODO: order children traversal by some heuristic to reduce register pressure
    program.visit_dfs(arena, |visit| match visit {
        IRVisit::Enter(ir, _) => {
            let edges = edges.entry(ir).or_insert(0);
            *edges += 1;
            *edges == 1
        }
        IRVisit::Exit(ir, _) => {
            ops.push(ir);
            true
        }
    });

    // allocate registers for each op
    let mut registers = HashMap::new();
    let mut stack = Vec::from_iter_in((0..255u8).rev(), arena);

    for op in ops.iter().copied() {
        let register = match stack.pop() {
            Some(register) => register,
            None => return Err(ShaderError::TooComplex),
        };

        registers.insert(op, register);
        op.visit_children(|input| {
            let register = registers[&input];
            let edges = edges.entry(input).or_default();
            *edges -= 1;
            if *edges == 0 {
                stack.push(register);
            }
        });
    }

    // map ops to vm opcodes
    let mut opcodes = Vec::new_in(arena);
    let mut outputs = Vec::new_in(arena);
    let mut max_register = 0u8;

    for op in ops.iter().copied() {
        max_register = max_register.max(registers[&op]);
        opcodes.push(
            op.0.map_inputs(|input| registers[&input])
                .map_outputs(|_| registers[&op]),
        );
    }

    for output in program.outputs {
        outputs.push(registers[output]);
    }

    Ok(CompiledProgram {
        opcodes: opcodes.into_bump_slice(),
        outputs: outputs.into_bump_slice(),
        registers: max_register + 1,
    })
}
