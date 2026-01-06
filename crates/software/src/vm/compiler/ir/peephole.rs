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

/// constant folding and arithmetic reduction (pre static-dynamic split)
///
/// expects split ops (MulF(_, RecipF) instead of DivF)
pub fn peeper_const<'a>(arena: &'a Bump, ir: IR<'a>) -> IR<'a> {
    use VMOp::*;
    match *ir.0 {
        AddF(a, b, _) => match (a.0, b.0) {
            // zero identity
            (LitF(0.0, _), _) => b,
            (_, LitF(0.0, _)) => a,

            // constant
            (LitF(x, _), LitF(y, _)) => IR::new(arena, LitF(x + y, ())),

            _ => ir,
        },

        NegF(a, _) => match a.0 {
            // constant
            LitF(x, _) => IR::new(arena, LitF(-x, ())),

            // -(-x) = x
            NegF(x, _) => *x,

            // -(a-b) = (b-a)
            SubF(a, b, _) => IR::new(arena, SubF(*b, *a, ())),

            _ => ir,
        },

        MulF(a, b, _) => match (a.0, b.0) {
            // zero identity
            (LitF(0.0, _), _) | (_, LitF(0.0, _)) => IR::new(arena, LitF(0.0, ())),

            // mult identity
            (LitF(1.0, _), _) => b,
            (_, LitF(1.0, _)) => a,

            // constant
            (LitF(x, _), LitF(y, _)) => IR::new(arena, LitF(x * y, ())),

            // sqrt(x)*sqrt(y) = sqrt(xy)
            (SqrtF(x, _), SqrtF(y, _)) => IR::new(arena, SqrtF(IR::new(arena, MulF(*x, *y, ())), ())),

            _ => ir,
        },

        RecipF(a, _) => match a.0 {
            // constant
            LitF(x, _) => IR::new(arena, LitF(x.recip(), ())),

            // 1/(1/x) = x
            RecipF(x, _) => *x,

            _ => ir,
        },

        SqrtF(a, _) => match a.0 {
            // constant
            LitF(x, _) => IR::new(arena, LitF(x.sqrt(), ())),

            // sqrt(x^2) = x
            MulF(x, y, _) if x == y => *x,

            _ => ir,
        },

        PowF(a, b, _) => match (a.0, b.0) {
            // x^1 = x
            (_, LitF(1.0, _)) => a,

            // constant
            (LitF(x, _), LitF(y, _)) => IR::new(arena, LitF(x.powf(*y), ())),

            // x^0.5 = sqrt(x)
            (_, LitF(0.5, _)) => IR::new(arena, SqrtF(a, ())),

            // x^-0.5 = sqrt(x)
            (_, LitF(-0.5, _)) => IR::new(arena, RecipSqrtF(a, ())),

            // x^-1 = sqrt(x)
            (_, LitF(-1.0, _)) => IR::new(arena, RecipF(a, ())),

            // x^2 = x*x
            (_, LitF(2.0, _)) => IR::new(arena, MulF(a, a, ())),

            _ => ir,
        },

        MinF(a, b, _) => match (a.0, b.0) {
            // constant
            (LitF(x, _), LitF(y, _)) => IR::new(arena, LitF(x.min(*y), ())),

            _ => ir,
        },

        MaxF(a, b, _) => match (a.0, b.0) {
            // constant
            (LitF(x, _), LitF(y, _)) => IR::new(arena, LitF(x.max(*y), ())),

            _ => ir,
        },

        AddI(a, b, _) => match (a.0, b.0) {
            // zero identity
            (LitI(0, _), _) => b,
            (_, LitI(0, _)) => a,

            // constant
            (LitI(x, _), LitI(y, _)) => IR::new(arena, LitI(x.wrapping_add(*y), ())),

            _ => ir,
        },

        NegI(a, _) => match a.0 {
            // constant
            LitI(x, _) => IR::new(arena, LitI(x.wrapping_neg(), ())),

            // -(-x) = x
            NegI(b, _) => *b,

            // -(a-b) = b-a
            SubI(a, b, _) => IR::new(arena, SubI(*b, *a, ())),

            _ => ir,
        },

        MulI(a, b, _) => match (a.0, b.0) {
            // zero identity
            (LitI(0, _), _) | (_, LitI(0, _)) => IR::new(arena, LitI(0, ())),

            // mult identity
            (LitI(1, _), _) => b,
            (_, LitI(1, _)) => a,

            // constant
            (LitI(x, _), LitI(y, _)) => IR::new(arena, LitI(x.wrapping_mul(*y), ())),

            _ => ir,
        },

        DivI(a, b, _) => match (a.0, b.0) {
            // 0/x = 0
            (LitI(0, _), _) => IR::new(arena, LitI(0, ())),

            // x/1 = x
            (_, LitI(1, _)) => a,

            // constant
            (LitI(x, _), LitI(y, _)) => IR::new(arena, LitI(x.wrapping_div(*y), ())),

            _ => ir,
        },

        MinI(a, b, _) => match (a.0, b.0) {
            // constant
            (LitI(x, _), LitI(y, _)) => IR::new(arena, LitI((*x).min(*y), ())),

            _ => ir,
        },

        MaxI(a, b, _) => match (a.0, b.0) {
            // constant
            (LitI(x, _), LitI(y, _)) => IR::new(arena, LitI((*x).max(*y), ())),

            _ => ir,
        },

        Select(cond, a, b, _) => match (cond.0, a.0, b.0) {
            // constant
            (LitI(0, _), _, _) => b,
            (LitI(-1, _), _, _) => a,

            // select(!x, a, b) = select(x, b, a)
            (NotI(c, _), _, _) => IR::new(arena, Select(*c, b, a, ())),
            _ => ir,
        },

        CastAsI(a, _) => match a.0 {
            // constant
            LitF(x, _) => IR::new(arena, LitI((*x) as i32, ())),

            // int(float(x)) = x
            CastAsF(x, _) => *x,

            _ => ir,
        },

        _ => ir,
    }
}

/// pre-chew some ops for future optimization passes
///
/// namely, division is getting split into multiplication by a reciprocal, which helps with dynamic splitting and hoisting the division op into the static path
pub fn peeper_split<'a>(arena: &'a Bump, ir: IR<'a>) -> IR<'a> {
    use VMOp::*;
    match *ir.0 {
        // x/y = x*(1/y)
        DivF(a, b, _) => IR::new(arena, MulF(a, IR::new(arena, RecipF(b, ())), ())),

        // x-y = x+(-y)
        SubF(a, b, _) => IR::new(arena, AddF(a, IR::new(arena, NegF(b, ())), ())),

        // x-y = x+(-y)
        SubI(a, b, _) => IR::new(arena, AddI(a, IR::new(arena, NegI(b, ())), ())),
        _ => ir,
    }
}

/// merge ops into more complex operations (AddF(Read, _) to AddRF, etc)
/// expects split ops (MulF(_, RecipF) instead of DivF)
pub fn peeper_join<'a>(arena: &'a Bump, ir: IR<'a>) -> IR<'a> {
    use VMOp::*;
    match *ir.0 {
        AddF(a, b, _) => match (a.0, b.0) {
            // constant-add
            (LitF(x, _), NegF(y, _)) => IR::new(arena, SubCF(*x, *y, ())),
            (LitF(x, _), _) => IR::new(arena, AddCF(*x, b, ())),
            (_, LitF(y, _)) => IR::new(arena, AddCF(*y, a, ())),

            // read-add
            (Read(x, _), _) => IR::new(arena, AddRF(*x, b, ())),
            (_, Read(y, _)) => IR::new(arena, AddRF(*y, a, ())),

            // mul-add
            (MulF(x, y, _), NegF(z, _)) => IR::new(arena, MulSubF(*x, *y, *z, ())),
            (MulF(x, y, _), _) => IR::new(arena, MulAddF(*x, *y, b, ())),

            // subtraction
            (_, NegF(y, _)) => IR::new(arena, SubF(a, *y, ())),
            (NegF(y, _), _) => IR::new(arena, SubF(b, *y, ())),

            _ => ir,
        },

        MulF(a, b, _) => match (a.0, b.0) {
            // constant-mul
            (LitF(x, _), _) => IR::new(arena, MulCF(*x, b, ())),
            (_, LitF(y, _)) => IR::new(arena, MulCF(*y, a, ())),

            // read-mul
            (Read(x, _), _) => IR::new(arena, MulRF(*x, b, ())),
            (_, Read(y, _)) => IR::new(arena, MulRF(*y, a, ())),

            // division
            (_, RecipF(z, _)) => IR::new(arena, DivF(a, *z, ())),
            (RecipF(z, _), _) => IR::new(arena, DivF(b, *z, ())),

            _ => ir,
        },

        RecipF(a, _) => match a.0 {
            // 1/(sqrt(x)) = rsqrt(x)
            SqrtF(x, _) => IR::new(arena, RecipSqrtF(*x, ())),

            // 1/(a/b) = b/a
            DivF(a, b, _) => IR::new(arena, DivF(*b, *a, ())),

            _ => ir,
        },

        SqrtF(a, _) => match a.0 {
            // sqrt(1/x) = rsqrt(x)
            RecipF(x, _) => IR::new(arena, RecipSqrtF(*x, ())),

            _ => ir,
        },

        MinF(a, b, _) => match (a.0, b.0) {
            // constant-min
            (LitF(x, _), _) => IR::new(arena, MinCF(*x, b, ())),
            (_, LitF(y, _)) => IR::new(arena, MinCF(*y, a, ())),

            _ => ir,
        },

        MaxF(a, b, _) => match (a.0, b.0) {
            // constant-max
            (LitF(x, _), _) => IR::new(arena, MaxCF(*x, b, ())),
            (_, LitF(y, _)) => IR::new(arena, MaxCF(*y, a, ())),

            _ => ir,
        },

        AddI(a, b, _) => match (a.0, b.0) {
            // constant-add
            (LitI(x, _), NegI(y, _)) => IR::new(arena, SubCI(*x, *y, ())),
            (LitI(x, _), _) => IR::new(arena, AddCI(*x, b, ())),
            (_, LitI(y, _)) => IR::new(arena, AddCI(*y, a, ())),

            _ => ir,
        },

        MulI(a, b, _) => match (a.0, b.0) {
            // constant-mul
            (LitI(x, _), _) => IR::new(arena, MulCI(*x, b, ())),
            (_, LitI(y, _)) => IR::new(arena, MulCI(*y, a, ())),

            _ => ir,
        },

        MinI(a, b, _) => match (a.0, b.0) {
            // constant-min
            (LitI(x, _), _) => IR::new(arena, MinCI(*x, b, ())),
            (_, LitI(y, _)) => IR::new(arena, MinCI(*y, a, ())),
            _ => ir,
        },

        MaxI(a, b, _) => match (a.0, b.0) {
            // constant-max
            (LitI(x, _), _) => IR::new(arena, MaxCI(*x, b, ())),
            (_, LitI(y, _)) => IR::new(arena, MaxCI(*y, a, ())),
            _ => ir,
        },

        _ => ir,
    }
}
