use super::{IR, IRProgram, IRVisit, VMOp};
use bumpalo::Bump;
use std::collections::HashMap;

/// split the IR graph into 2 parts: a "static" per quad graph, and "dynamic" per pixel graph
/// "static" graph is a graph that _can_ be executed once per quad,
/// and "dynamic" graph is a graph that _must_ be run for every pixel
pub fn split_static_dynamic<'a>(program: &IRProgram<'a>, arena: &'a Bump) -> (IRProgram<'a>, IRProgram<'a>) {
    let mut dynamic = HashMap::new();

    program.visit_dfs(arena, |visit| match visit {
        IRVisit::Enter(ir, _) => !dynamic.contains_key(&ir),
        IRVisit::Exit(ir, _) => {
            let is_dynamic = match ir.0 {
                VMOp::PosX(_) => true,
                VMOp::PosY(_) => true,
                _ => {
                    let mut result = false;
                    ir.0.map_inputs(|i| {
                        if dynamic[&i] {
                            result = true;
                        }
                    });
                    result
                }
            };

            dynamic.insert(ir, is_dynamic);
            true
        }
    });

    let mut boundary = Vec::new();
    let mut mapping = HashMap::new();

    program.visit_dfs(arena, |visit| match visit {
        IRVisit::Enter(ir, from) => !mapping.contains_key(&ir) && from.map(|from| dynamic[&from]).unwrap_or(true),
        IRVisit::Exit(ir, _) => {
            if dynamic[&ir] || !can_be_a_boundary(&ir) {
                mapping.insert(ir, ir.map_children(arena, |ir| mapping[&ir]));
            } else {
                let boundary_idx = boundary.len();
                boundary.push(ir);
                mapping.insert(ir, IR::new(arena, VMOp::Read(boundary_idx as u32, ())));
            }

            true
        }
    });

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
