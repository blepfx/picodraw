use super::serialize::ShaderDataLayout;
use picodraw_core::graph::*;
use std::{
    collections::{HashMap, VecDeque},
    fmt::{self, Write},
};

const VERTEX_SHADER: &str = r#"
precision highp float;
uniform int uBufferOffsetInstance;
uniform int uBufferOffsetData;
uniform usamplerBuffer uBuffer;
uniform vec2 uResolution;
uniform bool uScreenTarget;
flat out int fragType;
flat out int fragData;
flat out vec4 fragBounds;
out vec2 fragPosition;
void main() {
    int triangleId = gl_VertexID / 3;
    int vertexId = gl_VertexID % 3;
    int quadId = triangleId >> 1;
    int cornerId = (triangleId & 1) + vertexId;
    uvec4 packedData = texelFetch(uBuffer, uBufferOffsetInstance + quadId);
    vec2 topLeft = vec2(float(packedData.x & 65535u), float((packedData.x >> 16) & 65535u));
    vec2 bottomRight = vec2(float(packedData.y & 65535u), float((packedData.y >> 16) & 65535u));
    vec2 pos = vec2(float(cornerId >> 1), float(cornerId & 1)) * (bottomRight - topLeft) + topLeft;
    gl_Position = vec4((2 * pos / uResolution - 1) * vec2(1, uScreenTarget ? -1 : 1), 0, 1);
    fragPosition = pos;
    fragBounds = vec4(topLeft, bottomRight);
    fragType = int(packedData.z);
    fragData = uBufferOffsetData + int(packedData.w);    
}"#;

const FRAGMENT_SHADER_HEADER: &str = r#"
precision highp float;
uniform usamplerBuffer uBuffer;
uniform vec2 uResolution;
flat in int fragType;
flat in int fragData;
flat in vec4 fragBounds;
in vec2 fragPosition;
out vec4 outColor;

int u2i(uint x,uint m){return int(x)-int((x&m)<<1);}
float u2f(uint x){return uintBitsToFloat(x);}
vec4 txl(in sampler2D s,vec2 i){return textureLod(s,i/textureSize(s,0),0);}
vec4 txn(in sampler2D s,vec2 i){return texelFetch(s,ivec2(i),0);}
uvec4 data(int i){return texelFetch(uBuffer,fragData+i);}

void main(){
"#;

pub struct CodegenOptions {
    pub glsl_version: u32,
    pub texture_samplers: u32,
}

pub fn generate_vertex_shader(options: &CodegenOptions) -> String {
    let mut buffer = String::new();
    emit_version_header(&mut buffer, options.glsl_version);
    buffer.push_str(VERTEX_SHADER);
    buffer
}

pub struct FragmentCodegen {
    buffer: String,

    graph_first: bool,
    graph_atoms: HashMap<OpAddr, (String, OpType)>,
    graph_inputs: VecDeque<u32>,
    graph_textures: VecDeque<u32>,
}

impl FragmentCodegen {
    pub fn new(options: &CodegenOptions) -> Self {
        Self {
            buffer: {
                let mut buffer = String::new();
                emit_version_header(&mut buffer, options.glsl_version);
                emit_texture_samplers(&mut buffer, options.texture_samplers);
                buffer.push_str(FRAGMENT_SHADER_HEADER);
                buffer
            },
            graph_first: true,
            graph_atoms: HashMap::new(),
            graph_inputs: VecDeque::new(),
            graph_textures: VecDeque::new(),
        }
    }

    pub fn begin_graph(&mut self, layout: &ShaderDataLayout) {
        if !self.graph_first {
            write!(&mut self.buffer, "else ").ok();
        }

        write!(&mut self.buffer, "if(fragType == {}){{", layout.branch_id).ok();

        for (offset, _) in layout.fields.iter() {
            self.graph_inputs.push_back(*offset);
        }

        for index in layout.textures.clone() {
            self.graph_textures.push_back(index);
        }

        for i in 0..layout.size.div_ceil(16) {
            write!(&mut self.buffer, "uvec4 _i{:x}=data({});", i, i).ok();
        }
    }

    pub fn put_atom(&mut self, info: OpInfo) {
        let inline = match info.value {
            Op::Output(_) => false,
            Op::LiteralFloat(_) => true,
            Op::LiteralInt(_) => true,
            Op::LiteralBool(_) => true,
            Op::Input(OpInput::TextureRender) => true,
            Op::Input(OpInput::TextureStatic) => true,
            _ => info.dependents.len() <= 1,
        };

        if inline {
            let result = self.render_atom(info.ty, info.value);
            self.graph_atoms.insert(info.addr, (result, info.ty));
        } else {
            let ident = self.render_ident(info.addr);
            let result = self.render_atom(info.ty, info.value);
            let typestr = self.render_type(info.ty);

            write!(&mut self.buffer, "{} {}={};", typestr, ident, result).ok();
            self.graph_atoms.insert(info.addr, (ident, info.ty));
        }
    }

    pub fn end_graph(&mut self) {
        self.graph_first = false;
        self.graph_atoms.clear();
        self.graph_inputs.clear();
        self.buffer.push('}');
    }

    pub fn finish(mut self) -> String {
        if self.graph_first {
            self.buffer.push_str("outColor=vec4(1,1,0,1);}");
        } else {
            self.buffer.push_str("else{outColor=vec4(1,1,0,1);}}");
        }

        self.buffer
    }

    fn render_ident(&mut self, id: OpAddr) -> String {
        format!("_{:x}", id.0)
    }

    fn render_type(&mut self, ty: OpType) -> &'static str {
        match ty {
            OpType::F1 => "float",
            OpType::F2 => "vec2",
            OpType::F3 => "vec3",
            OpType::F4 | OpType::Void => "vec4",
            OpType::I1 => "int",
            OpType::I2 => "ivec2",
            OpType::I3 => "ivec3",
            OpType::I4 => "ivec4",
            OpType::Boolean => "bool",
            _ => unreachable!(),
        }
    }

    fn render_atom(&mut self, ty: OpType, op: Op) -> String {
        use Op::*;
        use OpType::*;

        macro_rules! emit {
            ($lit:literal, $x:ident) => {{ format!($lit, self.graph_atoms.get(&$x).expect("codegen error").0) }};

            ($lit:literal, $x:ident, $y:ident) => {{
                format!(
                    $lit,
                    self.graph_atoms.get(&$x).expect("codegen error").0,
                    self.graph_atoms.get(&$y).expect("codegen error").0
                )
            }};

            ($lit:literal, $x:ident, $y:ident, $z:ident) => {{
                format!(
                    $lit,
                    self.graph_atoms.get(&$x).expect("codegen error").0,
                    self.graph_atoms.get(&$y).expect("codegen error").0,
                    self.graph_atoms.get(&$z).expect("codegen error").0
                )
            }};

            ($lit:literal, $x:ident, $y:ident, $z:ident, $w:ident) => {{
                format!(
                    $lit,
                    self.graph_atoms.get(&$x).expect("codegen error").0,
                    self.graph_atoms.get(&$y).expect("codegen error").0,
                    self.graph_atoms.get(&$z).expect("codegen error").0,
                    self.graph_atoms.get(&$w).expect("codegen error").0
                )
            }};
        }

        match op {
            Position => format!("fragPosition"),
            Resolution => format!("uResolution"),
            QuadStart => format!("fragBounds.xy"),
            QuadEnd => format!("fragBounds.zw"),

            Input(OpInput::F32) => {
                let offset = self.graph_inputs.pop_front().expect("codegen error");
                format!("u2f({})", self.render_input(offset, 4))
            }
            Input(OpInput::I32) => {
                let offset = self.graph_inputs.pop_front().expect("codegen error");
                format!("int({})", self.render_input(offset, 4))
            }
            Input(OpInput::I16) => {
                let offset = self.graph_inputs.pop_front().expect("codegen error");
                format!("u2i({},32768u)", self.render_input(offset, 2))
            }
            Input(OpInput::I8) => {
                let offset = self.graph_inputs.pop_front().expect("codegen error");
                format!("u2i({},256u)", self.render_input(offset, 1))
            }
            Input(OpInput::U32) => {
                let offset = self.graph_inputs.pop_front().expect("codegen error");
                format!("int({})", self.render_input(offset, 4))
            }
            Input(OpInput::U16) => {
                let offset = self.graph_inputs.pop_front().expect("codegen error");
                format!("int({})", self.render_input(offset, 2))
            }
            Input(OpInput::U8) => {
                let offset = self.graph_inputs.pop_front().expect("codegen error");
                format!("int({})", self.render_input(offset, 1))
            }
            Input(OpInput::TextureRender) | Input(OpInput::TextureStatic) => {
                let index = self.graph_textures.pop_front().expect("codegen error");
                format!("{}", index)
            }

            Output(x) => emit!("(outColor={})", x),

            LiteralFloat(x) => match f32::from(x) {
                f32::INFINITY => format!("u2f(0x7F800000u)"),
                f32::NEG_INFINITY => format!("u2f(0xFF800000u)"),
                x if x.is_nan() => format!("u2f(0xFFFFFFFFu)"),
                x if x.is_sign_positive() => format!("{:?}", x),
                x => format!("({:?})", x),
            },

            LiteralInt(x) if x >= 0 => format!("{x}"),
            LiteralInt(x) => format!("({x})"),
            LiteralBool(true) => format!("true"),
            LiteralBool(false) => format!("false"),

            Add(x, y) => emit!("({}+{})", x, y),
            Sub(x, y) => emit!("({}-{})", x, y),
            Mul(x, y) => emit!("({}*{})", x, y),
            Div(x, y) => emit!("({}/{})", x, y),
            Rem(x, y) => emit!("mod({},{})", x, y),
            Dot(x, y) => emit!("dot({},{})", x, y),
            Cross(x, y) => emit!("cross({},{})", x, y),
            Neg(x) => emit!("(-{})", x),
            Sin(x) => emit!("sin({})", x),
            Cos(x) => emit!("cos({})", x),
            Tan(x) => emit!("tan({})", x),
            Asin(x) => emit!("asin({})", x),
            Acos(x) => emit!("acos({})", x),
            Atan(x) => emit!("atan({})", x),
            Atan2(x, y) => emit!("atan({},{})", x, y),
            Sqrt(x) => emit!("sqrt({})", x),
            Pow(x, y) => emit!("pow({},{})", x, y),
            Exp(x) => emit!("exp({})", x),
            Ln(x) => emit!("log({})", x),
            Min(x, y) => emit!("min({},{})", x, y),
            Max(x, y) => emit!("max({},{})", x, y),
            Clamp(x, y, z) => emit!("clamp({},{},{})", x, y, z),
            Abs(x) => emit!("abs({})", x),
            Sign(x) => emit!("sign({})", x),
            Floor(x) => emit!("floor({})", x),
            Lerp(x, y, z) => emit!("mix({},{},{})", y, z, x),
            Select(x, y, z) => emit!("mix({},{},{})", z, y, x),
            Smoothstep(x, y, z) => emit!("smoothstep({},{},{})", y, z, x),
            Step(x, y) => emit!("step({},{})", y, x),
            Eq(x, y) => emit!("({}=={})", x, y),
            Ne(x, y) => emit!("({}!={})", x, y),
            Lt(x, y) => emit!("({}<{})", x, y),
            Le(x, y) => emit!("({}<={})", x, y),
            Gt(x, y) => emit!("({}>={})", x, y),
            Ge(x, y) => emit!("({}>{})", x, y),
            And(x, y) => emit!("({}&{})", x, y),
            Or(x, y) => emit!("({}|{})", x, y),
            Xor(x, y) => emit!("({}^{})", x, y),
            Not(x) => emit!("(!{})", x),

            Vec2(x, y) if ty == F2 => emit!("vec2({},{})", x, y),
            Vec2(x, y) if ty == I2 => emit!("ivec2({},{})", x, y),

            Vec3(x, y, z) if ty == F3 => emit!("vec3({},{},{})", x, y, z),
            Vec3(x, y, z) if ty == I3 => emit!("ivec3({},{},{})", x, y, z),

            Vec4(x, y, z, w) if ty == F4 => emit!("vec4({},{},{},{})", x, y, z, w),
            Vec4(x, y, z, w) if ty == I4 => emit!("ivec4({},{},{},{})", x, y, z, w),

            Splat2(x) if ty == F2 => emit!("vec2({})", x),
            Splat2(x) if ty == I2 => emit!("ivec2({})", x),

            Splat3(x) if ty == F3 => emit!("vec3({})", x),
            Splat3(x) if ty == I3 => emit!("ivec3({})", x),

            Splat4(x) if ty == F4 => emit!("vec4({})", x),
            Splat4(x) if ty == I4 => emit!("ivec4({})", x),

            CastFloat(x) if ty == F1 => emit!("float({})", x),
            CastFloat(x) if ty == F2 => emit!("vec2({})", x),
            CastFloat(x) if ty == F3 => emit!("vec3({})", x),
            CastFloat(x) if ty == F4 => emit!("vec4({})", x),

            CastInt(x) if ty == I1 => emit!("int({})", x),
            CastInt(x) if ty == I2 => emit!("ivec2({})", x),
            CastInt(x) if ty == I3 => emit!("ivec3({})", x),
            CastInt(x) if ty == I4 => emit!("ivec4({})", x),

            Swizzle1(x, [s0]) => format!(
                "{}.{}",
                self.graph_atoms.get(&x).expect("codegen error").0,
                SwizzleDisplay(s0)
            ),
            Swizzle2(x, [s0, s1]) => format!(
                "{}.{}{}",
                self.graph_atoms.get(&x).expect("codegen error").0,
                SwizzleDisplay(s0),
                SwizzleDisplay(s1)
            ),
            Swizzle3(x, [s0, s1, s2]) => format!(
                "{}.{}{}{}",
                self.graph_atoms.get(&x).expect("codegen error").0,
                SwizzleDisplay(s0),
                SwizzleDisplay(s1),
                SwizzleDisplay(s2)
            ),
            Swizzle4(x, [s0, s1, s2, s3]) => format!(
                "{}.{}{}{}{}",
                self.graph_atoms.get(&x).expect("codegen error").0,
                SwizzleDisplay(s0),
                SwizzleDisplay(s1),
                SwizzleDisplay(s2),
                SwizzleDisplay(s3)
            ),

            Normalize(x) if ty == F1 => emit!("sign({})", x),
            Length(x) if self.graph_atoms.get(&x).expect("codegen error").1 == F1 => {
                emit!("abs({})", x)
            }

            Normalize(x) => emit!("normalize({})", x),
            Length(x) => emit!("length({})", x),

            DerivX(x) => emit!("dFdx({})", x),
            DerivY(x) => emit!("dFdy({})", x),
            DerivWidth(x) => emit!("fwidth({})", x),

            TextureLinear(x, y) => emit!("txl(uTextures[{}],{})", x, y),
            TextureNearest(x, y) => emit!("txn(uTextures[{}],{})", x, y),
            TextureSize(x) => emit!("textureSize(uTextures[{}],0)", x),

            op => panic!("unreachable op: op={:?}; ty={:?}", op, ty),
        }
    }

    fn render_input(&mut self, offset: u32, size: u32) -> String {
        let (b16, b4, b1) = (offset >> 4, (offset >> 2) & 3, (offset & 3) << 3);
        let b4 = match b4 {
            0 => "x",
            1 => "y",
            2 => "z",
            _ => "w",
        };

        if size == 1 {
            format!("(_i{:x}.{}>>{}u)&255u", b16, b4, b1)
        } else if size == 2 {
            format!("(_i{:x}.{}>>{}u)&65535u", b16, b4, b1)
        } else if size == 4 {
            format!("_i{:x}.{}", b16, b4)
        } else {
            unreachable!()
        }
    }
}

struct SwizzleDisplay(Swizzle);
impl fmt::Display for SwizzleDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Swizzle::X => write!(f, "x"),
            Swizzle::Y => write!(f, "y"),
            Swizzle::Z => write!(f, "z"),
            Swizzle::W => write!(f, "w"),
        }
    }
}

fn emit_version_header(buffer: &mut String, version: u32) {
    if version >= 330 {
        buffer.push_str("#version 330\n");
    } else if version >= 140 {
        buffer.push_str("#version 140\n");
        buffer.push_str("#extension ARB_shader_bit_encoding : require\n");
    } else {
        buffer.push_str("#version 130\n");
        buffer.push_str("#extension ARB_shader_bit_encoding : require\n");
        buffer.push_str("#extension ARB_texture_buffer_object : enable\n");
        buffer.push_str("#extension EXT_texture_buffer : enable\n");
    }
}

fn emit_texture_samplers(buffer: &mut String, texture_samplers: u32) {
    if texture_samplers > 0 {
        writeln!(buffer, "uniform sampler2D uTextures[{}];", texture_samplers).ok();
    }
}
