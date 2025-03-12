mod compiler;
mod interpreter;

pub use compiler::*;
pub use interpreter::*;

pub const TILE_SIZE: usize = 16;
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
pub type VMIR = VMOp<u32, ()>;
