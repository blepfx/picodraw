use super::{REGISTER_COUNT, VMOp, VMOpcode};
use bumpalo::Bump;
use picodraw_core::{Graph, graph::OpInput};

#[derive(Debug)]
pub struct CompiledShader {
    opcodes: Vec<VMOpcode>,
    output: [u8; 4],
    slots_data: u32,
    slots_texture: u8,
}

impl CompiledShader {
    pub fn compile(arena: &Bump, graph: &Graph) -> Self {
        let mut builder = ir::IRBuilder::new(arena);
        let mut slots_data = 0;
        let mut slots_texture = 0;

        builder.emit_graph_all(graph, |input, addr, builder| match input {
            OpInput::F32 => {
                builder.set_graph(addr, 0, ir::emit(arena, VMOp::ReadF(slots_data, ())));
                slots_data += 1;
            }

            OpInput::TextureRender | OpInput::TextureStatic => {
                builder.set_texture(addr, slots_texture);
                slots_texture += 1;
            }

            x if x.value_type().is_int() => {
                builder.set_graph(addr, 0, ir::emit(arena, VMOp::ReadI(slots_data, ())));
                slots_data += 1;
            }

            _ => todo!(),
        });

        let program = ir::IRProgram {
            outputs: arena.alloc([
                builder.get_graph(graph.output(), 0),
                builder.get_graph(graph.output(), 1),
                builder.get_graph(graph.output(), 2),
                builder.get_graph(graph.output(), 3),
            ]),
        };

        let program = program.optimize_peephole(arena);
        let program = program.optimize_hashcons(arena);
        let program = program.lower_to_opcodes(arena);

        assert!(
            program.register_count <= REGISTER_COUNT as u8,
            "too many registers used"
        );

        Self {
            output: [
                program.outputs[0],
                program.outputs[1],
                program.outputs[2],
                program.outputs[3],
            ],
            opcodes: program.opcodes.to_vec(),
            slots_data,
            slots_texture,
        }
    }

    pub fn opcodes(&self) -> &[VMOpcode] {
        &self.opcodes
    }

    pub fn output_register(&self, index: usize) -> u8 {
        self.output[index]
    }

    pub fn data_slots(&self) -> u32 {
        self.slots_data
    }

    pub fn texture_slots(&self) -> u8 {
        self.slots_texture
    }
}

// TODO: optimizations
// + common subexpression elimination
// + dead code elimination
// - move register ops closer to their use
// + constant folding
// + peephole optimizations (fold AddF(LitF(x), y) -> AddCF(x, y))

mod ir {
    use crate::vm::{VMOp, VMOpcode};
    use bumpalo::{Bump, collections::Vec};
    use picodraw_core::{
        Graph,
        graph::{OpAddr, OpInput, OpLiteral, OpValue},
    };
    use std::{collections::HashMap, fmt::Debug, hash::Hash, mem::discriminant};

    #[derive(Debug, Clone, Copy)]
    pub struct IR<'a>(pub &'a VMOp<IR<'a>, ()>);

    impl<'a> IR<'a> {
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

    pub fn emit<'a>(arena: &'a Bump, op: VMOp<IR<'a>, ()>) -> IR<'a> {
        IR(arena.alloc(op))
    }

    pub struct IRBuilder<'a> {
        arena: &'a Bump,
        map: HashMap<(OpAddr, u8), IR<'a>>,
        textures: HashMap<OpAddr, u8>,
    }

    impl<'a> IRBuilder<'a> {
        pub fn new(arena: &'a Bump) -> Self {
            Self {
                arena,
                map: HashMap::new(),
                textures: HashMap::new(),
            }
        }

        pub fn get_graph(&self, op: OpAddr, index: u8) -> IR<'a> {
            *self.map.get(&(op, index)).unwrap()
        }

        pub fn set_graph(&mut self, op: OpAddr, index: u8, value: IR<'a>) {
            self.map.insert((op, index), value);
        }

        pub fn set_texture(&mut self, op: OpAddr, index: u8) {
            self.textures.insert(op, index);
        }

        pub fn get_texture(&self, op: OpAddr) -> u8 {
            *self.textures.get(&op).unwrap()
        }

        pub fn emit_graph(&mut self, graph: &Graph, op: OpAddr, mut input: impl FnMut(OpInput, &mut IRBuilder<'a>)) {
            let ty = graph.type_of(op);

            macro_rules! op {
                ($i:ident => Register ( $a:ident )) => {
                    $a
                };

                ($i:ident => $op:ident( $x:literal )) => {{
                    emit(self.arena, VMOp::$op($x, ()))
                }};

                ($i:ident => $op:ident ( $( $a:ident $(( $($t:tt)* ))? ),* )) => {{
                    emit(self.arena, VMOp::$op(
                        $(
                            op!($i => $a $(( $($t)* ))?),
                        )*
                        ()
                    ))
                }};

                ($i:ident => $a:ident) => {
                    self.get_graph($a, $i as u8)
                };
            }

            macro_rules! out {
                ($($e:tt)*) => {
                    for i in 0..ty.size() {
                        let ir = op!(i => $($e)*);
                        self.set_graph(op, i as u8, ir);
                    }
                };
            }

            use OpValue::*;
            match graph.value_of(op) {
                Input(i) => {
                    input(i, self);
                }

                TextureSample(t, x, filter) => {
                    let texture = self.get_texture(t);
                    let pos_x = self.get_graph(x, 0);
                    let pos_y = self.get_graph(x, 1);
                    let tex_r = emit(self.arena, VMOp::Tex(texture, 2, filter, pos_x, pos_y, ()));
                    let tex_g = emit(self.arena, VMOp::Tex(texture, 1, filter, pos_x, pos_y, ()));
                    let tex_b = emit(self.arena, VMOp::Tex(texture, 0, filter, pos_x, pos_y, ()));
                    let tex_a = emit(self.arena, VMOp::Tex(texture, 3, filter, pos_x, pos_y, ()));
                    self.set_graph(op, 0, tex_r);
                    self.set_graph(op, 1, tex_g);
                    self.set_graph(op, 2, tex_b);
                    self.set_graph(op, 3, tex_a);
                }

                TextureSize(t) => {
                    let texture = self.get_texture(t);
                    let tex_w = emit(self.arena, VMOp::TexW(texture, ()));
                    let tex_h = emit(self.arena, VMOp::TexH(texture, ()));
                    self.set_graph(op, 0, tex_w);
                    self.set_graph(op, 1, tex_h);
                }

                Add(a, b) if ty.is_float() => out!(AddF(a, b)),
                Add(a, b) => out!(AddI(a, b)),
                Sub(a, b) if ty.is_float() => out!(SubF(a, b)),
                Sub(a, b) => out!(SubI(a, b)),
                Mul(a, b) if ty.is_float() => out!(MulF(a, b)),
                Mul(a, b) => out!(MulI(a, b)),
                Div(a, b) if ty.is_float() => out!(DivF(a, b)),
                Div(a, b) => out!(DivI(a, b)),
                Rem(a, b) if ty.is_float() => out!(ModF(a, b)),
                Rem(a, b) => out!(ModI(a, b)),
                Max(a, b) if ty.is_float() => out!(MaxF(a, b)),
                Max(a, b) => out!(MaxI(a, b)),
                Min(a, b) if ty.is_float() => out!(MinF(a, b)),
                Min(a, b) => out!(MinI(a, b)),
                Abs(a) if ty.is_float() => out!(AbsF(a)),
                Abs(a) => out!(AbsI(a)),
                Floor(a) => out!(FloorF(a)),
                Clamp(a, b, c) if ty.is_float() => out!(MaxF(b, MinF(a, c))),
                Clamp(a, b, c) => out!(MaxI(b, MinI(a, c))),

                Lerp(a, b, c) => out!(AddF(b, MulF(a, SubF(c, b)))),

                Smoothstep(a, b, c) => {
                    for i in 0..ty.size() {
                        let t = op!(i => DivF(SubF(a, b), SubF(c, b)));
                        let t = op!(i => MaxF(LitF(0.0), MinF(LitF(1.0), Register(t))));
                        let t = op!(i => MulF(Register(t), MulF(Register(t), SubF(LitF(3.0), MulF(LitF(2.0), Register(t))))));
                        self.set_graph(op, i as u8, t);
                    }
                }

                Step(a, b) if ty.is_float() => out!(Select(LtF(a, b), LitF(0.0), LitF(1.0))),
                Step(a, b) => out!(Select(LtI(a, b), LitI(0), LitI(1))),
                Neg(a) if ty.is_float() => out!(NegF(a)),
                Neg(a) => out!(NegI(a)),
                Sin(a) => out!(SinF(a)),
                Cos(a) => out!(CosF(a)),
                Tan(a) => out!(TanF(a)),
                Asin(a) => out!(AsinF(a)),
                Acos(a) => out!(AcosF(a)),
                Atan(a) => out!(AtanF(a)),
                Atan2(a, b) => out!(Atan2F(a, b)),
                Pow(a, b) => out!(PowF(a, b)),
                Sqrt(a) => out!(SqrtF(a)),
                Exp(a) => out!(ExpF(a)),
                Ln(a) => out!(LnF(a)),
                And(a, b) => out!(AndI(a, b)),
                Or(a, b) => out!(OrI(a, b)),
                Xor(a, b) => out!(XorI(a, b)),
                Shl(a, b) => out!(ShlI(a, b)),
                Shr(a, b) => out!(ShrI(a, b)),
                Not(a) => out!(NotI(a)),
                Eq(a, b) if graph.type_of(a).is_float() => out!(EqF(a, b)),
                Eq(a, b) => out!(EqI(a, b)),
                Ne(a, b) if graph.type_of(a).is_float() => out!(NotI(EqF(a, b))),
                Ne(a, b) => out!(NotI(EqI(a, b))),
                Lt(a, b) if graph.type_of(a).is_float() => out!(LtF(a, b)),
                Lt(a, b) => out!(LtI(a, b)),
                Le(a, b) if graph.type_of(a).is_float() => out!(NotI(GtF(a, b))),
                Le(a, b) => out!(NotI(GtI(a, b))),
                Gt(a, b) if graph.type_of(a).is_float() => out!(GtF(a, b)),
                Gt(a, b) => out!(GtI(a, b)),
                Ge(a, b) if graph.type_of(a).is_float() => out!(NotI(LtF(a, b))),
                Ge(a, b) => out!(NotI(LtI(a, b))),
                CastFloat(a) => out!(CastF(a)),
                CastInt(a) => out!(CastI(a)),

                Sign(a) => {
                    out!(Select(LtF(a, LitF(0.0)), LitF(-1.0), LitF(1.0)));
                }
                Normalize(a) if ty.size() == 1 => {
                    out!(Select(LtF(a, LitF(0.0)), LitF(-1.0), LitF(1.0)));
                }

                Length(a) if graph.type_of(a).size() == 1 => {
                    out!(AbsF(a));
                }

                DerivX(a) => out!(DxF(a)),
                DerivY(a) => out!(DyF(a)),
                DerivWidth(a) => out!(AddF(AbsF(DxF(a)), AbsF(DyF(a)))),

                Position => {
                    let pos_x = emit(self.arena, VMOp::PosX(()));
                    let pos_y = emit(self.arena, VMOp::PosY(()));
                    self.set_graph(op, 0, pos_x);
                    self.set_graph(op, 1, pos_y);
                }

                Resolution => {
                    let res_x = emit(self.arena, VMOp::ResX(()));
                    let res_y = emit(self.arena, VMOp::ResY(()));
                    self.set_graph(op, 0, res_x);
                    self.set_graph(op, 1, res_y);
                }

                QuadStart => {
                    let quad_t = emit(self.arena, VMOp::QuadT(()));
                    let quad_l = emit(self.arena, VMOp::QuadL(()));
                    self.set_graph(op, 0, quad_t);
                    self.set_graph(op, 1, quad_l);
                }

                QuadEnd => {
                    let quad_b = emit(self.arena, VMOp::QuadB(()));
                    let quad_r = emit(self.arena, VMOp::QuadR(()));
                    self.set_graph(op, 0, quad_b);
                    self.set_graph(op, 1, quad_r);
                }

                Literal(OpLiteral::Int(x)) => {
                    let ir = emit(self.arena, VMOp::LitI(x, ()));
                    self.set_graph(op, 0, ir);
                }

                Literal(OpLiteral::Float(x)) => {
                    let ir = emit(self.arena, VMOp::LitF(x, ()));
                    self.set_graph(op, 0, ir);
                }

                Literal(OpLiteral::Bool(x)) => {
                    let ir = emit(self.arena, VMOp::LitI(x as i32, ()));
                    self.set_graph(op, 0, ir);
                }

                Vec2(a, b) => {
                    self.set_graph(op, 0, self.get_graph(a, 0));
                    self.set_graph(op, 1, self.get_graph(b, 0));
                }
                Vec3(a, b, c) => {
                    self.set_graph(op, 0, self.get_graph(a, 0));
                    self.set_graph(op, 1, self.get_graph(b, 0));
                    self.set_graph(op, 2, self.get_graph(c, 0));
                }
                Vec4(a, b, c, d) => {
                    self.set_graph(op, 0, self.get_graph(a, 0));
                    self.set_graph(op, 1, self.get_graph(b, 0));
                    self.set_graph(op, 2, self.get_graph(c, 0));
                    self.set_graph(op, 3, self.get_graph(d, 0));
                }
                Splat2(a) => {
                    self.set_graph(op, 0, self.get_graph(a, 0));
                    self.set_graph(op, 1, self.get_graph(a, 0));
                }
                Splat3(a) => {
                    self.set_graph(op, 0, self.get_graph(a, 0));
                    self.set_graph(op, 1, self.get_graph(a, 0));
                    self.set_graph(op, 2, self.get_graph(a, 0));
                }
                Splat4(a) => {
                    self.set_graph(op, 0, self.get_graph(a, 0));
                    self.set_graph(op, 1, self.get_graph(a, 0));
                    self.set_graph(op, 2, self.get_graph(a, 0));
                    self.set_graph(op, 3, self.get_graph(a, 0));
                }
                ExtractX(a) => {
                    self.set_graph(op, 0, self.get_graph(a, 0));
                }
                ExtractY(a) => {
                    self.set_graph(op, 0, self.get_graph(a, 1));
                }
                ExtractZ(a) => {
                    self.set_graph(op, 0, self.get_graph(a, 2));
                }
                ExtractW(a) => {
                    self.set_graph(op, 0, self.get_graph(a, 3));
                }

                Select(a, b, c) => {
                    let selector = self.get_graph(a, 0);
                    for i in 0..ty.size() {
                        let x = self.get_graph(b, i as u8);
                        let y = self.get_graph(c, i as u8);
                        let ir = emit(self.arena, VMOp::Select(selector, x, y, ()));
                        self.set_graph(op, i as u8, ir);
                    }
                }

                Cross(a, b) => {
                    let ax = self.get_graph(a, 0);
                    let ay = self.get_graph(a, 1);
                    let az = self.get_graph(a, 2);
                    let bx = self.get_graph(b, 0);
                    let by = self.get_graph(b, 1);
                    let bz = self.get_graph(b, 2);

                    let aybz = emit(self.arena, VMOp::MulF(ay, bz, ()));
                    let azby = emit(self.arena, VMOp::MulF(az, by, ()));
                    let azbx = emit(self.arena, VMOp::MulF(az, bx, ()));
                    let axbz = emit(self.arena, VMOp::MulF(ax, bz, ()));
                    let axby = emit(self.arena, VMOp::MulF(ax, by, ()));
                    let aybx = emit(self.arena, VMOp::MulF(ay, bx, ()));

                    let x = emit(self.arena, VMOp::SubF(aybz, azby, ()));
                    let y = emit(self.arena, VMOp::SubF(azbx, axbz, ()));
                    let z = emit(self.arena, VMOp::SubF(axby, aybx, ()));

                    self.set_graph(op, 0, x);
                    self.set_graph(op, 1, y);
                    self.set_graph(op, 2, z);
                }

                Dot(a, b) => {
                    let mut out = None;
                    for i in 0..graph.type_of(a).size() {
                        let a = self.get_graph(a, i as u8);
                        let b = self.get_graph(b, i as u8);
                        let t = emit(self.arena, VMOp::MulF(a, b, ()));

                        out = match out {
                            Some(out) => Some(emit(self.arena, VMOp::AddF(out, t, ()))),
                            None => Some(t),
                        };
                    }
                    self.set_graph(op, 0, out.unwrap());
                }

                Length(a) => {
                    let mut out = None;
                    for i in 0..graph.type_of(a).size() {
                        let a = self.get_graph(a, i as u8);
                        let t = emit(self.arena, VMOp::MulF(a, a, ()));

                        out = match out {
                            Some(out) => Some(emit(self.arena, VMOp::AddF(out, t, ()))),
                            None => Some(t),
                        };
                    }

                    let ir = emit(self.arena, VMOp::SqrtF(out.unwrap(), ()));
                    self.set_graph(op, 0, ir);
                }

                Normalize(a) => {
                    let mut out = None;
                    for i in 0..graph.type_of(a).size() {
                        let a = self.get_graph(a, i as u8);
                        let t = emit(self.arena, VMOp::MulF(a, a, ()));

                        out = match out {
                            Some(out) => Some(emit(self.arena, VMOp::AddF(out, t, ()))),
                            None => Some(t),
                        };
                    }

                    let length = emit(self.arena, VMOp::SqrtF(out.unwrap(), ()));
                    for i in 0..graph.type_of(a).size() {
                        let a = self.get_graph(a, i as u8);
                        let ir = emit(self.arena, VMOp::DivF(a, length, ()));
                        self.set_graph(op, i as u8, ir);
                    }
                }
            }
        }

        pub fn emit_graph_all(&mut self, graph: &Graph, mut input: impl FnMut(OpInput, OpAddr, &mut IRBuilder<'a>)) {
            for op in graph.iter() {
                self.emit_graph(graph, op, |inp, builder| input(inp, op, builder));
            }
        }
    }

    #[derive(Debug)]
    pub struct IRProgram<'a> {
        pub outputs: &'a [IR<'a>],
    }

    pub struct VMProgram<'a> {
        pub opcodes: Vec<'a, VMOpcode>,
        pub outputs: Vec<'a, u8>,
        pub register_count: u8,
    }

    impl<'a> IRProgram<'a> {
        pub fn visit_ops<T>(
            &self,
            arena: &'a Bump,
            mut state: T,
            mut enter: impl FnMut(&mut T, IR<'a>, Option<IR<'a>>) -> bool,
            mut exit: impl FnMut(&mut T, IR<'a>),
        ) {
            enum Visit<'a> {
                Enter(IR<'a>, Option<IR<'a>>),
                Exit(IR<'a>),
            }

            let mut stack = Vec::new_in(arena);

            for ir in self.outputs {
                stack.push(Visit::Enter(*ir, None));
            }

            loop {
                match stack.pop() {
                    Some(Visit::Enter(ir, from)) => {
                        if enter(&mut state, ir, from) {
                            stack.push(Visit::Exit(ir));
                            ir.visit_children(|x| stack.push(Visit::Enter(x, Some(ir))));
                        }
                    }

                    Some(Visit::Exit(ir)) => {
                        exit(&mut state, ir);
                    }

                    None => break,
                }
            }
        }

        /// simple peephole optimizations and constant folding
        pub fn optimize_peephole(self, arena: &'a Bump) -> Self {
            fn single_peephole<'a>(arena: &'a Bump, ir: IR<'a>) -> IR<'a> {
                use VMOp::*;
                match *ir.0 {
                    AddF(a, b, _) => match (a.0, b.0) {
                        (LitF(x, _), LitF(y, _)) => emit(arena, LitF(x + y, ())),
                        (LitF(0.0, _), _) => b,
                        (_, LitF(0.0, _)) => a,
                        (LitF(x, _), b) => emit(arena, AddCF(*x, IR(b), ())),
                        (a, LitF(y, _)) => emit(arena, AddCF(*y, IR(a), ())),
                        (AddF(x, y, _), z) => emit(arena, Add3F(*x, *y, IR(z), ())),
                        (x, AddF(y, z, _)) => emit(arena, Add3F(IR(x), *y, *z, ())),
                        _ => ir,
                    },

                    AddI(a, b, _) => match (a.0, b.0) {
                        (LitI(x, _), LitI(y, _)) => emit(arena, LitI(x.wrapping_add(*y), ())),
                        (LitI(0, _), _) => b,
                        (_, LitI(0, _)) => a,
                        (LitI(x, _), b) => emit(arena, AddCI(*x, IR(b), ())),
                        (a, LitI(y, _)) => emit(arena, AddCI(*y, IR(a), ())),
                        (AddI(x, y, _), z) => emit(arena, Add3I(*x, *y, IR(z), ())),
                        (x, AddI(y, z, _)) => emit(arena, Add3I(IR(x), *y, *z, ())),
                        _ => ir,
                    },

                    SubF(a, b, _) => match (a.0, b.0) {
                        (LitF(x, _), LitF(y, _)) => emit(arena, LitF(x - y, ())),
                        (LitF(0.0, _), _) => emit(arena, NegF(b, ())),
                        (_, LitF(0.0, _)) => a,
                        (LitF(x, _), b) => emit(arena, SubCF(*x, IR(b), ())),
                        (a, LitF(y, _)) => emit(arena, AddCF(-y, IR(a), ())),
                        _ => ir,
                    },

                    SubI(a, b, _) => match (a.0, b.0) {
                        (LitI(x, _), LitI(y, _)) => emit(arena, LitI(x.wrapping_sub(-y), ())),
                        (LitI(0, _), _) => emit(arena, NegI(b, ())),
                        (_, LitI(0, _)) => a,
                        (LitI(x, _), b) => emit(arena, SubCI(*x, IR(b), ())),
                        (a, LitI(y, _)) => emit(arena, AddCI(-y, IR(a), ())),
                        _ => ir,
                    },

                    MulF(a, b, _) => match (a.0, b.0) {
                        (LitF(x, _), LitF(y, _)) => emit(arena, LitF(x * y, ())),
                        (LitF(0.0, _), _) | (_, LitF(0.0, _)) => emit(arena, LitF(0.0, ())),
                        (LitF(1.0, _), _) => b,
                        (_, LitF(1.0, _)) => a,
                        (LitF(x, _), b) => emit(arena, MulCF(*x, IR(b), ())),
                        (a, LitF(y, _)) => emit(arena, MulCF(*y, IR(a), ())),
                        (MulF(x, y, _), z) => emit(arena, Mul3F(*x, *y, IR(z), ())),
                        (x, MulF(y, z, _)) => emit(arena, Mul3F(IR(x), *y, *z, ())),
                        _ => ir,
                    },

                    MulI(a, b, _) => match (a.0, b.0) {
                        (LitI(x, _), LitI(y, _)) => emit(arena, LitI(x.wrapping_mul(*y), ())),
                        (LitI(0, _), _) | (_, LitI(0, _)) => emit(arena, LitI(0, ())),
                        (LitI(1, _), _) => b,
                        (_, LitI(1, _)) => a,
                        (LitI(x, _), b) => emit(arena, MulCI(*x, IR(b), ())),
                        (a, LitI(y, _)) => emit(arena, MulCI(*y, IR(a), ())),
                        (MulI(x, y, _), z) => emit(arena, Mul3I(*x, *y, IR(z), ())),
                        (x, MulI(y, z, _)) => emit(arena, Mul3I(IR(x), *y, *z, ())),
                        _ => ir,
                    },

                    DivF(a, b, _) => match (a.0, b.0) {
                        (LitF(x, _), LitF(y, _)) => emit(arena, LitF(x / y, ())),
                        (LitF(0.0, _), _) => emit(arena, LitF(0.0, ())),
                        (_, LitF(1.0, _)) => a,
                        (_, LitF(x, _)) => emit(arena, MulCF(x.recip(), a, ())),
                        _ => ir,
                    },

                    DivI(a, b, _) => match (a.0, b.0) {
                        (LitI(x, _), LitI(y, _)) => emit(arena, LitI(x.wrapping_div(*y), ())),
                        (LitI(0, _), _) => emit(arena, LitI(0, ())),
                        (_, LitI(1, _)) => a,
                        _ => ir,
                    },

                    NegF(a, _) => match a.0 {
                        LitF(x, _) => emit(arena, LitF(-x, ())),
                        NegF(b, _) => *b,
                        SubF(a, b, _) => emit(arena, SubF(*b, *a, ())),
                        _ => ir,
                    },

                    NegI(a, _) => match a.0 {
                        LitI(x, _) => emit(arena, LitI(x.wrapping_neg(), ())),
                        NegI(b, _) => *b,
                        SubI(a, b, _) => emit(arena, SubI(*b, *a, ())),
                        _ => ir,
                    },

                    MinF(a, b, _) => match (a.0, b.0) {
                        (LitF(x, _), LitF(y, _)) => emit(arena, LitF(x.min(*y), ())),
                        (LitF(x, _), b) => emit(arena, MinCF(*x, IR(b), ())),
                        (a, LitF(y, _)) => emit(arena, MinCF(*y, IR(a), ())),
                        _ => ir,
                    },

                    MinI(a, b, _) => match (a.0, b.0) {
                        (LitI(x, _), LitI(y, _)) => emit(arena, LitI((*x).min(*y), ())),
                        (LitI(x, _), b) => emit(arena, MinCI(*x, IR(b), ())),
                        (a, LitI(y, _)) => emit(arena, MinCI(*y, IR(a), ())),
                        _ => ir,
                    },

                    MaxF(a, b, _) => match (a.0, b.0) {
                        (LitF(x, _), LitF(y, _)) => emit(arena, LitF(x.max(*y), ())),
                        (LitF(x, _), b) => emit(arena, MaxCF(*x, IR(b), ())),
                        (a, LitF(y, _)) => emit(arena, MaxCF(*y, IR(a), ())),
                        _ => ir,
                    },

                    MaxI(a, b, _) => match (a.0, b.0) {
                        (LitI(x, _), LitI(y, _)) => emit(arena, LitI((*x).max(*y), ())),
                        (LitI(x, _), b) => emit(arena, MaxCI(*x, IR(b), ())),
                        (a, LitI(y, _)) => emit(arena, MaxCI(*y, IR(a), ())),
                        _ => ir,
                    },

                    PowF(a, b, _) => match (a.0, b.0) {
                        (LitF(x, _), LitF(y, _)) => emit(arena, LitF(x.powf(*y), ())),
                        (LitF(0.0, _), _) => emit(arena, LitF(0.0, ())),
                        (_, LitF(0.0, _)) => emit(arena, LitF(1.0, ())),
                        (_, LitF(1.0, _)) => a,

                        (x, LitF(0.5, _)) => emit(arena, SqrtF(IR(x), ())),
                        (x, LitF(2.0, _)) => emit(arena, MulF(IR(x), IR(x), ())),
                        (x, LitF(3.0, _)) => {
                            let x2 = emit(arena, MulF(IR(x), IR(x), ()));
                            emit(arena, MulF(IR(x), x2, ()))
                        }
                        (x, LitF(4.0, _)) => {
                            let x2 = emit(arena, MulF(IR(x), IR(x), ()));
                            emit(arena, MulF(x2, x2, ()))
                        }

                        _ => ir,
                    },

                    Select(cond, a, b, _) => match (cond.0, a.0, b.0) {
                        (LitI(0, _), _, _) => b,
                        (LitI(-1, _), _, _) => a,
                        (NotI(c, _), _, _) => emit(arena, Select(*c, b, a, ())),
                        _ => ir,
                    },

                    _ => ir,
                }
            }

            let mut mapping = HashMap::new();
            self.visit_ops(
                arena,
                &mut mapping,
                |mapping, ir, _| !mapping.contains_key(&ir),
                |mapping, ir| {
                    mapping.insert(ir, single_peephole(arena, ir.map_children(arena, |ir| mapping[&ir])));
                },
            );

            Self {
                outputs: arena.alloc_slice_fill_iter(self.outputs.iter().map(|ir| mapping[ir])),
            }
        }

        /// common subexpression elimination using hashconsing
        pub fn optimize_hashcons(self, arena: &'a Bump) -> Self {
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

                        ReadF(x, _) | ReadI(x, _) => x.hash(state),
                        LitF(x, _) => x.to_bits().hash(state),
                        LitI(x, _) => x.hash(state),

                        AddCF(x, b, _) | MulCF(x, b, _) | MinCF(x, b, _) | MaxCF(x, b, _) => {
                            x.to_bits().hash(state);
                            std::ptr::hash(b.0, state)
                        }

                        AddCI(x, b, _) | MulCI(x, b, _) | MinCI(x, b, _) | MaxCI(x, b, _) => {
                            x.hash(state);
                            std::ptr::hash(b.0, state)
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

            let mut forward = HashMap::new();
            let mut reverse = HashMap::new();

            self.visit_ops(
                arena,
                (&mut forward, &mut reverse),
                |(_, reverse), ir, _| !reverse.contains_key(&ir),
                |(forward, reverse), ir| {
                    let key = IRKey(ir.0.map_inputs(|input| reverse[&input]));
                    let normalized = *forward.entry(key).or_insert(ir.map_children(arena, |ir| reverse[&ir]));
                    reverse.insert(ir, normalized);
                },
            );

            Self {
                outputs: arena.alloc_slice_fill_iter(self.outputs.iter().map(|ir| reverse[ir])),
            }
        }

        /// register allocation and lowering to vm ops
        pub fn lower_to_opcodes(&self, arena: &'a Bump) -> VMProgram<'a> {
            // collect ops in dfs post order and collected output edge counts
            let mut ops = Vec::new_in(arena);
            let mut edges = HashMap::new();

            self.visit_ops(
                arena,
                (),
                |_, ir, _| {
                    let edges = edges.entry(ir).or_insert(0);
                    *edges += 1;
                    *edges == 1
                },
                |_, ir| {
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

            for output in self.outputs.iter().copied() {
                outputs.push(registers[&output]);
            }

            VMProgram {
                opcodes,
                outputs,
                register_count: state.len() as u8,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        util::dispatch_simd,
        vm::{CompiledShader, VMContext, VMInterpreter, VMSlot},
    };
    use bumpalo::Bump;
    use picodraw_core::{
        Graph,
        shader::{float2, float4, io},
    };
    use std::hint::black_box;

    #[test]
    fn test() {
        let graph = Graph::collect(|| {
            let z = io::read::<f32>();
            let p = io::position() / io::resolution();
            let d = p - float2((0.5, 0.5));
            let d = d.len();

            float4((d, d, d * z, 1.0))
        });

        let mut arena = Bump::new();
        let shader = CompiledShader::compile(&arena, &graph);
        arena.reset();

        let mut interpreter = VMInterpreter::new(&arena);

        const ITERS: usize = 10000000;
        let start = std::time::Instant::now();
        for i in 0..ITERS {
            let program = VMContext {
                ops: &shader.opcodes,
                data: &[VMSlot {
                    float: 0.0 + i as f32 * 0.01,
                }],
                textures: &[],
                pos_x: 0.0,
                pos_y: 0.0,
                quad_t: 0.0,
                quad_l: 0.0,
                quad_b: 0.0,
                quad_r: 0.0,
                res_x: 32.0,
                res_y: 32.0,
            };

            dispatch_simd(
                #[inline(always)]
                || unsafe {
                    interpreter.execute(program);
                },
            );

            black_box(interpreter.register(0).as_f32());
            black_box(interpreter.register(1).as_f32());
            black_box(interpreter.register(2).as_f32());
        }
        println!("{:?}", start.elapsed() / ITERS as u32);
    }
}
