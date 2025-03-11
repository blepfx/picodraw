use multiversion::multiversion;
use std::alloc::{Layout, alloc_zeroed};

pub const TILE_SIZE: usize = 32;
pub const REGISTER_COUNT: usize = 32;
pub const PIXEL_COUNT: usize = TILE_SIZE * TILE_SIZE;

#[repr(align(8))]
#[derive(Debug, Clone, Copy)]
pub enum VMOp<I, O> {
    PosX(O),
    PosY(O),
    ResX(O),
    ResY(O),
    QuadT(O),
    QuadL(O),
    QuadB(O),
    QuadR(O),

    LitF(f32, O),
    LitI(i32, O),

    ReadF(u32, O),
    ReadI(u32, O),

    AddI(I, I, O),
    AddF(I, I, O),
    SubI(I, I, O),
    SubF(I, I, O),
    MulI(I, I, O),
    MulF(I, I, O),
    DivI(I, I, O),
    DivF(I, I, O),
    ModI(I, I, O),
    ModF(I, I, O),

    AddCI(i32, I, O),
    AddCF(f32, I, O),
    SubCI(i32, I, O),
    SubCF(f32, I, O),
    MulCI(i32, I, O),
    MulCF(f32, I, O),

    NegF(I, O),
    NegI(I, O),

    MinF(I, I, O),
    MinI(I, I, O),
    MaxF(I, I, O),
    MaxI(I, I, O),
    AbsF(I, O),
    AbsI(I, O),
    FloorF(I, O),

    SinF(I, O),
    CosF(I, O),
    TanF(I, O),

    AsinF(I, O),
    AcosF(I, O),
    AtanF(I, O),
    Atan2F(I, I, O),

    SqrtF(I, O),
    PowF(I, I, O),
    ExpF(I, O),
    LnF(I, O),

    AndI(I, I, O),
    OrI(I, I, O),
    XorI(I, I, O),
    NotI(I, O),

    Select(I, I, I, O),

    CastF(I, O),
    CastI(I, O),

    DxF(I, O),
    DyF(I, O),

    EqI(I, I, O),
    EqF(I, I, O),
    LtI(I, I, O),
    LtF(I, I, O),
    GtI(I, I, O),
    GtF(I, I, O),
    NeI(I, I, O),
    NeF(I, I, O),
    LeI(I, I, O),
    LeF(I, I, O),
    GeI(I, I, O),
    GeF(I, I, O),
}

impl<I, O> VMOp<I, O> {
    pub fn map<I0, O0>(
        self,
        mut inp: impl FnMut(I) -> I0,
        out: impl FnOnce(O) -> O0,
    ) -> VMOp<I0, O0> {
        use VMOp::*;
        match self {
            PosX(o) => PosX(out(o)),
            PosY(o) => PosY(out(o)),
            ResX(o) => ResX(out(o)),
            ResY(o) => ResY(out(o)),
            QuadT(o) => QuadT(out(o)),
            QuadL(o) => QuadL(out(o)),
            QuadB(o) => QuadB(out(o)),
            QuadR(o) => QuadR(out(o)),
            LitF(val, o) => LitF(val, out(o)),
            LitI(val, o) => LitI(val, out(o)),
            ReadF(idx, o) => ReadF(idx, out(o)),
            ReadI(idx, o) => ReadI(idx, out(o)),
            AddI(a, b, o) => AddI(inp(a), inp(b), out(o)),
            AddF(a, b, o) => AddF(inp(a), inp(b), out(o)),
            SubI(a, b, o) => SubI(inp(a), inp(b), out(o)),
            SubF(a, b, o) => SubF(inp(a), inp(b), out(o)),
            MulI(a, b, o) => MulI(inp(a), inp(b), out(o)),
            MulF(a, b, o) => MulF(inp(a), inp(b), out(o)),
            DivI(a, b, o) => DivI(inp(a), inp(b), out(o)),
            DivF(a, b, o) => DivF(inp(a), inp(b), out(o)),
            ModI(a, b, o) => ModI(inp(a), inp(b), out(o)),
            ModF(a, b, o) => ModF(inp(a), inp(b), out(o)),
            AddCI(a, b, o) => AddCI(a, inp(b), out(o)),
            AddCF(a, b, o) => AddCF(a as f32, inp(b), out(o)),
            SubCI(a, b, o) => SubCI(a, inp(b), out(o)),
            SubCF(a, b, o) => SubCF(a as f32, inp(b), out(o)),
            MulCI(a, b, o) => MulCI(a, inp(b), out(o)),
            MulCF(a, b, o) => MulCF(a as f32, inp(b), out(o)),
            NegF(a, o) => NegF(inp(a), out(o)),
            NegI(a, o) => NegI(inp(a), out(o)),
            MinF(a, b, o) => MinF(inp(a), inp(b), out(o)),
            MinI(a, b, o) => MinI(inp(a), inp(b), out(o)),
            MaxF(a, b, o) => MaxF(inp(a), inp(b), out(o)),
            MaxI(a, b, o) => MaxI(inp(a), inp(b), out(o)),
            AbsF(a, o) => AbsF(inp(a), out(o)),
            AbsI(a, o) => AbsI(inp(a), out(o)),
            FloorF(a, o) => FloorF(inp(a), out(o)),
            SinF(a, o) => SinF(inp(a), out(o)),
            CosF(a, o) => CosF(inp(a), out(o)),
            TanF(a, o) => TanF(inp(a), out(o)),
            AsinF(a, o) => AsinF(inp(a), out(o)),
            AcosF(a, o) => AcosF(inp(a), out(o)),
            AtanF(a, o) => AtanF(inp(a), out(o)),
            Atan2F(a, b, o) => Atan2F(inp(a), inp(b), out(o)),
            SqrtF(a, o) => SqrtF(inp(a), out(o)),
            PowF(a, b, o) => PowF(inp(a), inp(b), out(o)),
            ExpF(a, o) => ExpF(inp(a), out(o)),
            LnF(a, o) => LnF(inp(a), out(o)),
            AndI(a, b, o) => AndI(inp(a), inp(b), out(o)),
            OrI(a, b, o) => OrI(inp(a), inp(b), out(o)),
            XorI(a, b, o) => XorI(inp(a), inp(b), out(o)),
            NotI(a, o) => NotI(inp(a), out(o)),
            Select(a, b, c, o) => Select(inp(a), inp(b), inp(c), out(o)),
            CastF(a, o) => CastF(inp(a), out(o)),
            CastI(a, o) => CastI(inp(a), out(o)),
            DxF(a, o) => DxF(inp(a), out(o)),
            DyF(a, o) => DyF(inp(a), out(o)),
            EqI(a, b, o) => EqI(inp(a), inp(b), out(o)),
            EqF(a, b, o) => EqF(inp(a), inp(b), out(o)),
            LtI(a, b, o) => LtI(inp(a), inp(b), out(o)),
            LtF(a, b, o) => LtF(inp(a), inp(b), out(o)),
            GtI(a, b, o) => GtI(inp(a), inp(b), out(o)),
            GtF(a, b, o) => GtF(inp(a), inp(b), out(o)),
            NeI(a, b, o) => NeI(inp(a), inp(b), out(o)),
            NeF(a, b, o) => NeF(inp(a), inp(b), out(o)),
            LeI(a, b, o) => LeI(inp(a), inp(b), out(o)),
            LeF(a, b, o) => LeF(inp(a), inp(b), out(o)),
            GeI(a, b, o) => GeI(inp(a), inp(b), out(o)),
            GeF(a, b, o) => GeF(inp(a), inp(b), out(o)),
        }
    }

    pub fn output(self) -> O {
        let mut output = None;
        self.map(|_| (), |o| output = Some(o));
        output.unwrap()
    }
}

pub type VMReg = u8;
pub type VMOpcode = VMOp<VMReg, VMReg>;

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
            use VMOp::*;

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

        unsafe {
            execute_simd(self, program);
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
        let mut interpreter = VMInterpreter::<VMSlot>::new();
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
