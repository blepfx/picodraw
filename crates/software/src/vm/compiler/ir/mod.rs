use crate::{VMOp, VMOpcode};
use bumpalo::{Bump, collections::Vec};
use std::{fmt::Debug, hash::Hash};

mod dynasplit;
mod graph2ir;
mod hashcons;
mod lowering;
mod peephole;

pub use dynasplit::split_static_dynamic;
pub use graph2ir::IRBuilder;
pub use hashcons::optimize_hashcons;
pub use lowering::lower_to_opcodes;
pub use peephole::optimize_peephole;

#[derive(Debug)]
pub struct VMProgram<'a> {
    pub opcodes: Vec<'a, VMOpcode>,
    pub outputs: Vec<'a, u8>,
    pub registers: u8,
}

#[derive(Debug)]
pub struct IRProgram<'a> {
    pub outputs: &'a [IR<'a>],
}

impl<'a> IRProgram<'a> {
    pub fn visit_ops<T>(
        &self,
        arena: &'a Bump,
        mut state: T,
        mut enter: impl FnMut(&mut T, IR<'a>, Option<IR<'a>>) -> bool,
        mut exit: impl FnMut(&mut T, IR<'a>, Option<IR<'a>>),
    ) {
        enum Visit<'a> {
            Enter(IR<'a>, Option<IR<'a>>),
            Exit(IR<'a>, Option<IR<'a>>),
        }

        let mut stack = Vec::new_in(arena);

        for ir in self.outputs {
            stack.push(Visit::Enter(*ir, None));
        }

        loop {
            match stack.pop() {
                Some(Visit::Enter(ir, from)) => {
                    if enter(&mut state, ir, from) {
                        stack.push(Visit::Exit(ir, from));
                        ir.visit_children(|x| stack.push(Visit::Enter(x, Some(ir))));
                    }
                }

                Some(Visit::Exit(ir, from)) => {
                    exit(&mut state, ir, from);
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
