use super::{VMOp, VMSlot, VMTile};
use crate::{
    BufferRef,
    util::Pod,
    vm::{CompiledProgram, VMTile16},
};
use bumpalo::{Bump, boxed::Box};

#[derive(Clone, Copy)]
pub struct VMContext<'a> {
    pub program: CompiledProgram<'a>,
    pub textures: &'a [BufferRef<'a>],
    pub inputs: &'a [VMSlot],

    pub pos_x: f32,
    pub pos_y: f32,
    pub quad_t: f32,
    pub quad_l: f32,
    pub quad_b: f32,
    pub quad_r: f32,
    pub res_x: f32,
    pub res_y: f32,
}

pub struct VMResult<'a, T: VMTile> {
    registers: &'a [T],
    outputs: &'a [u8],
}

pub struct VMMemory<'a> {
    memory: Box<'a, [VMTile16]>,
}

impl<'a> VMMemory<'a> {
    pub fn new(slots: usize, arena: &'a Bump) -> Self {
        Self {
            memory: Box::from_iter_in((0..slots.div_ceil(16 * 16)).map(|_| VMTile16::zeroed()), arena),
        }
    }

    pub fn slots<T: VMTile>(&self) -> usize {
        VMTile16::cast_slice::<T>(&self.memory).len()
    }
}

impl<'a> VMContext<'a> {
    /// SAFETY: caller must ensure that the program context is valid:
    /// - the `textures` array have at least the amount of elements that `Tex*` opcodes reference
    /// - every operation references a register that is less than the size of the `memory` argument
    /// - every operation writes to a register it doesn't read from (`AddF(0, 1, 1)` is NOT valid)
    /// - every operation reads only from those registers that were written to by preceding operations
    #[allow(unused_unsafe)]
    #[inline(always)]
    pub unsafe fn run<T: VMTile>(&self, memory: &'a mut VMMemory<'_>) -> VMResult<'a, T> {
        use VMOp::*;

        let registers: &mut [T] = VMTile16::cast_slice_mut(&mut memory.memory[..]);
        debug_assert!(self.program.used_registers() <= registers.len());

        macro_rules! registers {
            ($($input:expr,)* mut $output:expr) => {
                unsafe {
                    let ptr = registers.as_mut_ptr();
                    ($(&*ptr.add($input as usize),)* &mut *ptr.add($output as usize),)
                }
            };
        }

        macro_rules! op {
            (|$a:ident: f32, $b:ident: mut f32| $x:expr) => {{
                let (a, b) = registers!($a, mut $b);
                for (&$a, b) in a.as_f32().iter().zip(b.as_f32_mut()) {
                    *b = $x;
                }
            }};

            (|$a:ident: f32, $b:ident: mut i32| $x:expr) => {{
                let (a, b) = registers!($a, mut $b);
                for (&$a, b) in a.as_f32().iter().zip(b.as_i32_mut()) {
                    *b = $x;
                }
            }};

            (|$a:ident: i32, $b:ident: mut i32| $x:expr) => {{
                let (a, b) = registers!($a, mut $b);
                for (&$a, b) in a.as_i32().iter().zip(b.as_i32_mut()) {
                    *b = $x;
                }
            }};

            (|$a:ident: i32, $b:ident: mut f32| $x:expr) => {{
                let (a, b) = registers!($a, mut $b);
                for (&$a, b) in a.as_i32().iter().zip(b.as_f32_mut()) {
                    *b = $x;
                }
            }};

            (|$a:ident: f32, $b:ident: f32, $c:ident: mut f32| $x:expr) => {{
                let (a, b, c) = registers!($a, $b, mut $c);
                for ((&$a, &$b), c) in a.as_f32().iter().zip(b.as_f32()).zip(c.as_f32_mut()) {
                    *c = $x;
                }
            }};

            (|$a:ident: i32, $b:ident: i32, $c:ident: mut i32| $x:expr) => {{
                let (a, b, c) = registers!($a, $b, mut $c);
                for ((&$a, &$b), c) in a.as_i32().iter().zip(b.as_i32()).zip(c.as_i32_mut()) {
                    *c = $x;
                }
            }};

            (|$a:ident: f32, $b:ident: f32, $c:ident: mut i32| $x:expr) => {{
                let (a, b, c) = registers!($a, $b, mut $c);
                for ((&$a, &$b), c) in a.as_f32().iter().zip(b.as_f32()).zip(c.as_i32_mut()) {
                    *c = $x;
                }
            }};

            (|$a:ident: f32, $b:ident: f32, $c:ident: f32, $d:ident: mut f32| $x:expr) => {{
                let (a, b, c, d) = registers!($a, $b, $c, mut $d);
                for (((&$a, &$b), &$c), d) in a
                    .as_f32()
                    .iter()
                    .zip(b.as_f32())
                    .zip(c.as_f32())
                    .zip(d.as_f32_mut())
                {
                    *d = $x;
                }
            }};

            (|$a:ident: f32, $b:ident: f32, $c:ident: f32, $d:ident: f32, $e:ident: mut f32| $x:expr) => {{
                let (a, b, c, d, e) = registers!($a, $b, $c, $d, mut $e);
                for ((((&$a, &$b), &$c), &$d), e) in a
                    .as_f32()
                    .iter()
                    .zip(b.as_f32())
                    .zip(c.as_f32())
                    .zip(d.as_f32())
                    .zip(e.as_f32_mut())
                {
                    *e = $x;
                }
            }};

            (|$a:ident: i32, $b:ident: i32, $c:ident: i32, $d:ident: mut i32| $x:expr) => {{
                let (a, b, c, d) = registers!($a, $b, $c, mut $d);
                for (((&$a, &$b), &$c), d) in a
                    .as_i32()
                    .iter()
                    .zip(b.as_i32())
                    .zip(c.as_i32())
                    .zip(d.as_i32_mut())
                {
                    *d = $x;
                }
            }};
        }

        for op in self.program.opcodes().iter().copied() {
            match op {
                AddF(a, b, c) => {
                    op!(|a: f32, b: f32, c: mut f32| a + b);
                }
                AddI(a, b, c) => {
                    op!(|a: i32, b: i32, c: mut i32| a.wrapping_add(b));
                }
                SubF(a, b, c) => {
                    op!(|a: f32, b: f32, c: mut f32| a - b);
                }
                SubI(a, b, c) => {
                    op!(|a: i32, b: i32, c: mut i32| a.wrapping_sub(b));
                }
                MulF(a, b, c) => {
                    op!(|a: f32, b: f32, c: mut f32| a * b);
                }
                MulI(a, b, c) => {
                    op!(|a: i32, b: i32, c: mut i32| a.wrapping_mul(b));
                }
                DivF(a, b, c) => {
                    op!(|a: f32, b: f32, c: mut f32| a / b);
                }
                DivI(a, b, c) => {
                    op!(|a: i32, b: i32, c: mut i32| a.wrapping_div(b));
                }
                ModF(a, b, c) => {
                    op!(|a: f32, b: f32, c: mut f32| a.rem_euclid(b));
                }
                ModI(a, b, c) => {
                    op!(|a: i32, b: i32, c: mut i32| a.wrapping_rem_euclid(b));
                }
                MinF(a, b, c) => {
                    op!(|a: f32, b: f32, c: mut f32| a.min(b));
                }
                MinI(a, b, c) => {
                    op!(|a: i32, b: i32, c: mut i32| a.min(b));
                }
                MaxF(a, b, c) => {
                    op!(|a: f32, b: f32, c: mut f32| a.max(b));
                }
                MaxI(a, b, c) => {
                    op!(|a: i32, b: i32, c: mut i32| a.max(b));
                }
                AddCF(a, b, c) => {
                    op!(|b: f32, c: mut f32| a + b);
                }
                AddCI(a, b, c) => {
                    op!(|b: i32, c: mut i32| a.wrapping_add(b));
                }
                SubCF(a, b, c) => {
                    op!(|b: f32, c: mut f32| a - b);
                }
                SubCI(a, b, c) => {
                    op!(|b: i32, c: mut i32| a.wrapping_sub(b));
                }
                MulCF(a, b, c) => {
                    op!(|b: f32, c: mut f32| a * b);
                }
                MulCI(a, b, c) => {
                    op!(|b: i32, c: mut i32| a.wrapping_mul(b));
                }
                MinCF(a, b, c) => {
                    op!(|b: f32, c: mut f32| a.min(b));
                }
                MinCI(a, b, c) => {
                    op!(|b: i32, c: mut i32| a.min(b));
                }
                MaxCF(a, b, c) => {
                    op!(|b: f32, c: mut f32| a.max(b));
                }
                MaxCI(a, b, c) => {
                    op!(|b: i32, c: mut i32| a.max(b));
                }
                AddRF(a, b, c) => {
                    let a: f32 = self.inputs.get(a as usize).copied().unwrap_or_default().into();
                    op!(|b: f32, c: mut f32| a + b);
                }
                MulRF(a, b, c) => {
                    let a: f32 = self.inputs.get(a as usize).copied().unwrap_or_default().into();
                    op!(|b: f32, c: mut f32| a * b);
                }
                MulAddF(a, b, c, d) => {
                    op!(|a: f32, b: f32, c: f32, d: mut f32| a * b + c);
                }
                MulSubF(a, b, c, d) => {
                    op!(|a: f32, b: f32, c: f32, d: mut f32| a * b - c);
                }
                NegF(a, b) => {
                    op!(|a: f32, b: mut f32| -a);
                }
                NegI(a, b) => {
                    op!(|a: i32, b: mut i32| a.wrapping_neg());
                }
                AbsF(a, b) => {
                    op!(|a: f32, b: mut f32| a.abs());
                }
                AbsI(a, b) => {
                    op!(|a: i32, b: mut i32| a.abs());
                }
                RecipF(a, b) => {
                    op!(|a: f32, b: mut f32| a.recip());
                }
                RecipSqrtF(a, b) => {
                    op!(|a: f32, b: mut f32| a.sqrt().recip());
                }
                FloorF(a, b) => {
                    op!(|a: f32, b: mut f32| a.floor());
                }
                SinF(a, b) => {
                    op!(|a: f32, b: mut f32| a.sin());
                }
                CosF(a, b) => {
                    op!(|a: f32, b: mut f32| a.cos());
                }
                TanF(a, b) => {
                    op!(|a: f32, b: mut f32| a.tan());
                }
                AsinF(a, b) => {
                    op!(|a: f32, b: mut f32| a.asin());
                }
                AcosF(a, b) => {
                    op!(|a: f32, b: mut f32| a.acos());
                }
                AtanF(a, b) => {
                    op!(|a: f32, b: mut f32| a.atan());
                }
                Atan2F(a, b, c) => {
                    op!(|a: f32, b: f32, c: mut f32| a.atan2(b));
                }
                SqrtF(a, b) => {
                    op!(|a: f32, b: mut f32| a.sqrt());
                }
                PowF(a, b, c) => {
                    op!(|a: f32, b: f32, c: mut f32| a.powf(b));
                }
                ExpF(a, b) => {
                    op!(|a: f32, b: mut f32| a.exp());
                }
                LnF(a, b) => {
                    op!(|a: f32, b: mut f32| a.ln());
                }
                AndI(a, b, c) => {
                    op!(|a: i32, b: i32, c: mut i32| a & b);
                }
                OrI(a, b, c) => {
                    op!(|a: i32, b: i32, c: mut i32| a | b);
                }
                XorI(a, b, c) => {
                    op!(|a: i32, b: i32, c: mut i32| a ^ b);
                }
                NotI(a, b) => {
                    op!(|a: i32, b: mut i32| !a);
                }
                ShlI(a, b, c) => {
                    op!(|a: i32, b: i32, c: mut i32| a << b);
                }
                ShrI(a, b, c) => {
                    op!(|a: i32, b: i32, c: mut i32| a >> b);
                }
                CastAsF(a, b) => {
                    op!(|a: i32, b: mut f32| a as f32);
                }
                CastAsI(a, b) => {
                    op!(|a: f32, b: mut i32| a as i32);
                }

                EqF(a, b) => {
                    op!(|a: f32, b: mut i32| if a == 0.0 { -1 } else { 0 });
                }
                EqI(a, b) => {
                    op!(|a: i32, b: mut i32| if a == 0 { -1 } else { 0 });
                }
                LtF(a, b) => {
                    op!(|a: f32, b: mut i32| if a < 0.0 { -1 } else { 0 });
                }
                LtI(a, b) => {
                    op!(|a: i32, b: mut i32| if a < 0 { -1 } else { 0 });
                }
                GtF(a, b) => {
                    op!(|a: f32, b: mut i32| if a > 0.0 { -1 } else { 0 });
                }
                GtI(a, b) => {
                    op!(|a: i32,  b: mut i32| if a > 0 { -1 } else { 0 });
                }

                Select(a, b, c, d) => {
                    op!(|a: i32, b: i32, c: i32, d: mut i32| c ^ ((c ^ b) & a));
                }

                Read(idx, reg) => {
                    let (reg,) = registers!(mut reg);
                    let value = self.inputs.get(idx as usize).copied().unwrap_or_default();
                    reg.as_slice_mut().fill(value);
                }

                ReadU16(idx, reg) => {
                    let (reg,) = registers!(mut reg);
                    let value = VMSlot::cast_slice::<u16>(self.inputs)
                        .get(idx as usize)
                        .copied()
                        .unwrap_or_default();

                    reg.as_slice_mut().fill(VMSlot::from(value as i32));
                }

                ReadU8(idx, reg) => {
                    let (reg,) = registers!(mut reg);
                    let value = VMSlot::cast_slice::<u8>(self.inputs)
                        .get(idx as usize)
                        .copied()
                        .unwrap_or_default();

                    reg.as_slice_mut().fill(VMSlot::from(value as i32));
                }

                LitF(val, reg) => {
                    let (reg,) = registers!(mut reg);
                    reg.as_f32_mut().fill(val);
                }
                LitI(val, reg) => {
                    let (reg,) = registers!(mut reg);
                    reg.as_i32_mut().fill(val);
                }

                ResX(reg) => {
                    let (reg,) = registers!(mut reg);
                    reg.as_f32_mut().fill(self.res_x);
                }
                ResY(reg) => {
                    let (reg,) = registers!(mut reg);
                    reg.as_f32_mut().fill(self.res_y);
                }
                QuadT(reg) => {
                    let (reg,) = registers!(mut reg);
                    reg.as_f32_mut().fill(self.quad_t);
                }
                QuadL(reg) => {
                    let (reg,) = registers!(mut reg);
                    reg.as_f32_mut().fill(self.quad_l);
                }
                QuadB(reg) => {
                    let (reg,) = registers!(mut reg);
                    reg.as_f32_mut().fill(self.quad_b);
                }
                QuadR(reg) => {
                    let (reg,) = registers!(mut reg);
                    reg.as_f32_mut().fill(self.quad_r);
                }

                PosX(reg) => {
                    let (reg,) = registers!(mut reg);
                    let reg = reg.as_f32_mut();

                    for i in 0..T::HEIGHT {
                        for j in 0..T::WIDTH {
                            reg[i * T::WIDTH + j] = self.pos_x + j as f32;
                        }
                    }
                }
                PosY(reg) => {
                    let (reg,) = registers!(mut reg);
                    let reg = reg.as_f32_mut();

                    for i in 0..T::HEIGHT {
                        for j in 0..T::WIDTH {
                            reg[i * T::WIDTH + j] = self.pos_y + i as f32;
                        }
                    }
                }

                DxF(reg, out) => {
                    let (reg, out) = registers!(reg, mut out);
                    let reg = reg.as_f32();
                    let out = out.as_f32_mut();

                    for i in (0..T::HEIGHT).step_by(2) {
                        for j in (0..T::WIDTH).step_by(2) {
                            let d = reg[i * T::WIDTH + j + 1] - reg[i * T::WIDTH + j];

                            out[i * T::WIDTH + j] = d;
                            out[i * T::WIDTH + j + 1] = d;
                            out[i * T::WIDTH + j + T::WIDTH] = d;
                            out[i * T::WIDTH + j + T::WIDTH + 1] = d;
                        }
                    }
                }

                DyF(reg, out) => {
                    let (reg, out) = registers!(reg, mut out);
                    let reg = reg.as_f32();
                    let out = out.as_f32_mut();

                    for i in (0..T::HEIGHT).step_by(2) {
                        for j in (0..T::WIDTH).step_by(2) {
                            let d = reg[i * T::WIDTH + j] - reg[i * T::WIDTH + j + T::WIDTH];

                            out[i * T::WIDTH + j] = d;
                            out[i * T::WIDTH + j + 1] = d;
                            out[i * T::WIDTH + j + T::WIDTH] = d;
                            out[i * T::WIDTH + j + T::WIDTH + 1] = d;
                        }
                    }
                }

                TexW(tex, reg) => {
                    let (reg,) = registers!(mut reg);
                    let texture = self.textures.get(tex as usize).copied().unwrap_or_default();
                    reg.as_i32_mut().fill(texture.width() as i32);
                }

                TexH(tex, reg) => {
                    let (reg,) = registers!(mut reg);
                    let texture = self.textures.get(tex as usize).copied().unwrap_or_default();
                    reg.as_i32_mut().fill(texture.height() as i32);
                }

                // TODO: optimize
                Tex(tex, chan, filt, x, y, reg) => {
                    let (x, y, out) = registers!(x, y, mut reg);
                    let texture = self.textures.get(tex as usize).copied().unwrap_or_default();
                    let output = out.as_f32_mut();
                    let x = x.as_f32();
                    let y = y.as_f32();

                    for i in 0..T::HEIGHT {
                        for j in 0..T::WIDTH {
                            let color = texture.sample(x[i * T::WIDTH + j], y[i * T::WIDTH + j], filt);
                            let value = match chan {
                                0 => color.r,
                                1 => color.g,
                                2 => color.b,
                                3 => color.a,
                                _ => 0,
                            };

                            output[i * T::WIDTH + j] = value as f32 / 255.0;
                        }
                    }
                }
            }
        }

        VMResult {
            registers,
            outputs: self.program.output_registers(),
        }
    }
}

impl<'a, T: VMTile> VMResult<'a, T> {
    #[inline(always)]
    pub fn get(&self, index: usize) -> &'a T {
        &self.registers[self.outputs[index] as usize]
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.outputs.len()
    }

    #[inline(always)]
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &'a T> + DoubleEndedIterator {
        (0..self.len()).map(|i| self.get(i))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_vm() {
        let arena = Bump::new();
        let mut memory = VMMemory::new(3, &arena);

        let context = VMContext {
            program: unsafe {
                CompiledProgram::new_unchecked(&[VMOp::LitF(1.0, 0), VMOp::Read(0, 1), VMOp::AddF(0, 1, 2)], &[2], 3)
            },
            inputs: &[VMSlot::from(-1.5)],
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

        let result = unsafe { context.run::<VMSlot>(&mut memory) };
        let result: f32 = (*result.get(0)).into();
        assert_eq!(result, -0.5);
    }
}
