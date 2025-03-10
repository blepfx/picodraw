use super::interpreter::{REGISTER_COUNT, VMOpcode};
use fxhash::FxHashMap;
use picodraw_core::{
    Graph,
    graph::{OpAddr, OpInput, OpLiteral, OpValue},
};

#[derive(Debug)]
pub struct TranspiledShader {
    pub ops: Vec<VMOpcode>,
    pub output_registers: [u8; 4],
    pub data_slots: u32,
}

/// transpile `picodraw` graph operations into a sequence of VM opcodes
pub fn transpile(graph: &Graph) -> TranspiledShader {
    let mut allocator = RegisterAllocator::new();
    let mut result = Vec::new();
    let mut data = 0;

    for op in graph.iter() {
        let ty = graph.type_of(op);
        let end = match graph.dependents_of(op).max() {
            Some(end) => end,
            None if matches!(graph.value_of(op), OpValue::Output(_)) => op,
            None => continue,
        };

        // this cursed macro saves hundreds of lines of code
        macro_rules! op_helper {
            ($i:ident, $d:expr => $op:ident( $x:literal )) => {
                {
                    let register = allocator.allocate(op, if $d == 0 { end } else { op }, $i + $d * 128).unwrap();
                    result.push(VMOpcode::$op($x, register));
                    register
                }
            };

            ($i:ident, $d:expr => $op:ident ( $( $a:ident $(( $($t:tt)* ))? ),* )) => {
                {
                    let register = allocator.allocate(op, if $d == 0 { end } else { op }, $i + $d * 128).unwrap();
                    let opcode = VMOpcode::$op(
                        $(
                            op_helper!($i, $d + 1 => $a $(( $($t)* ))?),
                        )*
                        register
                    );
                    result.push(opcode);
                    register
                }
            };

            ($i:ident, $d:expr => $a:ident) => {
                allocator.request($a, $i).unwrap()
            };
        }

        macro_rules! op {
            ($($e:tt)*) => {
                for i in 0..ty.size() {
                    op_helper!(i, 0 => $($e)*);
                }
            };
        }

        macro_rules! op_dot {
            ($a:ident, $b:ident, $output:expr) => {{
                let len = op.ty_args[0].size();
                let reg = $output;

                for i in 0..len {
                    let reg_a = allocator.request($a, i).unwrap();
                    let reg_b = allocator.request($b, i).unwrap();
                    let reg_t = allocator.allocate(op, op, i + 256).unwrap();
                    result.push(VMOpcode::MulF(reg_a, reg_b, reg_t));
                }

                match len {
                    2 => {
                        let reg_a = allocator.request(op, 0 + 128).unwrap();
                        let reg_b = allocator.request(op, 1 + 128).unwrap();
                        result.push(VMOpcode::AddF(reg_a, reg_b, reg));
                    }

                    3 => {
                        let reg_a = allocator.request(op, 0 + 128).unwrap();
                        let reg_b = allocator.request(op, 1 + 128).unwrap();
                        let reg_c = allocator.request(op, 2 + 128).unwrap();
                        result.push(VMOpcode::Add3F(reg_a, reg_b, reg_c, reg));
                    }

                    4 => {
                        let reg_a = allocator.request(op, 0 + 128).unwrap();
                        let reg_b = allocator.request(op, 1 + 128).unwrap();
                        let reg_c = allocator.request(op, 2 + 128).unwrap();
                        let reg_d = allocator.request(op, 3 + 128).unwrap();
                        result.push(VMOpcode::Add4F(reg_a, reg_b, reg_c, reg_d, reg));
                    }

                    _ => unreachable!(),
                }
            }};
        }

        match graph.value_of(op) {
            OpValue::Output(a) => {
                let reg_r = allocator.request(a, 0).unwrap();
                let reg_g = allocator.request(a, 1).unwrap();
                let reg_b = allocator.request(a, 2).unwrap();
                let reg_a = allocator.request(a, 3).unwrap();
                allocator.output(0, reg_r);
                allocator.output(1, reg_g);
                allocator.output(2, reg_b);
                allocator.output(3, reg_a);
            }

            OpValue::Input(OpInput::F32) => {
                let reg = allocator.allocate(op, end, 0).unwrap();
                result.push(VMOpcode::ReadF(data, reg));
                data += 1;
            }

            OpValue::Input(OpInput::I32)
            | OpValue::Input(OpInput::I16)
            | OpValue::Input(OpInput::I8)
            | OpValue::Input(OpInput::U32)
            | OpValue::Input(OpInput::U16)
            | OpValue::Input(OpInput::U8) => {
                let reg = allocator.allocate(op, end, 0).unwrap();
                result.push(VMOpcode::ReadI(data, reg));
                data += 1;
            }

            OpValue::Position => {
                let reg_a = allocator.allocate(op, end, 0).unwrap();
                let reg_b = allocator.allocate(op, end, 1).unwrap();
                result.push(VMOpcode::PosX(reg_a));
                result.push(VMOpcode::PosY(reg_b));
            }

            OpValue::Resolution => {
                let reg_a = allocator.allocate(op, end, 0).unwrap();
                let reg_b = allocator.allocate(op, end, 1).unwrap();
                result.push(VMOpcode::ResX(reg_a));
                result.push(VMOpcode::ResY(reg_b));
            }

            OpValue::QuadStart => {
                let reg_a = allocator.allocate(op, end, 0).unwrap();
                let reg_b = allocator.allocate(op, end, 1).unwrap();
                result.push(VMOpcode::QuadL(reg_a));
                result.push(VMOpcode::QuadT(reg_b));
            }

            OpValue::QuadEnd => {
                let reg_a = allocator.allocate(op, end, 0).unwrap();
                let reg_b = allocator.allocate(op, end, 1).unwrap();
                result.push(VMOpcode::QuadR(reg_a));
                result.push(VMOpcode::QuadB(reg_b));
            }

            OpValue::Literal(OpLiteral::Int(x)) => {
                let reg = allocator.allocate(op, end, 0).unwrap();
                result.push(VMOpcode::LitI(x, reg));
            }

            OpValue::Literal(OpLiteral::Float(x)) => {
                let reg = allocator.allocate(op, end, 0).unwrap();
                result.push(VMOpcode::LitF(f32::from(x), reg));
            }

            OpValue::Literal(OpLiteral::Bool(x)) => {
                let reg = allocator.allocate(op, end, 0).unwrap();
                result.push(VMOpcode::LitI(if x { -1 } else { 0 }, reg));
            }

            OpValue::Add(a, b) if ty.is_float() => op!(AddF(a, b)),
            OpValue::Add(a, b) => op!(AddI(a, b)),
            OpValue::Sub(a, b) if ty.is_float() => op!(SubF(a, b)),
            OpValue::Sub(a, b) => op!(SubI(a, b)),
            OpValue::Mul(a, b) if ty.is_float() => op!(MulF(a, b)),
            OpValue::Mul(a, b) => op!(MulI(a, b)),
            OpValue::Div(a, b) if ty.is_float() => op!(DivF(a, b)),
            OpValue::Div(a, b) => op!(DivI(a, b)),
            OpValue::Rem(a, b) if ty.is_float() => op!(ModF(a, b)),
            OpValue::Rem(a, b) => op!(ModI(a, b)),
            OpValue::Max(a, b) if ty.is_float() => op!(MaxF(a, b)),
            OpValue::Max(a, b) => op!(MaxI(a, b)),
            OpValue::Min(a, b) if ty.is_float() => op!(MinF(a, b)),
            OpValue::Min(a, b) => op!(MinI(a, b)),
            OpValue::Abs(a) if ty.is_float() => op!(AbsF(a)),
            OpValue::Abs(a) => op!(AbsI(a)),
            OpValue::Floor(a) => op!(FloorF(a)),
            OpValue::Clamp(a, b, c) if ty.is_float() => op!(MaxF(b, MinF(a, c))),
            OpValue::Clamp(a, b, c) => op!(MaxI(b, MinI(a, c))),
            OpValue::Lerp(a, b, c) => op!(LerpF(a, b, c)),
            OpValue::Smoothstep(a, b, c) => op!(SmoothstepF(a, b, c)),

            OpValue::Step(a, b) if ty.is_float() => op!(Select(LtF(b, a), LitF(0.0), LitF(1.0))),
            OpValue::Step(a, b) => op!(Select(LtI(b, a), LitI(0), LitI(1))),

            OpValue::Neg(a) if ty.is_float() => op!(NegF(a)),
            OpValue::Neg(a) => op!(NegI(a)),
            OpValue::Sin(a) => op!(SinF(a)),
            OpValue::Cos(a) => op!(CosF(a)),
            OpValue::Tan(a) => op!(TanF(a)),
            OpValue::Asin(a) => op!(AsinF(a)),
            OpValue::Acos(a) => op!(AcosF(a)),
            OpValue::Atan(a) => op!(AtanF(a)),
            OpValue::Atan2(a, b) => op!(Atan2F(a, b)),
            OpValue::Pow(a, b) => op!(PowF(a, b)),
            OpValue::Sqrt(a) => op!(SqrtF(a)),
            OpValue::Exp(a) => op!(ExpF(a)),
            OpValue::Ln(a) => op!(LnF(a)),

            OpValue::And(a, b) => op!(AndI(a, b)),
            OpValue::Or(a, b) => op!(OrI(a, b)),
            OpValue::Xor(a, b) => op!(XorI(a, b)),
            OpValue::Not(a) => op!(NotI(a)),

            OpValue::Eq(a, b) if ty.is_float() => op!(EqF(a, b)),
            OpValue::Eq(a, b) => op!(EqI(a, b)),
            OpValue::Ne(a, b) if ty.is_float() => op!(NeF(a, b)),
            OpValue::Ne(a, b) => op!(NeI(a, b)),
            OpValue::Lt(a, b) if ty.is_float() => op!(LtF(a, b)),
            OpValue::Lt(a, b) => op!(LtI(a, b)),
            OpValue::Le(a, b) if ty.is_float() => op!(LeF(a, b)),
            OpValue::Le(a, b) => op!(LeI(a, b)),
            OpValue::Gt(a, b) if ty.is_float() => op!(GtF(a, b)),
            OpValue::Gt(a, b) => op!(GtI(a, b)),
            OpValue::Ge(a, b) if ty.is_float() => op!(GeF(a, b)),
            OpValue::Ge(a, b) => op!(GeI(a, b)),

            OpValue::CastFloat(a) => op!(CastF(a)),
            OpValue::CastInt(a) => op!(CastI(a)),

            OpValue::Dot(a, b) if graph.type_of(a).size() == 1 => {
                op!(MulF(a, b));
            }

            OpValue::Sign(a) => {
                op!(Select(LtF(a, LitF(0.0)), LitF(-1.0), LitF(1.0)));
            }

            OpValue::Normalize(a) if ty.size() == 1 => {
                op!(Select(LtF(a, LitF(0.0)), LitF(-1.0), LitF(1.0)));
            }

            OpValue::Length(a) if graph.type_of(a).size() == 1 => {
                op!(AbsF(a));
            }

            OpValue::Select(a, b, c) => {
                op!(Select(a, b, c));
            }

            OpValue::DerivX(a) => op!(DxF(a)),
            OpValue::DerivY(a) => op!(DyF(a)),
            OpValue::DerivWidth(a) => op!(AddF(AbsF(DxF(a)), AbsF(DyF(a)))),

            OpValue::Vec2(a, b) => {
                let reg_a = allocator.request(a, 0).unwrap();
                let reg_b = allocator.request(b, 0).unwrap();
                allocator.reuse(op, end, 0, reg_a);
                allocator.reuse(op, end, 1, reg_b);
            }

            OpValue::Vec3(a, b, c) => {
                let reg_a = allocator.request(a, 0).unwrap();
                let reg_b = allocator.request(b, 0).unwrap();
                let reg_c = allocator.request(c, 0).unwrap();
                allocator.reuse(op, end, 0, reg_a);
                allocator.reuse(op, end, 1, reg_b);
                allocator.reuse(op, end, 2, reg_c);
            }

            OpValue::Vec4(a, b, c, d) => {
                let reg_a = allocator.request(a, 0).unwrap();
                let reg_b = allocator.request(b, 0).unwrap();
                let reg_c = allocator.request(c, 0).unwrap();
                let reg_d = allocator.request(d, 0).unwrap();
                allocator.reuse(op, end, 0, reg_a);
                allocator.reuse(op, end, 1, reg_b);
                allocator.reuse(op, end, 2, reg_c);
                allocator.reuse(op, end, 3, reg_d);
            }

            OpValue::Splat2(a) => {
                let reg_a = allocator.request(a, 0).unwrap();
                allocator.reuse(op, end, 0, reg_a);
                allocator.reuse(op, end, 1, reg_a);
            }

            OpValue::Splat3(a) => {
                let reg_a = allocator.request(a, 0).unwrap();
                allocator.reuse(op, end, 0, reg_a);
                allocator.reuse(op, end, 1, reg_a);
                allocator.reuse(op, end, 2, reg_a);
            }

            OpValue::Splat4(a) => {
                let reg_a = allocator.request(a, 0).unwrap();
                allocator.reuse(op, end, 0, reg_a);
                allocator.reuse(op, end, 1, reg_a);
                allocator.reuse(op, end, 2, reg_a);
                allocator.reuse(op, end, 3, reg_a);
            }

            OpValue::ExtractX(a) => {
                let reg_a = allocator.request(a, 0).unwrap();
                allocator.reuse(op, end, 0, reg_a);
            }

            OpValue::ExtractY(a) => {
                let reg_a = allocator.request(a, 1).unwrap();
                allocator.reuse(op, end, 0, reg_a);
            }

            OpValue::ExtractZ(a) => {
                let reg_a = allocator.request(a, 2).unwrap();
                allocator.reuse(op, end, 0, reg_a);
            }

            OpValue::ExtractW(a) => {
                let reg_a = allocator.request(a, 3).unwrap();
                allocator.reuse(op, end, 0, reg_a);
            }

            OpValue::Cross(_, _) => {
                todo!()
            }

            OpValue::Dot(a, b) => {
                todo!()
            }

            OpValue::Length(a) => {
                todo!()
            }

            OpValue::Normalize(_) => {
                todo!()
            }

            OpValue::Input(OpInput::TextureRender) | OpValue::Input(OpInput::TextureStatic) => {
                todo!()
            }

            OpValue::TextureSize(_)
            | OpValue::TextureNearest(_, _)
            | OpValue::TextureLinear(_, _) => {
                todo!()
            }
        }
    }

    TranspiledShader {
        ops: result,
        data_slots: data,
        output_registers: [
            allocator.outputs[0].unwrap_or_default(),
            allocator.outputs[1].unwrap_or_default(),
            allocator.outputs[2].unwrap_or_default(),
            allocator.outputs[3].unwrap_or_default(),
        ],
    }
}

struct RegisterAllocator {
    registers: [Option<OpAddr>; REGISTER_COUNT],
    mapping: FxHashMap<(OpAddr, u32), u8>,
    outputs: [Option<u8>; 4],
}

impl RegisterAllocator {
    pub fn new() -> Self {
        Self {
            registers: [None; REGISTER_COUNT],
            mapping: FxHashMap::default(),
            outputs: [None; 4],
        }
    }

    pub fn allocate(&mut self, start: OpAddr, end: OpAddr, index: u32) -> Option<u8> {
        for (i, reg) in self.registers.iter_mut().enumerate() {
            // output registers should be "pinned", so we don't touch them
            if self.outputs.iter().any(|&x| Some(i as u8) == x) {
                continue;
            }

            // if not allocated or the value is dead and not used, allocate
            if reg.is_none() || matches!(*reg, Some(x) if x < start) {
                *reg = Some(end);
                self.mapping.insert((start, index), i as u8);
                return Some(i as u8);
            }
        }

        None
    }

    pub fn allocate_temp(&mut self, op: OpAddr, index: u32, scope: u32) -> u8 {
        self.allocate(op, op, index + scope * 128).unwrap()
    }

    pub fn request(&self, op: OpAddr, index: u32) -> Option<u8> {
        self.mapping.get(&(op, index)).copied()
    }

    pub fn reuse(&mut self, start: OpAddr, end: OpAddr, index: u32, register: u8) -> u8 {
        self.mapping.insert((start, index), register);
        self.registers[register as usize] = Some(
            self.registers[register as usize]
                .map(|x| x.max(end))
                .unwrap_or(end),
        );
        register
    }

    pub fn output(&mut self, index: u32, register: u8) {
        self.outputs[index as usize] = Some(register);
    }
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;

    use super::*;
    use crate::shader::interpreter::{VMInterpreter, VMProgram, VMRegister, VMSlot, VMTile};
    use picodraw_core::shader::{float2, float4, io};

    #[test]
    fn test() {
        let graph = Graph::collect(|| {
            let z = io::read::<f32>();
            let p = io::position() / io::resolution();
            let d = p - float2((0.5, 0.5));
            let d = (d.x() * d.x() + d.y() * d.y()).sqrt();
            io::write_color(float4((d, d, d * z, 1.0)));
        });

        let shader = transpile(&graph);
        let mut interpreter = VMInterpreter::<VMTile>::new();

        const ITERS: usize = 100000;
        let start = std::time::Instant::now();
        for i in 0..ITERS {
            let program = VMProgram {
                ops: &shader.ops,
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

            unsafe {
                interpreter.execute(program);
            }

            black_box(interpreter.register(0).as_f32());
            black_box(interpreter.register(1).as_f32());
            black_box(interpreter.register(2).as_f32());
        }
        println!("{:?}", start.elapsed() / ITERS as u32);
    }
}
