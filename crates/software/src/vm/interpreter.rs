use super::{PIXEL_COUNT, REGISTER_COUNT, TILE_SIZE, VMOp, VMOpcode, VMReg};
use bumpalo::{Bump, boxed::Box};
use std::alloc::Layout;

pub struct VMProgram<'a> {
    pub ops: &'a [VMOpcode],
    pub data: &'a [VMSlot],

    pub tile_x: f32,
    pub tile_y: f32,
    pub quad_t: f32,
    pub quad_l: f32,
    pub quad_b: f32,
    pub quad_r: f32,
    pub res_x: f32,
    pub res_y: f32,
}

pub struct VMInterpreter<'a, R> {
    data: Box<'a, [R; REGISTER_COUNT]>,
}

impl<'a, R: VMRegister> VMInterpreter<'a, R> {
    pub fn new(arena: &'a Bump) -> Self {
        Self {
            data: unsafe {
                Box::from_raw(arena.alloc_layout(Layout::new::<[R; 32]>()).cast().as_ptr())
            },
        }
    }

    #[allow(unused_unsafe)]
    #[inline(always)]
    pub unsafe fn execute(&mut self, program: VMProgram) {
        use VMOp::*;

        macro_rules! registers {
                ($($input:expr,)* mut $output:expr) => {
                    unsafe {
                        (
                            $(
                                &*self.data.as_ptr().add($input as usize),
                            )*
                            &mut *self.data.as_mut_ptr().add($output as usize),
                        )
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

        for op in program.ops.iter().copied() {
            match op {
                AddF(a, b, c) => {
                    op!(|a: f32, b: f32, c: mut f32| a + b);
                }
                AddI(a, b, c) => {
                    op!(|a: i32, b: i32, c: mut i32| a + b);
                }
                SubF(a, b, c) => {
                    op!(|a: f32, b: f32, c: mut f32| a - b);
                }
                SubI(a, b, c) => {
                    op!(|a: i32, b: i32, c: mut i32| a - b);
                }
                MulF(a, b, c) => {
                    op!(|a: f32, b: f32, c: mut f32| a * b);
                }
                MulI(a, b, c) => {
                    op!(|a: i32, b: i32, c: mut i32| a * b);
                }
                DivF(a, b, c) => {
                    op!(|a: f32, b: f32, c: mut f32| a / b);
                }
                DivI(a, b, c) => {
                    op!(|a: i32, b: i32, c: mut i32| a / b);
                }
                ModF(a, b, c) => {
                    op!(|a: f32, b: f32, c: mut f32| a % b);
                }
                ModI(a, b, c) => {
                    op!(|a: i32, b: i32, c: mut i32| a % b);
                }
                AddCF(a, b, c) => {
                    op!(|b: f32, c: mut f32| a + b);
                }
                AddCI(a, b, c) => {
                    op!(|b: i32, c: mut i32| a + b);
                }
                SubCF(a, b, c) => {
                    op!(|b: f32, c: mut f32| a - b);
                }
                SubCI(a, b, c) => {
                    op!(|b: i32, c: mut i32| a - b);
                }
                MulCF(a, b, c) => {
                    op!(|b: f32, c: mut f32| a * b);
                }
                MulCI(a, b, c) => {
                    op!(|b: i32, c: mut i32| a * b);
                }
                NegF(a, b) => {
                    op!(|a: f32, b: mut f32| -a);
                }
                NegI(a, b) => {
                    op!(|a: i32, b: mut i32| -a);
                }
                AbsF(a, b) => {
                    op!(|a: f32, b: mut f32| a.abs());
                }
                AbsI(a, b) => {
                    op!(|a: i32, b: mut i32| a.abs());
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
                CastF(a, b) => {
                    op!(|a: i32, b: mut f32| a as f32);
                }
                CastI(a, b) => {
                    op!(|a: f32, b: mut i32| a as i32);
                }

                EqF(a, b, c) => {
                    op!(|a: f32, b: f32, c: mut i32| if a == b { -1 } else { 0 });
                }
                EqI(a, b, c) => {
                    op!(|a: i32, b: i32, c: mut i32| if a == b { -1 } else { 0 });
                }
                LtF(a, b, c) => {
                    op!(|a: f32, b: f32, c: mut i32| if a < b { -1 } else { 0 });
                }
                LtI(a, b, c) => {
                    op!(|a: i32, b: i32, c: mut i32| if a < b { -1 } else { 0 });
                }
                GtF(a, b, c) => {
                    op!(|a: f32, b: f32, c: mut i32| if a > b { -1 } else { 0 });
                }
                GtI(a, b, c) => {
                    op!(|a: i32, b: i32, c: mut i32| if a > b { -1 } else { 0 });
                }
                NeF(a, b, c) => {
                    op!(|a: f32, b: f32, c: mut i32| if a != b { -1 } else { 0 });
                }
                NeI(a, b, c) => {
                    op!(|a: i32, b: i32, c: mut i32| if a != b { -1 } else { 0 });
                }
                LeF(a, b, c) => {
                    op!(|a: f32, b: f32, c: mut i32| if a <= b { -1 } else { 0 });
                }
                LeI(a, b, c) => {
                    op!(|a: i32, b: i32, c: mut i32| if a <= b { -1 } else { 0 });
                }
                GeF(a, b, c) => {
                    op!(|a: f32, b: f32, c: mut i32| if a >= b { -1 } else { 0 });
                }
                GeI(a, b, c) => {
                    op!(|a: i32, b: i32, c: mut i32| if a >= b { -1 } else { 0 });
                }
                Select(a, b, c, d) => {
                    op!(|a: i32, b: i32, c: i32, d: mut i32| b ^ ((b ^ c) & a));
                }

                ReadF(idx, reg) => unsafe {
                    let (reg,) = registers!(mut reg);
                    reg.as_f32_mut()
                        .fill(program.data.get_unchecked(idx as usize).float);
                },

                ReadI(idx, reg) => unsafe {
                    let (reg,) = registers!(mut reg);
                    reg.as_i32_mut()
                        .fill(program.data.get_unchecked(idx as usize).int);
                },

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
                    reg.as_f32_mut().fill(program.res_x);
                }
                ResY(reg) => {
                    let (reg,) = registers!(mut reg);
                    reg.as_f32_mut().fill(program.res_y);
                }
                QuadT(reg) => {
                    let (reg,) = registers!(mut reg);
                    reg.as_f32_mut().fill(program.quad_t);
                }
                QuadL(reg) => {
                    let (reg,) = registers!(mut reg);
                    reg.as_f32_mut().fill(program.quad_l);
                }
                QuadB(reg) => {
                    let (reg,) = registers!(mut reg);
                    reg.as_f32_mut().fill(program.quad_b);
                }
                QuadR(reg) => {
                    let (reg,) = registers!(mut reg);
                    reg.as_f32_mut().fill(program.quad_r);
                }

                PosX(reg) => {
                    let (reg,) = registers!(mut reg);
                    let reg = reg.as_f32_mut();

                    for i in 0..R::HEIGHT {
                        for j in 0..R::WIDTH {
                            reg[i * R::WIDTH + j] = program.tile_x + j as f32;
                        }
                    }
                }
                PosY(reg) => {
                    let (reg,) = registers!(mut reg);
                    let reg = reg.as_f32_mut();

                    for i in 0..R::HEIGHT {
                        for j in 0..R::WIDTH {
                            reg[i * R::WIDTH + j] = program.tile_y + i as f32;
                        }
                    }
                }

                DxF(reg, out) => {
                    let (reg, out) = registers!(reg, mut out);
                    let reg = reg.as_f32();
                    let out = out.as_f32_mut();

                    for i in 0..R::HEIGHT {
                        for j in (0..R::WIDTH).step_by(2) {
                            let a = reg[i * R::WIDTH + j];
                            let b = reg[i * R::WIDTH + j + 1];
                            out[i * R::WIDTH + j] = b - a;
                            out[i * R::WIDTH + j + 1] = b - a;
                        }
                    }
                }

                DyF(reg, out) => {
                    let (reg, out) = registers!(reg, mut out);
                    let reg = reg.as_f32();
                    let out = out.as_f32_mut();

                    for i in (0..R::HEIGHT).step_by(2) {
                        for j in 0..R::WIDTH {
                            let a = reg[i * R::WIDTH + j];
                            let b = reg[(i + 1) * R::WIDTH + j];
                            out[i * R::WIDTH + j] = b - a;
                            out[(i + 1) * R::WIDTH + j] = b - a;
                        }
                    }
                }
            }
        }
    }

    pub fn register(&self, id: VMReg) -> &R {
        &self.data[id as usize]
    }
}

#[derive(Copy, Clone)]
pub union VMSlot {
    pub int: i32,
    pub float: f32,
}

#[derive(Copy, Clone)]
#[repr(align(128))]
pub struct VMTile([VMSlot; PIXEL_COUNT]);

impl VMTile {
    #[inline(always)]
    pub fn as_f32(&self) -> &[f32; PIXEL_COUNT] {
        unsafe { &*(&self.0 as *const _ as *const [f32; PIXEL_COUNT]) }
    }
}

/// SAFETY: should be zeroable
pub unsafe trait VMRegister {
    const WIDTH: usize;
    const HEIGHT: usize;
    fn as_i32(&self) -> &[i32];
    fn as_f32(&self) -> &[f32];
    fn as_i32_mut(&mut self) -> &mut [i32];
    fn as_f32_mut(&mut self) -> &mut [f32];
}

unsafe impl VMRegister for VMSlot {
    const WIDTH: usize = 1;
    const HEIGHT: usize = 1;

    #[inline(always)]
    fn as_i32(&self) -> &[i32] {
        unsafe { &*(&self.int as *const _ as *const [i32; 1]) }
    }
    #[inline(always)]
    fn as_f32(&self) -> &[f32] {
        unsafe { &*(&self.float as *const _ as *const [f32; 1]) }
    }
    #[inline(always)]
    fn as_i32_mut(&mut self) -> &mut [i32] {
        unsafe { &mut *(&mut self.int as *mut _ as *mut [i32; 1]) }
    }
    #[inline(always)]
    fn as_f32_mut(&mut self) -> &mut [f32] {
        unsafe { &mut *(&mut self.float as *mut _ as *mut [f32; 1]) }
    }
}

unsafe impl VMRegister for VMTile {
    const WIDTH: usize = TILE_SIZE;
    const HEIGHT: usize = TILE_SIZE;
    #[inline(always)]
    fn as_i32(&self) -> &[i32] {
        unsafe { &*(&self.0 as *const _ as *const [i32; PIXEL_COUNT]) }
    }
    #[inline(always)]
    fn as_f32(&self) -> &[f32] {
        unsafe { &*(&self.0 as *const _ as *const [f32; PIXEL_COUNT]) }
    }
    #[inline(always)]
    fn as_i32_mut(&mut self) -> &mut [i32] {
        unsafe { &mut *(&mut self.0 as *mut _ as *mut [i32; PIXEL_COUNT]) }
    }
    #[inline(always)]
    fn as_f32_mut(&mut self) -> &mut [f32] {
        unsafe { &mut *(&mut self.0 as *mut _ as *mut [f32; PIXEL_COUNT]) }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_vm() {
        let arena = Bump::new();
        let mut interpreter = VMInterpreter::<VMSlot>::new(&arena);
        let program = VMProgram {
            ops: &[VMOp::LitF(1.0, 0), VMOp::ReadF(0, 1), VMOp::AddF(0, 1, 2)],
            data: &[VMSlot { float: -1.5 }],
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

        let result = interpreter.register(2).as_f32();
        assert_eq!(result, &[-0.5; 1]);
    }
}
