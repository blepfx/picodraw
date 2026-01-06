use crate::{TextureChannel, TextureFilter};
use std::{fmt::Debug, sync::Arc};

/// An operand representing a float value in the shader.
pub type OpFloat = u32;

/// An operand representing an int32 value in the shader.
pub type OpInt32 = u32;

/// An operand representing a boolean value in the shader.
pub type OpBool = u32;

/// An operand representing a texture ID in the shader.
pub type OpTex = u32;

/// A shader data. Shaders are represented by a directed acyclic graph of pure
/// (no side effects) instructions that outputs a color (as a `float4`).
///
/// Internally, the shader is represented as a "flat" AST (a single list of operations),
/// where each operation can reference previous operations as dependencies.
#[derive(Clone)]
pub struct ShaderData {
    nodes: Arc<[ShaderOp]>,
    output: [OpFloat; 4],
}

/// A builder for constructing [`ShaderData`] graphs.
///
/// Example:
/// ```rust
/// use picodraw_core::{ShaderBuilder, ShaderOp};
/// let mut builder = ShaderBuilder::new();
/// let x = builder.add(ShaderOp::PosX);
/// let y = builder.add(ShaderOp::PosY);
/// let sum = builder.add(ShaderOp::FAdd(x, y))
/// let diff = builder.add(ShaderOp::FSub(x, y));
/// let one = builder.add(ShaderOp::FLit(1.0));
/// let shader = builder.finish([sum, diff, sum, one]);
/// ```
#[derive(Default, Clone)]
pub struct ShaderBuilder {
    nodes: Vec<ShaderOp>,
}

/// A single operation in a shader graph.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ShaderOp {
    /// Float constant. `() -> float`
    FLit(f32),

    /// Integer constant. `() -> int32`
    ILit(i32),

    /// Boolean constant. `() -> bool`
    BLit(bool),

    /// Read a float value at from the given index (offset = 4 * index)
    ///
    /// `() -> float`
    ReadF32(u32),

    /// Read an int32 value at from the given index (offset = 4 * index)
    ///
    /// `() -> int32`
    ReadI32(u32),

    /// Read a u16 value at from the given index (offset = 2 * index)
    ///
    /// `() -> int32`
    ReadU16(u32),

    /// Read a u8 value at from the given index (offset = 1 * index)
    ///
    /// `() -> int32`
    ReadU8(u32),

    /// Pixel X position. `() -> float`
    PosX,

    /// Pixel Y position. `() -> float`
    PosY,

    /// Render target width. `() -> float`
    ResX,

    /// Render target height. `() -> float`
    ResY,

    /// Bottom edge Y coordinate of the current quad.
    ///
    ///  `() -> float`
    QuadB,

    /// Left edge X coordinate of the current quad.
    ///
    ///  `() -> float`
    QuadL,

    /// Top edge Y coordinate of the current quad.
    ///
    /// `() -> float`
    QuadT,

    /// Right edge X coordinate of the current quad.
    ///
    /// `() -> float`
    QuadR,

    /// a + b. `float, float -> float`
    FAdd(OpFloat, OpFloat),

    /// a - b. `float, float -> float`
    FSub(OpFloat, OpFloat),

    /// a * b. `float, float -> float`
    FMul(OpFloat, OpFloat),

    /// a / b. `float, float -> float`
    FDiv(OpFloat, OpFloat),

    /// a % b. Always returns a positive value. `float, float -> float`
    FMod(OpFloat, OpFloat),

    /// a if a < b else b. `float, float -> float`
    FMin(OpFloat, OpFloat),

    /// a if a > b else b. `float, float -> float`
    FMax(OpFloat, OpFloat),

    /// -x. `float -> float`
    FNeg(OpFloat),

    /// abs(x). `float -> float`
    FAbs(OpFloat),

    /// 1.0 if x > 0.0, 0 if x == 0.0, -1.0 if x < 0.0. `float -> float`
    FSign(OpFloat),

    /// a + b. Wraps on overflow. `int32, int32 -> int32`
    IAdd(OpInt32, OpInt32),

    /// a - b. Wraps on overflow. `int32, int32 -> int32`
    ISub(OpInt32, OpInt32),

    /// a * b. Wraps on overflow. `int32, int32 -> int32`
    IMul(OpInt32, OpInt32),

    /// a / b. `int32, int32 -> int32`
    IDiv(OpInt32, OpInt32),

    /// a % b. Always returns a positive value. `int32, int32 -> int32`
    IMod(OpInt32, OpInt32),

    /// a if a < b else b. `int32, int32 -> int32`
    IMin(OpInt32, OpInt32),

    /// a if a > b else b. `int32, int32 -> int32`
    IMax(OpInt32, OpInt32),

    /// -x. `int32 -> int32`
    INeg(OpInt32),

    /// abs(x). `int32 -> int32`
    IAbs(OpInt32),

    /// 1 if x > 0, 0 if x == 0, -1 if x < 0. `int32 -> int32`
    ISign(OpInt32),

    /// sin(x). `float -> float`
    Sin(OpFloat),

    /// cos(x). `float -> float`
    Cos(OpFloat),

    /// tan(x). `float -> float`
    Tan(OpFloat),

    /// asin(x). `float -> float`
    Asin(OpFloat),

    /// acos(x). `float -> float`
    Acos(OpFloat),

    /// atan(x). `float -> float`
    Atan(OpFloat),

    /// atan2(y, x). `float, float -> float`
    Atan2(OpFloat, OpFloat),

    /// pow(x, y). `float, float -> float`
    Pow(OpFloat, OpFloat),

    /// sqrt(x). `float -> float`
    Sqrt(OpFloat),

    /// ln(x). `float -> float`
    Ln(OpFloat),

    /// exp(x). `float -> float`
    Exp(OpFloat),

    /// `float -> float`
    Floor(OpFloat),

    /// Linear interpolation (`a + (b - a) * t`).
    ///
    /// `float (t), float (a), float (b) -> float`
    Lerp(OpFloat, OpFloat, OpFloat),

    /// Screen-space derivative in X direction.
    ///
    /// `float -> float`
    DerivX(OpFloat),

    /// Screen-space derivative in Y direction.
    ///
    /// `float -> float`
    DerivY(OpFloat),

    /// a | b. `int32, int32 -> int32`
    IOr(OpInt32, OpInt32),

    /// a & b. `int32, int32 -> int32`
    IAnd(OpInt32, OpInt32),

    /// a ^ b. `int32, int32 -> int32`
    IXor(OpInt32, OpInt32),

    /// a << b. `int32, int32 -> int32`
    IShl(OpInt32, OpInt32),

    /// a >> b. `int32, int32 -> int32`
    IShr(OpInt32, OpInt32),

    /// !a. `int32 -> int32`
    INot(OpInt32),

    /// a | b. `bool, bool -> bool`
    BOr(OpBool, OpBool),

    /// a & b. `bool, bool -> bool`
    BAnd(OpBool, OpBool),

    /// a ^ b. `bool, bool -> bool`
    BXor(OpBool, OpBool),

    /// !a. `bool -> bool`
    BNot(OpBool),

    /// a == b. `int32, int32 -> bool`
    IEq(OpInt32, OpInt32),

    /// a != b. `int32, int32 -> bool`
    INe(OpInt32, OpInt32),

    /// a < b. `int32, int32 -> bool`
    ILt(OpInt32, OpInt32),

    /// a <= b. `int32, int32 -> bool`
    ILe(OpInt32, OpInt32),

    /// a > b. `int32, int32 -> bool`
    IGt(OpInt32, OpInt32),

    /// a >= b. `int32, int32 -> bool`
    IGe(OpInt32, OpInt32),

    /// a == b. `float, float -> bool`
    FEq(OpFloat, OpFloat),

    /// a != b. `float, float -> bool`
    FNe(OpFloat, OpFloat),

    /// a < b. `float, float -> bool`
    FLt(OpFloat, OpFloat),

    /// a <= b. `float, float -> bool`
    FLe(OpFloat, OpFloat),

    /// a > b. `float, float -> bool`
    FGt(OpFloat, OpFloat),

    /// a >= b. `float, float -> bool`
    FGe(OpFloat, OpFloat),

    /// Chooses a value based on the boolean selector.
    /// It is allowed for backends to not evaluate both branches.
    ///
    /// `bool, int32, int32 -> int32`
    ISelect(OpBool, OpInt32, OpInt32),

    /// See [`ShaderOp::ISelect`]. `bool, float, float -> float`
    FSelect(OpBool, OpFloat, OpFloat),

    /// See [`ShaderOp::ISelect`]. `bool, bool, bool -> bool`
    BSelect(OpBool, OpBool, OpBool),

    /// `int32 -> float`
    FCastInt32(OpInt32),

    /// `float -> int32`
    ICastFloat(OpFloat),

    /// Sample a texture. Coordinates are in pixels.
    ///
    /// `texture, x, y, filter, channel -> float`
    TexSample(OpTex, OpFloat, OpFloat, TextureFilter, TextureChannel),

    /// Texture width. `texture -> int32`
    TexW(OpTex),

    /// Texture height. `texture -> int32`
    TexH(OpTex),
}

/// The type of value produced by [`ShaderOp`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ShaderOpType {
    /// A 32-bit floating point value.
    Float,

    /// A 32-bit integer value.
    Int32,

    /// A single-bit true or false value.
    Bool,
}

impl ShaderBuilder {
    /// Create a new empty shader builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a new operation to the shader graph, returning its operand ID that can be used as input to other operations.
    ///
    /// # Panics
    /// This function will panic if any of the dependencies of the provided operation
    /// do not exist in the current graph, or if the type checks for all dependencies fail.
    pub fn add(&mut self, op: ShaderOp) -> u32 {
        op.visit_deps(|index, ty| {
            let expected_ty = self.nodes.get(index as usize).map(|op| op.output_type());
            assert!(
                expected_ty == Some(ty),
                "shader op dependency type mismatch: expected {:?}, got {:?}",
                expected_ty,
                ty
            );
        });

        let id = self.nodes.len() as u32;
        self.nodes.push(op);
        id
    }

    /// Get the operation at the given index.
    ///
    /// # Panics
    /// This function will panic if the index is out of bounds.
    pub fn get(&self, index: u32) -> ShaderOp {
        self.nodes
            .get(index as usize)
            .copied()
            .expect("shader op index out of bounds")
    }

    /// Finish building the shader, returning the constructed [`ShaderData`].
    /// The provided output operands will be used as the RGBA output of the shader.
    ///
    /// # Panics
    /// This function will panic if any of the provided output operands do not exist in the current graph,
    /// or if they are not of type `float`.
    pub fn finish(self, output: [OpFloat; 4]) -> ShaderData {
        for output in &output {
            assert!(
                self.nodes.get(*output as usize).map(|op| op.output_type()) == Some(ShaderOpType::Float),
                "shader output must be of type float"
            );
        }

        ShaderData {
            nodes: Arc::from(self.nodes),
            output,
        }
    }
}

impl ShaderData {
    /// Total number of operations this shader has.
    pub fn len(&self) -> u32 {
        self.nodes.len() as u32
    }

    /// Check if this shader has no operations.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Get the output operands of this shader as RGBA float values.
    pub fn output(&self) -> [OpFloat; 4] {
        self.output
    }

    /// Get the operation at the given index, if it exists.
    ///
    /// # Panics
    /// This function will panic if the index is out of bounds.
    pub fn get(&self, index: u32) -> ShaderOp {
        self.nodes
            .get(index as usize)
            .copied()
            .expect("shader op index out of bounds")
    }

    /// Iterate over all operations in this shader, yielding their index and the operation itself.
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = (u32, ShaderOp)> + ExactSizeIterator + '_ {
        self.nodes.iter().copied().enumerate().map(|(i, op)| (i as u32, op))
    }
}

impl ShaderOp {
    /// Visit all dependencies of this shader operation.
    pub fn visit_deps(self, mut dep: impl FnMut(u32, ShaderOpType)) {
        self.try_visit_deps::<()>(|arg, ty| {
            dep(arg, ty);
            Ok(())
        })
        .ok();
    }

    /// Visit all dependencies of this shader operation, allowing the visitor to return an error.
    pub fn try_visit_deps<E>(self, mut dep: impl FnMut(u32, ShaderOpType) -> Result<(), E>) -> Result<(), E> {
        match self {
            ShaderOp::FLit(_)
            | ShaderOp::ILit(_)
            | ShaderOp::BLit(_)
            | ShaderOp::ReadF32(_)
            | ShaderOp::ReadI32(_)
            | ShaderOp::ReadU16(_)
            | ShaderOp::ReadU8(_)
            | ShaderOp::PosX
            | ShaderOp::PosY
            | ShaderOp::ResX
            | ShaderOp::ResY
            | ShaderOp::QuadB
            | ShaderOp::QuadL
            | ShaderOp::QuadT
            | ShaderOp::QuadR
            | ShaderOp::TexW(_)
            | ShaderOp::TexH(_) => Ok(()),

            ShaderOp::FAdd(a, b)
            | ShaderOp::FSub(a, b)
            | ShaderOp::FMul(a, b)
            | ShaderOp::FDiv(a, b)
            | ShaderOp::FMod(a, b)
            | ShaderOp::FMin(a, b)
            | ShaderOp::FMax(a, b)
            | ShaderOp::Atan2(a, b)
            | ShaderOp::Pow(a, b)
            | ShaderOp::FEq(a, b)
            | ShaderOp::FNe(a, b)
            | ShaderOp::FLt(a, b)
            | ShaderOp::FLe(a, b)
            | ShaderOp::FGt(a, b)
            | ShaderOp::FGe(a, b) => {
                dep(a, ShaderOpType::Float)?;
                dep(b, ShaderOpType::Float)?;
                Ok(())
            }

            ShaderOp::IAdd(a, b)
            | ShaderOp::ISub(a, b)
            | ShaderOp::IMul(a, b)
            | ShaderOp::IDiv(a, b)
            | ShaderOp::IMod(a, b)
            | ShaderOp::IMin(a, b)
            | ShaderOp::IMax(a, b)
            | ShaderOp::IOr(a, b)
            | ShaderOp::IAnd(a, b)
            | ShaderOp::IXor(a, b)
            | ShaderOp::IShl(a, b)
            | ShaderOp::IShr(a, b)
            | ShaderOp::IEq(a, b)
            | ShaderOp::INe(a, b)
            | ShaderOp::ILt(a, b)
            | ShaderOp::ILe(a, b)
            | ShaderOp::IGt(a, b)
            | ShaderOp::IGe(a, b) => {
                dep(a, ShaderOpType::Int32)?;
                dep(b, ShaderOpType::Int32)?;
                Ok(())
            }

            ShaderOp::FNeg(a)
            | ShaderOp::FAbs(a)
            | ShaderOp::FSign(a)
            | ShaderOp::Sin(a)
            | ShaderOp::Cos(a)
            | ShaderOp::Tan(a)
            | ShaderOp::Asin(a)
            | ShaderOp::Acos(a)
            | ShaderOp::Atan(a)
            | ShaderOp::Sqrt(a)
            | ShaderOp::Ln(a)
            | ShaderOp::Exp(a)
            | ShaderOp::Floor(a)
            | ShaderOp::DerivX(a)
            | ShaderOp::DerivY(a)
            | ShaderOp::ICastFloat(a) => {
                dep(a, ShaderOpType::Float)?;
                Ok(())
            }

            ShaderOp::INot(a)
            | ShaderOp::INeg(a)
            | ShaderOp::IAbs(a)
            | ShaderOp::ISign(a)
            | ShaderOp::FCastInt32(a) => {
                dep(a, ShaderOpType::Int32)?;
                Ok(())
            }

            ShaderOp::BOr(a, b) | ShaderOp::BAnd(a, b) | ShaderOp::BXor(a, b) => {
                dep(a, ShaderOpType::Bool)?;
                dep(b, ShaderOpType::Bool)?;
                Ok(())
            }

            ShaderOp::BNot(a) => {
                dep(a, ShaderOpType::Bool)?;
                Ok(())
            }

            ShaderOp::Lerp(a, b, c) => {
                dep(a, ShaderOpType::Float)?;
                dep(b, ShaderOpType::Float)?;
                dep(c, ShaderOpType::Float)?;
                Ok(())
            }

            ShaderOp::FSelect(a, b, c) => {
                dep(a, ShaderOpType::Bool)?;
                dep(b, ShaderOpType::Float)?;
                dep(c, ShaderOpType::Float)?;
                Ok(())
            }
            ShaderOp::ISelect(a, b, c) => {
                dep(a, ShaderOpType::Bool)?;
                dep(b, ShaderOpType::Int32)?;
                dep(c, ShaderOpType::Int32)?;
                Ok(())
            }
            ShaderOp::BSelect(a, b, c) => {
                dep(a, ShaderOpType::Bool)?;
                dep(b, ShaderOpType::Bool)?;
                dep(c, ShaderOpType::Bool)?;
                Ok(())
            }

            ShaderOp::TexSample(_, a, b, _, _) => {
                dep(a, ShaderOpType::Float)?;
                dep(b, ShaderOpType::Float)?;
                Ok(())
            }
        }
    }

    /// The output type of this shader operation.
    pub fn output_type(self) -> ShaderOpType {
        use ShaderOp::*;

        match self {
            PosX
            | PosY
            | ResX
            | ResY
            | QuadB
            | QuadL
            | QuadT
            | QuadR
            | ReadF32(_)
            | FLit(_)
            | FAdd(_, _)
            | FSub(_, _)
            | FMul(_, _)
            | FDiv(_, _)
            | FMod(_, _)
            | FMin(_, _)
            | FMax(_, _)
            | FNeg(_)
            | FAbs(_)
            | FSign(_)
            | Sin(_)
            | Cos(_)
            | Tan(_)
            | Asin(_)
            | Acos(_)
            | Atan(_)
            | Atan2(_, _)
            | Pow(_, _)
            | Sqrt(_)
            | Ln(_)
            | Exp(_)
            | Floor(_)
            | Lerp(_, _, _)
            | DerivX(_)
            | DerivY(_)
            | FSelect(_, _, _)
            | FCastInt32(_)
            | TexSample(_, _, _, _, _) => ShaderOpType::Float,

            ReadI32(_)
            | ReadU16(_)
            | ReadU8(_)
            | ILit(_)
            | IAdd(_, _)
            | ISub(_, _)
            | IMul(_, _)
            | IDiv(_, _)
            | IMod(_, _)
            | IMin(_, _)
            | IMax(_, _)
            | INeg(_)
            | IAbs(_)
            | ISign(_)
            | IOr(_, _)
            | IAnd(_, _)
            | IXor(_, _)
            | IShl(_, _)
            | IShr(_, _)
            | INot(_)
            | ISelect(_, _, _)
            | ICastFloat(_)
            | TexH(_)
            | TexW(_) => ShaderOpType::Int32,

            BLit(_)
            | BAnd(_, _)
            | BOr(_, _)
            | BXor(_, _)
            | BNot(_)
            | IEq(_, _)
            | INe(_, _)
            | ILt(_, _)
            | ILe(_, _)
            | IGt(_, _)
            | IGe(_, _)
            | FEq(_, _)
            | FNe(_, _)
            | FLt(_, _)
            | FLe(_, _)
            | FGt(_, _)
            | FGe(_, _)
            | BSelect(_, _, _) => ShaderOpType::Bool,
        }
    }
}

impl Debug for ShaderData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "ShaderData {{")?;
        for (i, op) in self.iter() {
            writeln!(f, "   ${} := {:?}", i, op)?;
        }
        for out in &self.output {
            writeln!(f, "   Output({})", out)?;
        }
        writeln!(f, "}}")
    }
}
