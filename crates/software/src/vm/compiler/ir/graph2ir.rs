use super::{IR, IRProgram, VMOp};
use bumpalo::Bump;
use picodraw_core::graph::{Graph, OpAddr, OpInput, OpLiteral, OpValue};
use std::collections::HashMap;

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

    pub fn from_graph(
        arena: &'a Bump,
        graph: &Graph,
        mut callback: impl FnMut(&mut IRBuilder<'a>, OpAddr, OpInput),
    ) -> Self {
        let mut builder = Self::new(arena);

        for op in graph.iter() {
            builder.emit_single(graph, op, |input, builder| {
                callback(builder, op, input);
            });
        }

        builder
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

    pub fn emit_single(&mut self, graph: &Graph, op: OpAddr, mut input: impl FnMut(OpInput, &mut IRBuilder<'a>)) {
        let ty = graph.type_of(op);

        macro_rules! op {
            ($i:ident => Register ( $a:ident )) => {
                $a
            };

            ($i:ident => $op:ident( $x:literal )) => {{
                IR::new(self.arena, VMOp::$op($x, ()))
            }};

            ($i:ident => $op:ident ( $( $a:ident $(( $($t:tt)* ))? ),* )) => {{
                IR::new(self.arena, VMOp::$op(
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
                let tex_r = IR::new(self.arena, VMOp::Tex(texture, 2, filter, pos_x, pos_y, ()));
                let tex_g = IR::new(self.arena, VMOp::Tex(texture, 1, filter, pos_x, pos_y, ()));
                let tex_b = IR::new(self.arena, VMOp::Tex(texture, 0, filter, pos_x, pos_y, ()));
                let tex_a = IR::new(self.arena, VMOp::Tex(texture, 3, filter, pos_x, pos_y, ()));
                self.set_graph(op, 0, tex_r);
                self.set_graph(op, 1, tex_g);
                self.set_graph(op, 2, tex_b);
                self.set_graph(op, 3, tex_a);
            }

            TextureSize(t) => {
                let texture = self.get_texture(t);
                let tex_w = IR::new(self.arena, VMOp::TexW(texture, ()));
                let tex_h = IR::new(self.arena, VMOp::TexH(texture, ()));
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
                    let t =
                        op!(i => MulF(Register(t), MulF(Register(t), SubF(LitF(3.0), MulF(LitF(2.0), Register(t))))));
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
                let pos_x = IR::new(self.arena, VMOp::PosX(()));
                let pos_y = IR::new(self.arena, VMOp::PosY(()));
                self.set_graph(op, 0, pos_x);
                self.set_graph(op, 1, pos_y);
            }

            Resolution => {
                let res_x = IR::new(self.arena, VMOp::ResX(()));
                let res_y = IR::new(self.arena, VMOp::ResY(()));
                self.set_graph(op, 0, res_x);
                self.set_graph(op, 1, res_y);
            }

            QuadStart => {
                let quad_t = IR::new(self.arena, VMOp::QuadT(()));
                let quad_l = IR::new(self.arena, VMOp::QuadL(()));
                self.set_graph(op, 0, quad_t);
                self.set_graph(op, 1, quad_l);
            }

            QuadEnd => {
                let quad_b = IR::new(self.arena, VMOp::QuadB(()));
                let quad_r = IR::new(self.arena, VMOp::QuadR(()));
                self.set_graph(op, 0, quad_b);
                self.set_graph(op, 1, quad_r);
            }

            Literal(OpLiteral::Int(x)) => {
                let ir = IR::new(self.arena, VMOp::LitI(x, ()));
                self.set_graph(op, 0, ir);
            }

            Literal(OpLiteral::Float(x)) => {
                let ir = IR::new(self.arena, VMOp::LitF(x, ()));
                self.set_graph(op, 0, ir);
            }

            Literal(OpLiteral::Bool(x)) => {
                let ir = IR::new(self.arena, VMOp::LitI(x as i32, ()));
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
                    let ir = IR::new(self.arena, VMOp::Select(selector, x, y, ()));
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

                let aybz = IR::new(self.arena, VMOp::MulF(ay, bz, ()));
                let azby = IR::new(self.arena, VMOp::MulF(az, by, ()));
                let azbx = IR::new(self.arena, VMOp::MulF(az, bx, ()));
                let axbz = IR::new(self.arena, VMOp::MulF(ax, bz, ()));
                let axby = IR::new(self.arena, VMOp::MulF(ax, by, ()));
                let aybx = IR::new(self.arena, VMOp::MulF(ay, bx, ()));

                let x = IR::new(self.arena, VMOp::SubF(aybz, azby, ()));
                let y = IR::new(self.arena, VMOp::SubF(azbx, axbz, ()));
                let z = IR::new(self.arena, VMOp::SubF(axby, aybx, ()));

                self.set_graph(op, 0, x);
                self.set_graph(op, 1, y);
                self.set_graph(op, 2, z);
            }

            Dot(a, b) => {
                let mut out = None;
                for i in 0..graph.type_of(a).size() {
                    let a = self.get_graph(a, i as u8);
                    let b = self.get_graph(b, i as u8);
                    let t = IR::new(self.arena, VMOp::MulF(a, b, ()));

                    out = match out {
                        Some(out) => Some(IR::new(self.arena, VMOp::AddF(out, t, ()))),
                        None => Some(t),
                    };
                }
                self.set_graph(op, 0, out.unwrap());
            }

            Length(a) => {
                let mut out = None;
                for i in 0..graph.type_of(a).size() {
                    let a = self.get_graph(a, i as u8);
                    let t = IR::new(self.arena, VMOp::MulF(a, a, ()));

                    out = match out {
                        Some(out) => Some(IR::new(self.arena, VMOp::AddF(out, t, ()))),
                        None => Some(t),
                    };
                }

                let ir = IR::new(self.arena, VMOp::SqrtF(out.unwrap(), ()));
                self.set_graph(op, 0, ir);
            }

            Normalize(a) => {
                let mut out = None;
                for i in 0..graph.type_of(a).size() {
                    let a = self.get_graph(a, i as u8);
                    let t = IR::new(self.arena, VMOp::MulF(a, a, ()));

                    out = match out {
                        Some(out) => Some(IR::new(self.arena, VMOp::AddF(out, t, ()))),
                        None => Some(t),
                    };
                }

                let length = IR::new(self.arena, VMOp::SqrtF(out.unwrap(), ()));
                for i in 0..graph.type_of(a).size() {
                    let a = self.get_graph(a, i as u8);
                    let ir = IR::new(self.arena, VMOp::DivF(a, length, ()));
                    self.set_graph(op, i as u8, ir);
                }
            }
        }
    }

    pub fn extract_program(&self, output: OpAddr, output_n: u8) -> IRProgram<'a> {
        IRProgram {
            outputs: self
                .arena
                .alloc_slice_fill_iter((0..output_n).map(|i| self.get_graph(output, i as u8))),
        }
    }
}
