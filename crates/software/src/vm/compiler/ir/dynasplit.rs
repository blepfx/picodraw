use super::{IR, IRProgram, VMOp};
use bumpalo::Bump;
use std::collections::HashMap;

/// split the IR graph into 2 parts: a "static" per quad graph, and "dynamic" per pixel graph
/// "static" graph is a graph that can be executed once per quad,
/// and "dynamic" graph is a graph that must be run for every pixel
pub fn split_static_dynamic<'a>(program: &IRProgram<'a>, arena: &'a Bump) -> (IRProgram<'a>, IRProgram<'a>) {
    let mut dynamic = HashMap::new();

    program.visit_ops(
        arena,
        &mut dynamic,
        |mapping, ir, _| !mapping.contains_key(&ir),
        |mapping, ir, _| {
            let is_dynamic = match ir.0 {
                VMOp::PosX(_) => true,
                VMOp::PosY(_) => true,
                _ => {
                    let mut result = false;
                    ir.0.map_inputs(|i| {
                        if mapping[&i] {
                            result = true;
                        }
                    });
                    result
                }
            };

            mapping.insert(ir, is_dynamic);
        },
    );

    let mut boundary = Vec::new();
    let mut mapping = HashMap::new();

    program.visit_ops(
        arena,
        &mut mapping,
        |mapping, ir, from| !mapping.contains_key(&ir) && from.map(|from| dynamic[&from]).unwrap_or(true),
        |mapping, ir, _| {
            if dynamic[&ir] || !can_be_a_boundary(&ir) {
                mapping.insert(ir, ir.map_children(arena, |ir| mapping[&ir]));
            } else {
                let boundary_idx = boundary.len();
                boundary.push(ir);
                mapping.insert(ir, IR::new(arena, VMOp::Read(boundary_idx as u32, ())));
            }
        },
    );

    let program_static = IRProgram {
        outputs: arena.alloc_slice_fill_iter(boundary.iter().copied()),
    };

    let program_dynamic = IRProgram {
        outputs: arena.alloc_slice_fill_iter(program.outputs.iter().map(|ir| mapping[ir])),
    };

    (program_static, program_dynamic)
}

fn can_be_a_boundary(ir: &IR) -> bool {
    match ir.0 {
        VMOp::LitF(_, _) => false,
        VMOp::LitI(_, _) => false,
        VMOp::QuadB(_) => false,
        VMOp::QuadT(_) => false,
        VMOp::QuadL(_) => false,
        VMOp::QuadR(_) => false,
        VMOp::ResX(_) => false,
        VMOp::ResY(_) => false,
        VMOp::TexH(_, _) => false,
        VMOp::TexW(_, _) => false,
        _ => true,
    }
}
