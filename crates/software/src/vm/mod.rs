mod compiler;
mod interpreter;

pub use compiler::*;
pub use interpreter::*;

pub const TILE_SIZE: usize = 16;
pub const REGISTER_COUNT: usize = 64;
pub const PIXEL_COUNT: usize = TILE_SIZE * TILE_SIZE;

#[repr(align(8))]
#[derive(PartialEq, Debug, Clone, Copy)]
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

    Read(u32, O),

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
    MinF(I, I, O),
    MinI(I, I, O),
    MaxF(I, I, O),
    MaxI(I, I, O),

    AddCI(i32, I, O),
    AddCF(f32, I, O),
    SubCI(i32, I, O),
    SubCF(f32, I, O),
    MulCI(i32, I, O),
    MulCF(f32, I, O),
    MinCI(i32, I, O),
    MinCF(f32, I, O),
    MaxCI(i32, I, O),
    MaxCF(f32, I, O),

    Add3F(I, I, I, O),
    Add3I(I, I, I, O),
    Mul3F(I, I, I, O),
    Mul3I(I, I, I, O),

    NegF(I, O),
    NegI(I, O),

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
    ShlI(I, I, O),
    ShrI(I, I, O),

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

    TexW(u8, O),
    TexH(u8, O),
    Tex(u8, u8, picodraw_core::TextureFilter, I, I, O),
}

impl<I, O> VMOp<I, O> {
    #[doc(hidden)]
    fn map_inner<I0, O0>(self, mut inp: impl FnMut(I) -> I0, out: impl FnOnce(O) -> O0) -> VMOp<I0, O0> {
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
            Read(idx, o) => Read(idx, out(o)),
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
            AddCF(a, b, o) => AddCF(a, inp(b), out(o)),
            SubCI(a, b, o) => SubCI(a, inp(b), out(o)),
            SubCF(a, b, o) => SubCF(a, inp(b), out(o)),
            MulCI(a, b, o) => MulCI(a, inp(b), out(o)),
            MulCF(a, b, o) => MulCF(a, inp(b), out(o)),
            MinCI(a, b, o) => MinCI(a, inp(b), out(o)),
            MinCF(a, b, o) => MinCF(a, inp(b), out(o)),
            MaxCI(a, b, o) => MaxCI(a, inp(b), out(o)),
            MaxCF(a, b, o) => MaxCF(a, inp(b), out(o)),

            Add3F(a, b, c, o) => Add3F(inp(a), inp(b), inp(c), out(o)),
            Add3I(a, b, c, o) => Add3I(inp(a), inp(b), inp(c), out(o)),
            Mul3F(a, b, c, o) => Mul3F(inp(a), inp(b), inp(c), out(o)),
            Mul3I(a, b, c, o) => Mul3I(inp(a), inp(b), inp(c), out(o)),

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
            ShlI(a, b, o) => ShlI(inp(a), inp(b), out(o)),
            ShrI(a, b, o) => ShrI(inp(a), inp(b), out(o)),
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

            TexW(a, o) => TexW(a, out(o)),
            TexH(a, o) => TexH(a, out(o)),
            Tex(a, b, c, d, e, o) => Tex(a, b, c, inp(d), inp(e), out(o)),
        }
    }

    pub fn map_inputs<I0>(self, inp: impl FnMut(I) -> I0) -> VMOp<I0, O> {
        self.map_inner(inp, |o| o)
    }

    pub fn map_outputs<O0>(self, out: impl FnOnce(O) -> O0) -> VMOp<I, O0> {
        self.map_inner(|i| i, out)
    }

    pub fn output(self) -> O {
        let mut output = None;
        self.map_inner(|_| (), |o| output = Some(o));
        output.unwrap()
    }
}

pub type VMReg = u8;
pub type VMOpcode = VMOp<VMReg, VMReg>;
