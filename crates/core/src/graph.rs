use std::{
    cell::RefCell,
    hash::{DefaultHasher, Hash, Hasher},
    mem::replace,
    ops::BitAnd,
};

thread_local! {
    static COLLECT_GRAPH: RefCell<Option<Graph>> = RefCell::new(None);
}

pub struct Graph {
    ops: Vec<Op>,
    hash: DefaultHasher,
}

#[derive(Debug, Hash)]
pub struct OpInfo {
    pub addr: OpAddr,
    pub value: Op,
    pub ty: OpType,
    pub dynamic: OpDynamic,
    pub dependencies: Vec<OpAddr>,
    pub dependents: Vec<OpAddr>,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum OpInput {
    TextureStatic,
    TextureRender,

    F32,
    I32,
    I16,
    I8,
    U32,
    U16,
    U8,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum OpType {
    F1,
    F2,
    F3,
    F4,
    I1,
    I2,
    I3,
    I4,
    Boolean,
    TextureStatic,
    TextureRender,
    Void,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum OpDynamic {
    Const,
    PerFrame,
    PerObject,
    PerPixel,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct OpAddr(pub u32);

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Op {
    Position,
    Resolution,
    QuadStart,
    QuadEnd,

    Input(OpInput),
    Output(OpAddr),

    LiteralFloat(Float),
    LiteralInt(i32),
    LiteralBool(bool),

    Add(OpAddr, OpAddr),
    Sub(OpAddr, OpAddr),
    Mul(OpAddr, OpAddr),
    Div(OpAddr, OpAddr),
    Rem(OpAddr, OpAddr),
    Dot(OpAddr, OpAddr),
    Cross(OpAddr, OpAddr),
    Neg(OpAddr),

    Sin(OpAddr),
    Cos(OpAddr),
    Tan(OpAddr),

    Asin(OpAddr),
    Acos(OpAddr),
    Atan(OpAddr),
    Atan2(OpAddr, OpAddr),

    Sqrt(OpAddr),
    Pow(OpAddr, OpAddr),
    Exp(OpAddr),
    Ln(OpAddr),

    Min(OpAddr, OpAddr),
    Max(OpAddr, OpAddr),
    Clamp(OpAddr, OpAddr, OpAddr),
    Abs(OpAddr),
    Sign(OpAddr),
    Floor(OpAddr),

    Lerp(OpAddr, OpAddr, OpAddr),
    Select(OpAddr, OpAddr, OpAddr),
    Smoothstep(OpAddr, OpAddr, OpAddr),
    Step(OpAddr, OpAddr),

    Eq(OpAddr, OpAddr),
    Ne(OpAddr, OpAddr),
    Lt(OpAddr, OpAddr),
    Le(OpAddr, OpAddr),
    Gt(OpAddr, OpAddr),
    Ge(OpAddr, OpAddr),

    And(OpAddr, OpAddr),
    Or(OpAddr, OpAddr),
    Xor(OpAddr, OpAddr),
    Not(OpAddr),

    Vec2(OpAddr, OpAddr),
    Vec3(OpAddr, OpAddr, OpAddr),
    Vec4(OpAddr, OpAddr, OpAddr, OpAddr),

    Splat2(OpAddr),
    Splat3(OpAddr),
    Splat4(OpAddr),

    CastFloat(OpAddr),
    CastInt(OpAddr),

    Swizzle1(OpAddr, [Swizzle; 1]),
    Swizzle2(OpAddr, [Swizzle; 2]),
    Swizzle3(OpAddr, [Swizzle; 3]),
    Swizzle4(OpAddr, [Swizzle; 4]),

    Length(OpAddr),
    Normalize(OpAddr),

    DerivX(OpAddr),
    DerivY(OpAddr),
    DerivWidth(OpAddr),

    TextureLinear(OpAddr, OpAddr),
    TextureNearest(OpAddr, OpAddr),
    TextureSize(OpAddr),
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Swizzle {
    X,
    Y,
    Z,
    W,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Float(u32);

impl Graph {
    pub fn push_collect(op: Op) -> OpAddr {
        COLLECT_GRAPH.with(|graph| {
            let mut graph = graph.borrow_mut();
            let graph = graph.as_mut().expect("not executing in a shader graph context");

            graph.push(op)
        })
    }

    pub fn collect(f: impl FnOnce()) -> Self {
        let prev = COLLECT_GRAPH.with(|engine| replace(&mut *engine.borrow_mut(), Some(Self::empty())));
        f();
        COLLECT_GRAPH
            .with(|engine| replace(&mut *engine.borrow_mut(), prev))
            .unwrap()
    }

    pub fn empty() -> Self {
        Self {
            ops: vec![],
            hash: DefaultHasher::new(),
        }
    }

    pub fn resolve(&self) -> Vec<OpInfo> {
        let mut infos: Vec<OpInfo> = vec![];
        for (idx, op) in self.ops.iter().enumerate() {
            let addr = OpAddr(idx as u32);
            let info = Op::info(*op, addr, &infos);

            for dep in info.dependencies.iter() {
                infos[dep.0 as usize].dependents.push(addr);
            }
            infos.push(info);
        }

        infos
    }

    pub fn iter(&self) -> impl Iterator<Item = (OpAddr, Op)> + '_ {
        self.ops.iter().enumerate().map(|(id, op)| (OpAddr(id as u32), *op))
    }

    pub fn hash(&self) -> u64 {
        self.hash.finish()
    }

    pub fn push(&mut self, value: Op) -> OpAddr {
        value.hash(&mut self.hash);
        self.ops.push(value);
        OpAddr((self.ops.len() - 1) as u32)
    }
}

impl OpType {
    pub fn is_numeric(self) -> bool {
        use OpType::*;
        matches!(self, F1 | F2 | F3 | F4 | I1 | I2 | I3 | I4)
    }
}

impl Op {
    fn info(value: Self, addr: OpAddr, ops: &[OpInfo]) -> OpInfo {
        use OpDynamic::*;
        use OpType::*;

        let get = |addr: OpAddr| ops.get(addr.0 as usize).expect("invalid op addr");

        match value {
            Op::Position => OpInfo {
                addr,
                value,
                ty: F2,
                dynamic: PerPixel,
                dependencies: vec![],
                dependents: vec![],
            },

            Op::QuadStart | Op::QuadEnd => OpInfo {
                addr,
                value,
                ty: F2,
                dynamic: PerObject,
                dependencies: vec![],
                dependents: vec![],
            },

            Op::Resolution => OpInfo {
                addr,
                value,
                ty: F2,
                dynamic: PerFrame,
                dependencies: vec![],
                dependents: vec![],
            },

            Op::Input(OpInput::F32) => OpInfo {
                addr,
                value,
                ty: F1,
                dynamic: PerObject,
                dependencies: vec![],
                dependents: vec![],
            },

            Op::Input(OpInput::I32 | OpInput::I16 | OpInput::I8 | OpInput::U32 | OpInput::U16 | OpInput::U8) => {
                OpInfo {
                    addr,
                    value,
                    ty: I1,
                    dynamic: PerObject,
                    dependencies: vec![],
                    dependents: vec![],
                }
            }

            Op::Input(OpInput::TextureStatic) => OpInfo {
                addr,
                value,
                ty: TextureStatic,
                dynamic: PerObject,
                dependencies: vec![],
                dependents: vec![],
            },

            Op::Input(OpInput::TextureRender) => OpInfo {
                addr,
                value,
                ty: TextureRender,
                dynamic: PerObject,
                dependencies: vec![],
                dependents: vec![],
            },

            Op::Output(x) => {
                let arg0 = get(x);

                if arg0.ty != F4 {
                    panic!("type check");
                }

                OpInfo {
                    addr,
                    value,
                    ty: Void,
                    dynamic: arg0.dynamic,
                    dependencies: vec![x],
                    dependents: vec![],
                }
            }

            Op::LiteralFloat(_) => OpInfo {
                addr,
                value,
                ty: F1,
                dynamic: Const,
                dependencies: vec![],
                dependents: vec![],
            },

            Op::LiteralInt(_) => OpInfo {
                addr,
                value,
                ty: I1,
                dynamic: Const,
                dependencies: vec![],
                dependents: vec![],
            },

            Op::LiteralBool(_) => OpInfo {
                addr,
                value,
                ty: Boolean,
                dynamic: Const,
                dependencies: vec![],
                dependents: vec![],
            },

            Op::Add(l, r)
            | Op::Sub(l, r)
            | Op::Mul(l, r)
            | Op::Div(l, r)
            | Op::Rem(l, r)
            | Op::Min(l, r)
            | Op::Max(l, r)
            | Op::Step(l, r) => {
                let arg0 = get(l);
                let arg1 = get(r);

                if arg0.ty != arg1.ty || !arg0.ty.is_numeric() {
                    panic!("type check {:?}", (value, arg0, arg1));
                }

                OpInfo {
                    addr,
                    value,
                    ty: arg0.ty,
                    dynamic: arg0.dynamic & arg1.dynamic,
                    dependencies: vec![l, r],
                    dependents: vec![],
                }
            }

            Op::Sin(x)
            | Op::Cos(x)
            | Op::Tan(x)
            | Op::Asin(x)
            | Op::Acos(x)
            | Op::Atan(x)
            | Op::Sqrt(x)
            | Op::Exp(x)
            | Op::Ln(x)
            | Op::Floor(x)
            | Op::DerivX(x)
            | Op::DerivY(x)
            | Op::DerivWidth(x)
            | Op::Normalize(x) => {
                let arg0 = get(x);

                if !matches!(arg0.ty, F1 | F2 | F3 | F4) {
                    panic!("type check");
                }

                OpInfo {
                    addr,
                    value,
                    ty: arg0.ty,
                    dynamic: arg0.dynamic,
                    dependencies: vec![x],
                    dependents: vec![],
                }
            }

            Op::Neg(x) | Op::Abs(x) | Op::Sign(x) => {
                let arg0 = get(x);

                if !arg0.ty.is_numeric() {
                    panic!("type check");
                }

                OpInfo {
                    addr,
                    value,
                    ty: arg0.ty,
                    dynamic: arg0.dynamic,
                    dependencies: vec![x],
                    dependents: vec![],
                }
            }

            Op::Atan2(x, y) | Op::Pow(x, y) => {
                let arg0 = get(x);
                let arg1 = get(y);

                if arg0.ty != arg1.ty || !matches!(arg0.ty, F1 | F2 | F3 | F4) {
                    panic!("type check");
                }

                OpInfo {
                    addr,
                    value,
                    ty: arg0.ty,
                    dynamic: arg0.dynamic & arg1.dynamic,
                    dependencies: vec![x, y],
                    dependents: vec![],
                }
            }

            Op::Dot(x, y) => {
                let arg0 = get(x);
                let arg1 = get(y);

                if arg0.ty != arg1.ty || !matches!(arg0.ty, F2 | F3 | F4) {
                    panic!("type check");
                }

                OpInfo {
                    addr,
                    value,
                    ty: F1,
                    dynamic: arg0.dynamic & arg1.dynamic,
                    dependencies: vec![x, y],
                    dependents: vec![],
                }
            }

            Op::Cross(x, y) => {
                let arg0 = get(x);
                let arg1 = get(y);

                if arg0.ty != arg1.ty || arg0.ty != F3 {
                    panic!("type check");
                }

                OpInfo {
                    addr,
                    value,
                    ty: F3,
                    dynamic: arg0.dynamic & arg1.dynamic,
                    dependencies: vec![x, y],
                    dependents: vec![],
                }
            }

            Op::Clamp(x, y, z) => {
                let arg0 = get(x);
                let arg1 = get(y);
                let arg2 = get(z);

                if arg0.ty != arg1.ty || arg0.ty != arg2.ty || !arg0.ty.is_numeric() {
                    panic!("type check");
                }

                OpInfo {
                    addr,
                    value,
                    ty: arg0.ty,
                    dynamic: arg0.dynamic & arg1.dynamic & arg2.dynamic,
                    dependencies: vec![x, y, z],
                    dependents: vec![],
                }
            }

            Op::Lerp(x, y, z) => {
                let arg0 = get(x);
                let arg1 = get(y);
                let arg2 = get(z);

                if arg0.ty != arg1.ty || arg1.ty != arg2.ty {
                    panic!("type check");
                }

                OpInfo {
                    addr,
                    value,
                    ty: arg0.ty,
                    dynamic: arg0.dynamic & arg1.dynamic & arg2.dynamic,
                    dependencies: vec![x, y, z],
                    dependents: vec![],
                }
            }

            Op::Select(x, y, z) => {
                let arg0 = get(x);
                let arg1 = get(y);
                let arg2 = get(z);

                if arg0.ty != Boolean || arg1.ty != arg2.ty {
                    panic!("type check");
                }

                OpInfo {
                    addr,
                    value,
                    ty: arg1.ty,
                    dynamic: arg0.dynamic & arg1.dynamic & arg2.dynamic,
                    dependencies: vec![x, y, z],
                    dependents: vec![],
                }
            }

            Op::Smoothstep(x, y, z) => {
                let arg0 = get(x);
                let arg1 = get(y);
                let arg2 = get(z);

                if arg0.ty != arg1.ty || arg0.ty != arg2.ty || !matches!(arg0.ty, F1 | F2 | F3 | F4) {
                    panic!("type check");
                }

                OpInfo {
                    addr,
                    value,
                    ty: arg0.ty,
                    dynamic: arg0.dynamic & arg1.dynamic & arg2.dynamic,
                    dependencies: vec![x, y, z],
                    dependents: vec![],
                }
            }

            Op::Eq(x, y) | Op::Ne(x, y) | Op::Lt(x, y) | Op::Le(x, y) | Op::Gt(x, y) | Op::Ge(x, y) => {
                let arg0 = get(x);
                let arg1 = get(y);

                if arg0.ty != arg1.ty || !matches!(arg0.ty, Boolean | I1 | F1) {
                    panic!("type check");
                }

                OpInfo {
                    addr,
                    value,
                    ty: Boolean,
                    dynamic: arg0.dynamic & arg1.dynamic,
                    dependencies: vec![x, y],
                    dependents: vec![],
                }
            }

            Op::And(x, y) | Op::Or(x, y) | Op::Xor(x, y) => {
                let arg0 = get(x);
                let arg1 = get(y);

                if arg0.ty != arg1.ty || !matches!(arg0.ty, Boolean | I1 | I2 | I3 | I4) {
                    panic!("type check");
                }

                OpInfo {
                    addr,
                    value,
                    ty: arg0.ty,
                    dynamic: arg0.dynamic & arg1.dynamic,
                    dependencies: vec![x, y],
                    dependents: vec![],
                }
            }

            Op::Not(x) => {
                let arg0 = get(x);

                if !matches!(arg0.ty, Boolean) {
                    panic!("type check");
                }

                OpInfo {
                    addr,
                    value,
                    ty: arg0.ty,
                    dynamic: arg0.dynamic,
                    dependencies: vec![x],
                    dependents: vec![],
                }
            }

            Op::Vec2(x, y) => {
                let arg0 = get(x);
                let arg1 = get(y);

                let ty = match arg0.ty {
                    I1 => I2,
                    F1 => F2,
                    _ => panic!("type check"),
                };

                if arg0.ty != arg1.ty {
                    panic!("type check");
                }

                OpInfo {
                    addr,
                    value,
                    ty,
                    dynamic: arg0.dynamic & arg1.dynamic,
                    dependencies: vec![x, y],
                    dependents: vec![],
                }
            }

            Op::Vec3(x, y, z) => {
                let arg0 = get(x);
                let arg1 = get(y);
                let arg2 = get(z);

                let ty = match arg0.ty {
                    I1 => I3,
                    F1 => F3,
                    _ => panic!("type check"),
                };

                if arg0.ty != arg1.ty || arg0.ty != arg2.ty {
                    panic!("type check");
                }

                OpInfo {
                    addr,
                    value,
                    ty,
                    dynamic: arg0.dynamic & arg1.dynamic & arg2.dynamic,
                    dependencies: vec![x, y, z],
                    dependents: vec![],
                }
            }

            Op::Vec4(x, y, z, w) => {
                let arg0 = get(x);
                let arg1 = get(y);
                let arg2 = get(z);
                let arg3 = get(w);

                let ty = match arg0.ty {
                    I1 => I4,
                    F1 => F4,
                    _ => panic!("type check"),
                };

                if arg0.ty != arg1.ty || arg0.ty != arg2.ty || arg0.ty != arg3.ty {
                    panic!("type check");
                }

                OpInfo {
                    addr,
                    value,
                    ty,
                    dynamic: arg0.dynamic & arg1.dynamic & arg2.dynamic & arg3.dynamic,
                    dependencies: vec![x, y, z, w],
                    dependents: vec![],
                }
            }

            Op::CastFloat(x) => {
                let arg0 = get(x);

                let ty = match arg0.ty {
                    I1 => F1,
                    I2 => F2,
                    I3 => F3,
                    I4 => F4,
                    _ => panic!("type check"),
                };

                OpInfo {
                    addr,
                    value,
                    ty,
                    dynamic: arg0.dynamic,
                    dependencies: vec![x],
                    dependents: vec![],
                }
            }

            Op::CastInt(x) => {
                let arg0 = get(x);

                let ty = match arg0.ty {
                    F1 => I1,
                    F2 => I2,
                    F3 => I3,
                    F4 => I4,
                    _ => panic!("type check"),
                };

                OpInfo {
                    addr,
                    value,
                    ty,
                    dynamic: arg0.dynamic,
                    dependencies: vec![x],
                    dependents: vec![],
                }
            }

            Op::Splat2(x) => {
                let arg0 = get(x);

                let ty = match arg0.ty {
                    F1 => F2,
                    I1 => I2,
                    _ => panic!("type check"),
                };

                OpInfo {
                    addr,
                    value,
                    ty,
                    dynamic: arg0.dynamic,
                    dependencies: vec![x],
                    dependents: vec![],
                }
            }

            Op::Splat3(x) => {
                let arg0 = get(x);

                let ty = match arg0.ty {
                    F1 => F3,
                    I1 => I3,
                    _ => panic!("type check"),
                };

                OpInfo {
                    addr,
                    value,
                    ty,
                    dynamic: arg0.dynamic,
                    dependencies: vec![x],
                    dependents: vec![],
                }
            }

            Op::Splat4(x) => {
                let arg0 = get(x);

                let ty = match arg0.ty {
                    F1 => F4,
                    I1 => I4,
                    _ => panic!("type check"),
                };

                OpInfo {
                    addr,
                    value,
                    ty,
                    dynamic: arg0.dynamic,
                    dependencies: vec![x],
                    dependents: vec![],
                }
            }

            Op::Swizzle1(x, _) => {
                let arg0 = get(x);

                let ty = match arg0.ty {
                    F1 | F2 | F3 | F4 => F1,
                    I1 | I2 | I3 | I4 => I1,
                    _ => panic!("type check"),
                };

                OpInfo {
                    addr,
                    value,
                    ty,
                    dynamic: arg0.dynamic,
                    dependencies: vec![x],
                    dependents: vec![],
                }
            }

            Op::Swizzle2(x, _) => {
                let arg0 = get(x);

                let ty = match arg0.ty {
                    F2 | F3 | F4 => F2,
                    I2 | I3 | I4 => I2,
                    _ => panic!("type check"),
                };

                OpInfo {
                    addr,
                    value,
                    ty,
                    dynamic: arg0.dynamic,
                    dependencies: vec![x],
                    dependents: vec![],
                }
            }

            Op::Swizzle3(x, _) => {
                let arg0 = get(x);

                let ty = match arg0.ty {
                    F3 | F4 => F3,
                    I3 | I4 => I3,
                    _ => panic!("type check"),
                };

                OpInfo {
                    addr,
                    value,
                    ty,
                    dynamic: arg0.dynamic,
                    dependencies: vec![x],
                    dependents: vec![],
                }
            }

            Op::Swizzle4(x, _) => {
                let arg0 = get(x);

                let ty = match arg0.ty {
                    F4 => F4,
                    I4 => I4,
                    _ => panic!("type check"),
                };

                OpInfo {
                    addr,
                    value,
                    ty,
                    dynamic: arg0.dynamic,
                    dependencies: vec![x],
                    dependents: vec![],
                }
            }

            Op::Length(x) => {
                let arg0 = get(x);

                if !matches!(arg0.ty, F1 | F2 | F3 | F4) {
                    panic!("type check");
                }

                OpInfo {
                    addr,
                    value,
                    ty: F1,
                    dynamic: arg0.dynamic,
                    dependencies: vec![x],
                    dependents: vec![],
                }
            }
            Op::TextureLinear(x, y) | Op::TextureNearest(x, y) => {
                let arg0 = get(x);
                let arg1 = get(y);

                if !matches!(arg0.ty, TextureRender | TextureStatic) {
                    panic!("type check");
                }

                if !matches!(arg1.ty, F2 | I2) {
                    panic!("type check");
                }

                OpInfo {
                    addr,
                    value,
                    ty: F4,
                    dynamic: arg1.dynamic & PerFrame,
                    dependencies: vec![x, y],
                    dependents: vec![],
                }
            }
            Op::TextureSize(x) => {
                let arg0 = get(x);

                if !matches!(arg0.ty, TextureRender | TextureStatic) {
                    panic!("type check");
                }

                OpInfo {
                    addr,
                    value,
                    ty: I2,
                    dynamic: PerFrame,
                    dependencies: vec![x],
                    dependents: vec![],
                }
            }
        }
    }
}

impl BitAnd for OpDynamic {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::PerPixel, _) | (_, Self::PerPixel) => Self::PerPixel,
            (Self::PerObject, _) | (_, Self::PerObject) => Self::PerObject,
            (Self::PerFrame, _) | (_, Self::PerFrame) => Self::PerFrame,
            _ => Self::Const,
        }
    }
}

impl From<f32> for Float {
    fn from(value: f32) -> Self {
        Float(value.to_bits())
    }
}

impl From<Float> for f32 {
    fn from(value: Float) -> Self {
        f32::from_bits(value.0)
    }
}
