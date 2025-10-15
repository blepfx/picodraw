use super::{IR, IRProgram, IRVisit, VMOp};
use bumpalo::Bump;
use std::collections::HashMap;

pub fn optimize_peephole<'a>(
    program: &IRProgram<'a>,
    arena: &'a Bump,
    peeper: impl Fn(&'a Bump, IR<'a>) -> IR<'a>,
) -> IRProgram<'a> {
    let mut mapping = HashMap::new();
    program.visit_dfs(arena, |visit| match visit {
        IRVisit::Enter(ir, _) => !mapping.contains_key(&ir),
        IRVisit::Exit(ir, _) => {
            mapping.insert(ir, peeper(arena, ir.map_children(arena, |ir| mapping[&ir])));
            true
        }
    });

    IRProgram {
        outputs: arena.alloc_slice_fill_iter(program.outputs.iter().map(|ir| mapping[ir])),
    }
}

/// pre-chew some ops for future optimization passes
///
/// namely, division is getting split into multiplication by a reciprocal, which helps with dynamic splitting and hoisting the division op into the static path
pub fn peeper_split<'a>(arena: &'a Bump, ir: IR<'a>) -> IR<'a> {
    use VMOp::*;
    match *ir.0 {
        DivF(a, b, _) => IR::new(
            arena,
            VMOp::MulF(a, IR::new(arena, DivF(IR::new(arena, LitF(1.0, ())), b, ())), ()),
        ),

        _ => ir,
    }
}

// whos peeping they hole rn
/// merge ops if possible, do constant folding and other misc optimizations
pub fn peeper_join<'a>(arena: &'a Bump, ir: IR<'a>) -> IR<'a> {
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
            (LitI(x, _), LitI(y, _)) => IR::new(arena, LitI(x.wrapping_sub(*y), ())),
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
            (LitF(x, _), _) => IR::new(arena, MulCF(*x, b, ())),
            (_, LitF(y, _)) => IR::new(arena, MulCF(*y, a, ())),

            (_, DivF(IR(LitF(1.0, _)), z, _)) => IR::new(arena, DivF(a, *z, ())),
            (DivF(IR(LitF(1.0, _)), z, _), _) => IR::new(arena, DivF(b, *z, ())),

            (MulF(x, y, _), _) => IR::new(arena, Mul3F(*x, *y, b, ())),
            (_, MulF(y, z, _)) => IR::new(arena, Mul3F(a, *y, *z, ())),

            _ => ir,
        },

        MulI(a, b, _) => match (a.0, b.0) {
            (LitI(x, _), LitI(y, _)) => IR::new(arena, LitI(x.wrapping_mul(*y), ())),
            (LitI(0, _), _) | (_, LitI(0, _)) => IR::new(arena, LitI(0, ())),
            (LitI(1, _), _) => b,
            (_, LitI(1, _)) => a,
            (LitI(x, _), _) => IR::new(arena, MulCI(*x, b, ())),
            (_, LitI(y, _)) => IR::new(arena, MulCI(*y, a, ())),
            (MulI(x, y, _), _) => IR::new(arena, Mul3I(*x, *y, b, ())),
            (_, MulI(y, z, _)) => IR::new(arena, Mul3I(a, *y, *z, ())),
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
            (LitF(x, _), _) => IR::new(arena, MinCF(*x, b, ())),
            (_, LitF(y, _)) => IR::new(arena, MinCF(*y, a, ())),
            _ => ir,
        },

        MinI(a, b, _) => match (a.0, b.0) {
            (LitI(x, _), LitI(y, _)) => IR::new(arena, LitI((*x).min(*y), ())),
            (LitI(x, _), _) => IR::new(arena, MinCI(*x, b, ())),
            (_, LitI(y, _)) => IR::new(arena, MinCI(*y, a, ())),
            _ => ir,
        },

        MaxF(a, b, _) => match (a.0, b.0) {
            (LitF(x, _), LitF(y, _)) => IR::new(arena, LitF(x.max(*y), ())),
            (LitF(x, _), _) => IR::new(arena, MaxCF(*x, b, ())),
            (_, LitF(y, _)) => IR::new(arena, MaxCF(*y, a, ())),
            _ => ir,
        },

        MaxI(a, b, _) => match (a.0, b.0) {
            (LitI(x, _), LitI(y, _)) => IR::new(arena, LitI((*x).max(*y), ())),
            (LitI(x, _), _) => IR::new(arena, MaxCI(*x, b, ())),
            (_, LitI(y, _)) => IR::new(arena, MaxCI(*y, a, ())),
            _ => ir,
        },

        PowF(a, b, _) => match (a.0, b.0) {
            (LitF(x, _), LitF(y, _)) => IR::new(arena, LitF(x.powf(*y), ())),
            (LitF(0.0, _), _) => IR::new(arena, LitF(0.0, ())),
            (_, LitF(0.0, _)) => IR::new(arena, LitF(1.0, ())),
            (_, LitF(1.0, _)) => a,
            (_, LitF(0.5, _)) => IR::new(arena, SqrtF(a, ())),
            (_, LitF(2.0, _)) => IR::new(arena, MulF(a, a, ())),
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
