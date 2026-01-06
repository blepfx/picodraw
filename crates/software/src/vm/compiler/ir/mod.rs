use crate::vm::VMOp;
use bumpalo::{Bump, collections::Vec};
use std::{fmt::Debug, hash::Hash};

mod dynasplit;
mod graph2ir;
mod hashcons;
mod ir2opcode;
mod peephole;

pub use dynasplit::*;
pub use graph2ir::*;
pub use hashcons::*;
pub use ir2opcode::*;
pub use peephole::*;

pub enum IRVisit<'a> {
    Enter(IR<'a>, Option<IR<'a>>),
    Exit(IR<'a>, Option<IR<'a>>),
}

#[derive(Debug)]
pub struct IRProgram<'a> {
    pub outputs: &'a [IR<'a>],
}

impl<'a> IRProgram<'a> {
    pub fn visit_dfs(&self, arena: &'a Bump, mut visit: impl FnMut(IRVisit<'a>) -> bool) {
        let mut stack = Vec::new_in(arena);

        for ir in self.outputs {
            stack.push(IRVisit::Enter(*ir, None));
        }

        loop {
            match stack.pop() {
                Some(IRVisit::Enter(ir, from)) => {
                    if visit(IRVisit::Enter(ir, from)) {
                        stack.push(IRVisit::Exit(ir, from));
                        ir.visit_children(|x| stack.push(IRVisit::Enter(x, Some(ir))));
                    }
                }

                Some(IRVisit::Exit(ir, from)) => {
                    visit(IRVisit::Exit(ir, from));
                }

                None => break,
            }
        }
    }
}

#[derive(Clone, Copy)]
pub struct IR<'a>(pub &'a VMOp<IR<'a>, ()>);

impl<'a> IR<'a> {
    pub fn new(arena: &'a Bump, op: VMOp<IR<'a>, ()>) -> Self {
        IR(arena.alloc(op))
    }

    pub fn visit_children(&self, f: impl FnMut(IR<'a>)) {
        self.0.map_inputs(f);
    }

    pub fn map_children(&self, arena: &'a Bump, f: impl FnMut(IR<'a>) -> IR<'a>) -> IR<'a> {
        IR(arena.alloc(self.0.map_inputs(f)))
    }
}

impl<'a> Eq for IR<'a> {}
impl<'a> PartialEq for IR<'a> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.0, other.0)
    }
}

impl<'a> Hash for IR<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::ptr::hash(self.0, state);
    }
}

impl<'a> Debug for IR<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "IR({:p} := {:?})", self.0, self.0.map_inputs(|x| x.0 as *const _))
    }
}
