use multiversion::multiversion;
use std::alloc::{Layout, alloc_zeroed};

pub const TILE_SIZE: usize = 32;
pub const REGISTER_COUNT: usize = 32;
pub const PIXEL_COUNT: usize = TILE_SIZE * TILE_SIZE;

#[derive(Debug, Clone, Copy)]
pub enum VMOpcode {
    PosX(VMReg),
    PosY(VMReg),
    ResX(VMReg),
    ResY(VMReg),
    QuadT(VMReg),
    QuadL(VMReg),
    QuadB(VMReg),
    QuadR(VMReg),

    LitF(f32, VMReg),
    LitI(i32, VMReg),

    ReadF(u32, VMReg),
    ReadI(u32, VMReg),

    AddI(VMReg, VMReg, VMReg),
    AddF(VMReg, VMReg, VMReg),
    SubI(VMReg, VMReg, VMReg),
    SubF(VMReg, VMReg, VMReg),
    MulI(VMReg, VMReg, VMReg),
    MulF(VMReg, VMReg, VMReg),
    DivI(VMReg, VMReg, VMReg),
    DivF(VMReg, VMReg, VMReg),
    ModI(VMReg, VMReg, VMReg),
    ModF(VMReg, VMReg, VMReg),
    NegF(VMReg, VMReg),
    NegI(VMReg, VMReg),

    Add3F(VMReg, VMReg, VMReg, VMReg),
    Add4F(VMReg, VMReg, VMReg, VMReg, VMReg),

    MinF(VMReg, VMReg, VMReg),
    MinI(VMReg, VMReg, VMReg),
    MaxF(VMReg, VMReg, VMReg),
    MaxI(VMReg, VMReg, VMReg),
    AbsF(VMReg, VMReg),
    AbsI(VMReg, VMReg),
    FloorF(VMReg, VMReg),

    SinF(VMReg, VMReg),
    CosF(VMReg, VMReg),
    TanF(VMReg, VMReg),

    AsinF(VMReg, VMReg),
    AcosF(VMReg, VMReg),
    AtanF(VMReg, VMReg),
    Atan2F(VMReg, VMReg, VMReg),

    SqrtF(VMReg, VMReg),
    PowF(VMReg, VMReg, VMReg),
    ExpF(VMReg, VMReg),
    LnF(VMReg, VMReg),

    AndI(VMReg, VMReg, VMReg),
    OrI(VMReg, VMReg, VMReg),
    XorI(VMReg, VMReg, VMReg),
    NotI(VMReg, VMReg),

    LerpF(VMReg, VMReg, VMReg, VMReg),
    SmoothstepF(VMReg, VMReg, VMReg, VMReg),

    Select(VMReg, VMReg, VMReg, VMReg),

    CastF(VMReg, VMReg),
    CastI(VMReg, VMReg),

    DxF(VMReg, VMReg),
    DyF(VMReg, VMReg),

    EqI(VMReg, VMReg, VMReg),
    EqF(VMReg, VMReg, VMReg),
    LtI(VMReg, VMReg, VMReg),
    LtF(VMReg, VMReg, VMReg),
    GtI(VMReg, VMReg, VMReg),
    GtF(VMReg, VMReg, VMReg),
    NeI(VMReg, VMReg, VMReg),
    NeF(VMReg, VMReg, VMReg),
    LeI(VMReg, VMReg, VMReg),
    LeF(VMReg, VMReg, VMReg),
    GeI(VMReg, VMReg, VMReg),
    GeF(VMReg, VMReg, VMReg),
}
pub type VMReg = u8;

#[derive(Copy, Clone)]
pub union VMSlot {
    pub int: i32,
    pub float: f32,
}

#[derive(Copy, Clone)]
#[repr(align(128))]
pub struct VMTile([VMSlot; PIXEL_COUNT]);

/// SAFETY: should be zeroable
pub unsafe trait VMRegister {
    const WIDTH: usize;
    const HEIGHT: usize;
    fn as_i32(&self) -> &[i32];
    fn as_f32(&self) -> &[f32];
    fn as_i32_mut(&mut self) -> &mut [i32];
    fn as_f32_mut(&mut self) -> &mut [f32];
}

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

pub struct VMInterpreter<R> {
    data: Box<[R; REGISTER_COUNT]>,
}

impl<R: VMRegister> VMInterpreter<R> {
    pub fn new() -> Self {
        Self {
            data: unsafe { Box::from_raw(alloc_zeroed(Layout::new::<[R; 32]>()) as *mut [R; 32]) },
        }
    }

    pub unsafe fn execute(&mut self, program: VMProgram) {
        #[inline(always)]
        #[allow(unused_unsafe)]
        #[multiversion(targets = "simd")]
        unsafe fn execute_simd<R: VMRegister>(state: &mut VMInterpreter<R>, program: VMProgram) {
            use VMOpcode::*;

            macro_rules! registers {
                ($($input:expr,)* mut $output:expr) => {
                    unsafe {
                        (
                            $(
                                &*state.data.as_ptr().add($input as usize),
                            )*
                            &mut *state.data.as_mut_ptr().add($output as usize),
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
                    Add3F(a, b, c, d) => {
                        op!(|a: f32, b: f32, c: f32, d: mut f32| a + b + c);
                    }
                    Add4F(a, b, c, d, e) => {
                        op!(|a: f32, b: f32, c: f32, d: f32, e: mut f32| a + b + c + d);
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
                    LerpF(a, b, c, d) => {
                        op!(|a: f32, b: f32, c: f32, d: mut f32| a.mul_add(c - b, b));
                    }
                    SmoothstepF(a, b, c, d) => {
                        op!(|a: f32, b: f32, c: f32, d: mut f32| {
                            let t = ((a - b) / (c - b)).clamp(0.0, 1.0);
                            t * t * (3.0 - 2.0 * t)
                        });
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

        unsafe {
            execute_simd(self, program);
        }
    }

    pub fn register(&self, id: VMReg) -> &R {
        &self.data[id as usize]
    }
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
        let mut interpreter = VMInterpreter::<VMSlot>::new();
        let program = VMProgram {
            ops: &[
                VMOpcode::LitF(1.0, 0),
                VMOpcode::ReadF(0, 1),
                VMOpcode::AddF(0, 1, 2),
            ],
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
