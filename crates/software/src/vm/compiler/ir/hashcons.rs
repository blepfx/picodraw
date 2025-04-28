use super::{IR, IRProgram, VMOp};
use bumpalo::Bump;
use std::{collections::HashMap, hash::Hash, mem::discriminant};

/// common subexpression elimination
/// this is a simple hashconsing pass that will eliminate duplicate IR nodes
pub fn optimize_hashcons<'a>(program: &IRProgram<'a>, arena: &'a Bump) -> IRProgram<'a> {
    let mut forward = HashMap::new();
    let mut reverse = HashMap::new();

    program.visit_ops(
        arena,
        (&mut forward, &mut reverse),
        |(_, reverse), ir, _| !reverse.contains_key(&ir),
        |(forward, reverse), ir, _| {
            let key = IRKey(ir.0.map_inputs(|input| reverse[&input]));
            let normalized = *forward.entry(key).or_insert(ir.map_children(arena, |ir| reverse[&ir]));
            reverse.insert(ir, normalized);
        },
    );

    IRProgram {
        outputs: arena.alloc_slice_fill_iter(program.outputs.iter().map(|ir| reverse[ir])),
    }
}

/// an IR graph node that can be hashed and compared "structurally"
#[derive(Debug)]
struct IRKey<'a>(VMOp<IR<'a>, ()>);

impl<'a> Eq for IRKey<'a> {}
impl<'a> PartialEq for IRKey<'a> {
    fn eq(&self, other: &Self) -> bool {
        use VMOp::*;
        match (self.0, other.0) {
            (AddI(a, b, _), AddI(x, y, _))
            | (MulI(a, b, _), MulI(x, y, _))
            | (MaxI(a, b, _), MaxI(x, y, _))
            | (MinI(a, b, _), MinI(x, y, _))
            | (AndI(a, b, _), AndI(x, y, _))
            | (OrI(a, b, _), OrI(x, y, _))
            | (XorI(a, b, _), XorI(x, y, _))
            | (EqI(a, b, _), EqI(x, y, _))
            | (AddF(a, b, _), AddF(x, y, _))
            | (MulF(a, b, _), MulF(x, y, _))
            | (MaxF(a, b, _), MaxF(x, y, _))
            | (MinF(a, b, _), MinF(x, y, _))
            | (EqF(a, b, _), EqF(x, y, _)) => (a == x && b == y) || (a == y && b == x),

            _ => self.0 == other.0,
        }
    }
}
impl<'a> Hash for IRKey<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        use VMOp::*;

        discriminant(&self.0).hash(state);
        match self.0 {
            AddI(a, b, _)
            | MulI(a, b, _)
            | MaxI(a, b, _)
            | MinI(a, b, _)
            | AndI(a, b, _)
            | OrI(a, b, _)
            | XorI(a, b, _)
            | EqI(a, b, _)
            | AddF(a, b, _)
            | MulF(a, b, _)
            | MaxF(a, b, _)
            | MinF(a, b, _)
            | EqF(a, b, _) => {
                (a.0 as *const _ as usize ^ b.0 as *const _ as usize).hash(state);
            }

            Read(x, _) => x.hash(state),
            LitF(x, _) => x.to_bits().hash(state),
            LitI(x, _) => x.hash(state),

            AddCF(x, b, _) | MulCF(x, b, _) | MinCF(x, b, _) | MaxCF(x, b, _) => {
                x.to_bits().hash(state);
                b.hash(state);
            }

            AddCI(x, b, _) | MulCI(x, b, _) | MinCI(x, b, _) | MaxCI(x, b, _) => {
                x.hash(state);
                b.hash(state);
            }

            TexW(x, _) | TexH(x, _) => x.hash(state),
            Tex(x, c, f, _, _, _) => {
                x.hash(state);
                c.hash(state);
                f.hash(state);
            }

            _ => {
                self.0.map_inputs(|i| std::ptr::hash(i.0, state));
            }
        }
    }
}
