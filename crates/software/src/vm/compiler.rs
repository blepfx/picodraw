use super::{VMIR, VMOp, VMOpcode};
use picodraw_core::{
    Graph,
    graph::{OpAddr, OpInput, OpLiteral, OpValue},
};
use std::collections::HashMap;

struct IRBuilder {
    ops: Vec<VMIR>,
    map: HashMap<(OpAddr, u8), u32>,
}

impl IRBuilder {
    pub fn new() -> Self {
        Self {
            ops: Vec::new(),
            map: HashMap::new(),
        }
    }

    pub fn emit_ir(&mut self, op: VMIR) -> u32 {
        let idx = self.ops.len() as u32;
        self.ops.push(op);
        idx
    }

    pub fn get_graph(&self, op: OpAddr, index: u8) -> u32 {
        *self.map.get(&(op, index)).unwrap()
    }

    pub fn set_graph(&mut self, op: OpAddr, index: u8, value: u32) {
        self.map.insert((op, index), value);
    }

    pub fn emit_graph(&mut self, graph: &Graph, op: OpAddr) {
        let ty = graph.type_of(op);

        macro_rules! op {
            ($i:ident => Register ( $a:ident )) => {
                $a
            };

            ($i:ident => Value ( $a:expr )) => {
                $a
            };

            ($i:ident => $op:ident( $x:literal )) => {{
                let op = VMOp::$op($x, ());
                self.emit_ir(op)
            }};

            ($i:ident => $op:ident ( $( $a:ident $(( $($t:tt)* ))? ),* )) => {{
                let op = VMOp::$op(
                    $(
                        op!($i => $a $(( $($t)* ))?),
                    )*
                    ()
                );
                self.emit_ir(op)
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
            Input(_) => unreachable!(),

            TextureLinear(_, _) => {
                todo!()
            }

            TextureNearest(_, _) => {
                todo!()
            }

            TextureSize(_) => {
                todo!()
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

            Step(a, b) if ty.is_float() => out!(Select(LtF(b, a), LitF(0.0), LitF(1.0))),
            Step(a, b) => out!(Select(LtI(b, a), LitI(0), LitI(1))),
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
            Not(a) => out!(NotI(a)),
            Eq(a, b) if ty.is_float() => out!(EqF(a, b)),
            Eq(a, b) => out!(EqI(a, b)),
            Ne(a, b) if ty.is_float() => out!(NeF(a, b)),
            Ne(a, b) => out!(NeI(a, b)),
            Lt(a, b) if ty.is_float() => out!(LtF(a, b)),
            Lt(a, b) => out!(LtI(a, b)),
            Le(a, b) if ty.is_float() => out!(LeF(a, b)),
            Le(a, b) => out!(LeI(a, b)),
            Gt(a, b) if ty.is_float() => out!(GtF(a, b)),
            Gt(a, b) => out!(GtI(a, b)),
            Ge(a, b) if ty.is_float() => out!(GeF(a, b)),
            Ge(a, b) => out!(GeI(a, b)),
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
            Select(a, b, c) => {
                out!(Select(a, b, c));
            }

            DerivX(a) => out!(DxF(a)),
            DerivY(a) => out!(DyF(a)),
            DerivWidth(a) => out!(AddF(AbsF(DxF(a)), AbsF(DyF(a)))),

            Position => {
                let pos_x = self.emit_ir(VMIR::PosX(()));
                let pos_y = self.emit_ir(VMIR::PosY(()));
                self.set_graph(op, 0, pos_x);
                self.set_graph(op, 1, pos_y);
            }

            Resolution => {
                let res_x = self.emit_ir(VMIR::ResX(()));
                let res_y = self.emit_ir(VMIR::ResY(()));
                self.set_graph(op, 0, res_x);
                self.set_graph(op, 1, res_y);
            }

            QuadStart => {
                let quad_t = self.emit_ir(VMIR::QuadT(()));
                let quad_l = self.emit_ir(VMIR::QuadL(()));
                self.set_graph(op, 0, quad_t);
                self.set_graph(op, 1, quad_l);
            }

            QuadEnd => {
                let quad_b = self.emit_ir(VMIR::QuadB(()));
                let quad_r = self.emit_ir(VMIR::QuadR(()));
                self.set_graph(op, 0, quad_b);
                self.set_graph(op, 1, quad_r);
            }

            Literal(OpLiteral::Int(x)) => {
                let ir = self.emit_ir(VMIR::LitI(x, ()));
                self.set_graph(op, 0, ir);
            }

            Literal(OpLiteral::Float(x)) => {
                let ir = self.emit_ir(VMIR::LitF(x, ()));
                self.set_graph(op, 0, ir);
            }

            Literal(OpLiteral::Bool(x)) => {
                let ir = self.emit_ir(VMIR::LitI(x as i32, ()));
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

            Cross(_, _) => {
                todo!()
            }

            Dot(a, b) => {
                let mut out = None;
                for i in 0..graph.type_of(a).size() {
                    let a = self.get_graph(a, i as u8);
                    let b = self.get_graph(b, i as u8);
                    let t = self.emit_ir(VMIR::MulF(a, b, ()));

                    out = match out {
                        Some(out) => Some(self.emit_ir(VMIR::AddF(out, t, ()))),
                        None => Some(t),
                    };
                }
                self.set_graph(op, 0, out.unwrap());
            }

            Length(a) => {
                let mut out = None;
                for i in 0..graph.type_of(a).size() {
                    let a = self.get_graph(a, i as u8);
                    let t = self.emit_ir(VMIR::MulF(a, a, ()));

                    out = match out {
                        Some(out) => Some(self.emit_ir(VMIR::AddF(out, t, ()))),
                        None => Some(t),
                    };
                }

                let ir = self.emit_ir(VMIR::SqrtF(out.unwrap(), ()));
                self.set_graph(op, 0, ir);
            }

            Normalize(a) => {
                let mut out = None;
                for i in 0..graph.type_of(a).size() {
                    let a = self.get_graph(a, i as u8);
                    let t = self.emit_ir(VMIR::MulF(a, a, ()));

                    out = match out {
                        Some(out) => Some(self.emit_ir(VMIR::AddF(out, t, ()))),
                        None => Some(t),
                    };
                }

                let length = self.emit_ir(VMIR::SqrtF(out.unwrap(), ()));
                for i in 0..graph.type_of(a).size() {
                    let a = self.get_graph(a, i as u8);
                    let ir = self.emit_ir(VMIR::DivF(a, length, ()));
                    self.set_graph(op, i as u8, ir);
                }
            }
        }
    }

    pub fn allocate_registers(&self, outputs: &[u32]) -> Vec<VMOpcode> {
        // step 1: lifetime analysis
        let lifetimes = {
            let mut lifetimes = vec![0; self.ops.len()];
            for (idx, op) in self.ops.iter().enumerate().rev() {
                if outputs.contains(&(idx as u32)) {
                    lifetimes[idx] = self.ops.len() as u32;
                }

                op.map(
                    |input| {
                        lifetimes[input as usize] = lifetimes[input as usize].max(idx as u32);
                    },
                    |_| (),
                );
            }
            lifetimes
        };

        // step 2: register allocation itself
        // simple linear scan, its possible to reorder ops to reduce register pressure
        // but we can do that later at an optimization stage
        let mut registers: Vec<(u32, u32)> = vec![];
        let mut result = vec![];

        for (idx, op) in self.ops.iter().enumerate() {
            result.push(
                op.map(
                    |input| {
                        for (i, (start, end)) in registers.iter().enumerate() {
                            if *start == input && *end >= input {
                                return i as u8;
                            }
                        }
                        panic!("register not found");
                    },
                    |x| x,
                )
                .map(
                    |x| x,
                    |_| {
                        let start = idx as u32;
                        let end = lifetimes[idx];
                        for (i, reg) in registers.iter_mut().enumerate() {
                            if reg.1 < start {
                                *reg = (start, end);
                                return i as u8;
                            }
                        }

                        registers.push((start, end));
                        (registers.len() - 1) as u8
                    },
                ),
            );
        }

        result
    }
}

#[derive(Debug)]
pub struct CompiledShader {
    opcodes: Vec<VMOpcode>,
    output: [u8; 4],
    data_slots: u32,
}

impl CompiledShader {
    pub fn compile(graph: &Graph) -> Self {
        let mut builder = IRBuilder::new();
        let mut data_slots = 0;

        for op in graph.iter() {
            match graph.value_of(op) {
                OpValue::Input(OpInput::F32) => {
                    let ir = builder.emit_ir(VMIR::ReadF(data_slots, ()));
                    builder.set_graph(op, 0, ir);
                    data_slots += 1;
                }

                OpValue::Input(x) if x.value_type().is_int() => {
                    let ir = builder.emit_ir(VMIR::ReadI(data_slots, ()));
                    builder.set_graph(op, 0, ir);
                    data_slots += 1;
                }

                _ => {
                    builder.emit_graph(graph, op);
                }
            }
        }

        let output_r = builder.get_graph(graph.output(), 0);
        let output_g = builder.get_graph(graph.output(), 1);
        let output_b = builder.get_graph(graph.output(), 2);
        let output_a = builder.get_graph(graph.output(), 3);

        let opcodes = builder.allocate_registers(&[output_r, output_g, output_b, output_a]);
        Self {
            output: [
                opcodes[output_r as usize].output() as u8,
                opcodes[output_g as usize].output() as u8,
                opcodes[output_b as usize].output() as u8,
                opcodes[output_a as usize].output() as u8,
            ],
            opcodes,
            data_slots,
        }
    }

    pub fn opcodes(&self) -> &[VMOpcode] {
        &self.opcodes
    }

    pub fn output_register(&self, index: usize) -> u8 {
        self.output[index]
    }

    pub fn data_slots(&self) -> u32 {
        self.data_slots
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        simd::dispatch,
        vm::{CompiledShader, VMInterpreter, VMProgram, VMRegister, VMSlot, VMTile},
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

        let shader = CompiledShader::compile(&graph);
        let arena = Bump::new();
        let mut interpreter = VMInterpreter::<VMTile>::new(&arena);

        const ITERS: usize = 10000000;
        let start = std::time::Instant::now();
        for i in 0..ITERS {
            let program = VMProgram {
                ops: &shader.opcodes,
                data: &[VMSlot {
                    float: 0.0 + i as f32 * 0.01,
                }],
                tile_x: 0.0,
                tile_y: 0.0,
                quad_t: 0.0,
                quad_l: 0.0,
                quad_b: 0.0,
                quad_r: 0.0,
                res_x: 32.0,
                res_y: 32.0,
            };

            dispatch(
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
