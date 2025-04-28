use super::{IR, IRProgram, VMOp};
use bumpalo::Bump;
use std::collections::HashMap;

/// do peephole optimizations and constant folding on the IR graph
pub fn optimize_peephole<'a>(program: &IRProgram<'a>, arena: &'a Bump) -> IRProgram<'a> {
    let mut mapping = HashMap::new();
    program.visit_ops(
        arena,
        &mut mapping,
        |mapping, ir, _| !mapping.contains_key(&ir),
        |mapping, ir, _| {
            mapping.insert(ir, single_peephole(arena, ir.map_children(arena, |ir| mapping[&ir])));
        },
    );

    IRProgram {
        outputs: arena.alloc_slice_fill_iter(program.outputs.iter().map(|ir| mapping[ir])),
    }
}

// whos peeping they hole rn
fn single_peephole<'a>(arena: &'a Bump, ir: IR<'a>) -> IR<'a> {
    use VMOp::*;
    match *ir.0 {
        AddF(a, b, _) => match (a.0, b.0) {
            (LitF(x, _), LitF(y, _)) => IR::new(arena, LitF(x + y, ())),
            (LitF(0.0, _), _) => b,
            (_, LitF(0.0, _)) => a,
            (LitF(x, _), b) => IR::new(arena, AddCF(*x, IR(b), ())),
            (a, LitF(y, _)) => IR::new(arena, AddCF(*y, IR(a), ())),
            (AddF(x, y, _), z) => IR::new(arena, Add3F(*x, *y, IR(z), ())),
            (x, AddF(y, z, _)) => IR::new(arena, Add3F(IR(x), *y, *z, ())),
            _ => ir,
        },

        AddI(a, b, _) => match (a.0, b.0) {
            (LitI(x, _), LitI(y, _)) => IR::new(arena, LitI(x.wrapping_add(*y), ())),
            (LitI(0, _), _) => b,
            (_, LitI(0, _)) => a,
            (LitI(x, _), b) => IR::new(arena, AddCI(*x, IR(b), ())),
            (a, LitI(y, _)) => IR::new(arena, AddCI(*y, IR(a), ())),
            (AddI(x, y, _), z) => IR::new(arena, Add3I(*x, *y, IR(z), ())),
            (x, AddI(y, z, _)) => IR::new(arena, Add3I(IR(x), *y, *z, ())),
            _ => ir,
        },

        SubF(a, b, _) => match (a.0, b.0) {
            (LitF(x, _), LitF(y, _)) => IR::new(arena, LitF(x - y, ())),
            (LitF(0.0, _), _) => IR::new(arena, NegF(b, ())),
            (_, LitF(0.0, _)) => a,
            (LitF(x, _), b) => IR::new(arena, SubCF(*x, IR(b), ())),
            (a, LitF(y, _)) => IR::new(arena, AddCF(-y, IR(a), ())),
            _ => ir,
        },

        SubI(a, b, _) => match (a.0, b.0) {
            (LitI(x, _), LitI(y, _)) => IR::new(arena, LitI(x.wrapping_sub(-y), ())),
            (LitI(0, _), _) => IR::new(arena, NegI(b, ())),
            (_, LitI(0, _)) => a,
            (LitI(x, _), b) => IR::new(arena, SubCI(*x, IR(b), ())),
            (a, LitI(y, _)) => IR::new(arena, AddCI(-y, IR(a), ())),
            _ => ir,
        },

        MulF(a, b, _) => match (a.0, b.0) {
            (LitF(x, _), LitF(y, _)) => IR::new(arena, LitF(x * y, ())),
            (LitF(0.0, _), _) | (_, LitF(0.0, _)) => IR::new(arena, LitF(0.0, ())),
            (LitF(1.0, _), _) => b,
            (_, LitF(1.0, _)) => a,
            (LitF(x, _), b) => IR::new(arena, MulCF(*x, IR(b), ())),
            (a, LitF(y, _)) => IR::new(arena, MulCF(*y, IR(a), ())),
            (MulF(x, y, _), z) => IR::new(arena, Mul3F(*x, *y, IR(z), ())),
            (x, MulF(y, z, _)) => IR::new(arena, Mul3F(IR(x), *y, *z, ())),
            _ => ir,
        },

        MulI(a, b, _) => match (a.0, b.0) {
            (LitI(x, _), LitI(y, _)) => IR::new(arena, LitI(x.wrapping_mul(*y), ())),
            (LitI(0, _), _) | (_, LitI(0, _)) => IR::new(arena, LitI(0, ())),
            (LitI(1, _), _) => b,
            (_, LitI(1, _)) => a,
            (LitI(x, _), b) => IR::new(arena, MulCI(*x, IR(b), ())),
            (a, LitI(y, _)) => IR::new(arena, MulCI(*y, IR(a), ())),
            (MulI(x, y, _), z) => IR::new(arena, Mul3I(*x, *y, IR(z), ())),
            (x, MulI(y, z, _)) => IR::new(arena, Mul3I(IR(x), *y, *z, ())),
            _ => ir,
        },

        DivF(a, b, _) => match (a.0, b.0) {
            (LitF(x, _), LitF(y, _)) => IR::new(arena, LitF(x / y, ())),
            (LitF(0.0, _), _) => IR::new(arena, LitF(0.0, ())),
            (_, LitF(1.0, _)) => a,
            (_, LitF(x, _)) => IR::new(arena, MulCF(x.recip(), a, ())),
            _ => ir,
        },

        DivI(a, b, _) => match (a.0, b.0) {
            (LitI(x, _), LitI(y, _)) => IR::new(arena, LitI(x.wrapping_div(*y), ())),
            (LitI(0, _), _) => IR::new(arena, LitI(0, ())),
            (_, LitI(1, _)) => a,
            _ => ir,
        },

        NegF(a, _) => match a.0 {
            LitF(x, _) => IR::new(arena, LitF(-x, ())),
            NegF(b, _) => *b,
            SubF(a, b, _) => IR::new(arena, SubF(*b, *a, ())),
            _ => ir,
        },

        NegI(a, _) => match a.0 {
            LitI(x, _) => IR::new(arena, LitI(x.wrapping_neg(), ())),
            NegI(b, _) => *b,
            SubI(a, b, _) => IR::new(arena, SubI(*b, *a, ())),
            _ => ir,
        },

        MinF(a, b, _) => match (a.0, b.0) {
            (LitF(x, _), LitF(y, _)) => IR::new(arena, LitF(x.min(*y), ())),
            (LitF(x, _), b) => IR::new(arena, MinCF(*x, IR(b), ())),
            (a, LitF(y, _)) => IR::new(arena, MinCF(*y, IR(a), ())),
            _ => ir,
        },

        MinI(a, b, _) => match (a.0, b.0) {
            (LitI(x, _), LitI(y, _)) => IR::new(arena, LitI((*x).min(*y), ())),
            (LitI(x, _), b) => IR::new(arena, MinCI(*x, IR(b), ())),
            (a, LitI(y, _)) => IR::new(arena, MinCI(*y, IR(a), ())),
            _ => ir,
        },

        MaxF(a, b, _) => match (a.0, b.0) {
            (LitF(x, _), LitF(y, _)) => IR::new(arena, LitF(x.max(*y), ())),
            (LitF(x, _), b) => IR::new(arena, MaxCF(*x, IR(b), ())),
            (a, LitF(y, _)) => IR::new(arena, MaxCF(*y, IR(a), ())),
            _ => ir,
        },

        MaxI(a, b, _) => match (a.0, b.0) {
            (LitI(x, _), LitI(y, _)) => IR::new(arena, LitI((*x).max(*y), ())),
            (LitI(x, _), b) => IR::new(arena, MaxCI(*x, IR(b), ())),
            (a, LitI(y, _)) => IR::new(arena, MaxCI(*y, IR(a), ())),
            _ => ir,
        },

        PowF(a, b, _) => match (a.0, b.0) {
            (LitF(x, _), LitF(y, _)) => IR::new(arena, LitF(x.powf(*y), ())),
            (LitF(0.0, _), _) => IR::new(arena, LitF(0.0, ())),
            (_, LitF(0.0, _)) => IR::new(arena, LitF(1.0, ())),
            (_, LitF(1.0, _)) => a,

            (x, LitF(0.5, _)) => IR::new(arena, SqrtF(IR(x), ())),
            (x, LitF(2.0, _)) => IR::new(arena, MulF(IR(x), IR(x), ())),
            (x, LitF(3.0, _)) => {
                let x2 = IR::new(arena, MulF(IR(x), IR(x), ()));
                IR::new(arena, MulF(IR(x), x2, ()))
            }
            (x, LitF(4.0, _)) => {
                let x2 = IR::new(arena, MulF(IR(x), IR(x), ()));
                IR::new(arena, MulF(x2, x2, ()))
            }

            _ => ir,
        },

        Select(cond, a, b, _) => match (cond.0, a.0, b.0) {
            (LitI(0, _), _, _) => b,
            (LitI(-1, _), _, _) => a,
            (NotI(c, _), _, _) => IR::new(arena, Select(*c, b, a, ())),
            _ => ir,
        },

        _ => ir,
    }
}
