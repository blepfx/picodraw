use super::{IRProgram, VMProgram};
use bumpalo::{Bump, collections::Vec};
use std::collections::HashMap;

/// register allocation and lowering to executable vm ops
pub fn lower_to_opcodes<'a>(program: &IRProgram<'a>, arena: &'a Bump) -> VMProgram<'a> {
    // collect ops in dfs post order and collected output edge counts
    let mut ops = Vec::new_in(arena);
    let mut edges = HashMap::new();

    program.visit_ops(
        arena,
        (),
        |_, ir, _| {
            let edges = edges.entry(ir).or_insert(0);
            *edges += 1;
            *edges == 1
        },
        |_, ir, _| {
            ops.push(ir);
        },
    );

    // allocate registers for each op
    let mut registers = HashMap::new();
    let mut state = Vec::new_in(arena);

    for op in ops.iter().copied() {
        let output_register = match state.iter().position(|x| !x) {
            Some(register) => {
                state[register] = true;
                register as u8
            }
            None => {
                state.push(true);
                (state.len() - 1) as u8
            }
        };

        registers.insert(op, output_register);

        op.visit_children(|input| {
            let register = registers[&input];
            let edges = edges.entry(input).or_default();
            *edges -= 1;
            if *edges == 0 {
                state[register as usize] = false;
            }
        });
    }

    // map ops to vm opcodes
    let mut opcodes = Vec::new_in(arena);
    let mut outputs = Vec::new_in(arena);

    for op in ops.iter().copied() {
        opcodes.push(
            op.0.map_inputs(|input| registers[&input])
                .map_outputs(|_| registers[&op]),
        );
    }

    for output in program.outputs.iter().copied() {
        outputs.push(registers[&output]);
    }

    if state.len() > 256 {
        panic!("too many registers used");
    }

    VMProgram {
        opcodes,
        outputs,
        registers: state.len() as u8,
    }
}
