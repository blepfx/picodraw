use super::{CompilerBufferMode, CompilerOptions};
use picodraw_core::{TextureFilter, graph::*};
use std::{
    collections::{HashMap, VecDeque},
    fmt::Write,
};

const VERTEX_SHADER: &str = r#"

#ifdef GL_FRAGMENT_PRECISION_HIGH
precision highp float;
precision highp int;
#endif

uniform vec2 uResolution;
uniform bool uScreenTarget;
uniform int uBufferListOffset;

flat out int fragType;
flat out int fragData;
flat out vec4 fragBounds;
out vec2 fragPosition;

void main() {
    int triangleId = gl_VertexID / 3;
    int vertexId = gl_VertexID % 3;
    int quadId = triangleId >> 1;
    int cornerId = (triangleId & 1) + vertexId;

    uvec4 packedData = ri(uBufferListOffset + quadId);
    vec2 topLeft = vec2(float(packedData.x & 65535u), float((packedData.x >> 16) & 65535u));
    vec2 bottomRight = vec2(float(packedData.y & 65535u), float((packedData.y >> 16) & 65535u));
    vec2 pos = vec2(float(cornerId >> 1), float(cornerId & 1)) * (bottomRight - topLeft) + topLeft;
    
    gl_Position = vec4((2 * pos / uResolution - 1) * vec2(1, uScreenTarget ? -1 : 1), 0, 1);
    fragPosition = pos;
    fragBounds = vec4(topLeft, bottomRight);
    fragType = int(packedData.z);
    fragData = int(packedData.w);    
}"#;

const FRAGMENT_SHADER_HEADER: &str = r#"

#ifdef GL_FRAGMENT_PRECISION_HIGH
precision highp float;
precision highp int;
#endif

uniform vec2 uResolution;
uniform int uBufferDataOffset;

flat in int fragType;
flat in int fragData;
flat in vec4 fragBounds;
in vec2 fragPosition;
out vec4 outColor;

int u2i(uint x,uint m){return int(x)-int((x&m)<<1);}
vec4 txl(in sampler2D s,vec2 i){return textureLod(s,i/textureSize(s,0),0);}
vec4 txn(in sampler2D s,vec2 i){return texelFetch(s,ivec2(i),0);}
vec4 df(int i){return rf(uBufferDataOffset+fragData+i);}
uvec4 di(int i){return ri(uBufferDataOffset+fragData+i);}
void main(){
"#;

pub fn generate_vertex_shader(options: &CompilerOptions) -> String {
    let mut buffer = String::new();
    emit_version_header(&mut buffer, options.glsl_version, options.buffer_mode);
    emit_buffer_binding(&mut buffer, options.buffer_mode);
    buffer.push_str(VERTEX_SHADER);
    buffer
}

pub struct FragmentCodegen {
    buffer: String,

    graph_first: bool,
    graph_atoms: HashMap<OpAddr, String>,
    graph_inputs: VecDeque<u32>,
    graph_textures: VecDeque<u32>,
}

impl FragmentCodegen {
    pub fn new(options: &CompilerOptions) -> Self {
        Self {
            buffer: {
                let mut buffer = String::new();
                emit_version_header(&mut buffer, options.glsl_version, options.buffer_mode);
                emit_buffer_binding(&mut buffer, options.buffer_mode);
                emit_texture_samplers(&mut buffer, options.texture_units);
                buffer.push_str(FRAGMENT_SHADER_HEADER);
                buffer
            },
            graph_first: true,
            graph_atoms: HashMap::new(),
            graph_inputs: VecDeque::new(),
            graph_textures: VecDeque::new(),
        }
    }

    pub fn emit_graph_begin(&mut self, branch_id: u32) {
        if !self.graph_first {
            write!(&mut self.buffer, "else ").ok();
        }

        write!(&mut self.buffer, "if(fragType == {}){{\n", branch_id).ok();
    }

    pub fn emit_graph_input(&mut self, offset: u32) {
        self.graph_inputs.push_back(offset);
    }

    pub fn emit_graph_texture(&mut self, index: u32) {
        self.graph_textures.push_back(index);
    }

    pub fn emit_graph_end(&mut self, graph: &Graph) {
        write!(
            &mut self.buffer,
            "outColor={};\n}}",
            self.graph_atoms.get(&graph.output()).expect("codegen error")
        )
        .ok();

        self.graph_first = false;
        self.graph_atoms.clear();
        self.graph_inputs.clear();
    }

    pub fn emit_atom(&mut self, graph: &Graph, op: OpAddr) {
        let inline = match graph.value_of(op) {
            OpValue::Literal(_) => true,
            OpValue::Position => true,
            OpValue::Resolution => true,
            OpValue::QuadStart => true,
            OpValue::QuadEnd => true,
            OpValue::Input(OpInput::TextureRender) => true,
            OpValue::Input(OpInput::TextureStatic) => true,
            _ => {
                let dependents = (graph.output() == op) as usize + graph.dependents_of(op).count();
                dependents < 2
            }
        };

        if inline {
            let result = self.emit_atom_value(graph, op);
            self.graph_atoms.insert(op, result);
        } else {
            let ident = self.emit_ident(op);
            let result = self.emit_atom_value(graph, op);
            let typestr = self.emit_type(graph.type_of(op));

            write!(&mut self.buffer, "{} {}={};\n", typestr, ident, result).ok();
            self.graph_atoms.insert(op, ident);
        }
    }

    pub fn finish(mut self) -> String {
        if self.graph_first {
            self.buffer.push_str("outColor=vec4(1,1,0,1);\n}");
        } else {
            self.buffer.push_str("else{\noutColor=vec4(1,1,0,1);\n}\n}");
        }

        self.buffer
    }

    fn emit_ident(&mut self, id: OpAddr) -> String {
        format!("_{:x}", id)
    }

    fn emit_type(&mut self, ty: OpType) -> &'static str {
        match ty {
            OpType::F1 => "float",
            OpType::F2 => "vec2",
            OpType::F3 => "vec3",
            OpType::F4 => "vec4",
            OpType::I1 => "int",
            OpType::I2 => "ivec2",
            OpType::I3 => "ivec3",
            OpType::I4 => "ivec4",
            OpType::Boolean => "bool",
            _ => unreachable!(),
        }
    }

    fn emit_atom_value(&mut self, graph: &Graph, op: OpAddr) -> String {
        use OpType::*;
        use OpValue::*;

        macro_rules! emit {
            ($lit:literal, $x:ident) => {{ format!($lit, self.graph_atoms.get(&$x).expect("codegen error")) }};

            ($lit:literal, $x:ident, $y:ident) => {{
                format!(
                    $lit,
                    self.graph_atoms.get(&$x).expect("codegen error"),
                    self.graph_atoms.get(&$y).expect("codegen error")
                )
            }};

            ($lit:literal, $x:ident, $y:ident, $z:ident) => {{
                format!(
                    $lit,
                    self.graph_atoms.get(&$x).expect("codegen error"),
                    self.graph_atoms.get(&$y).expect("codegen error"),
                    self.graph_atoms.get(&$z).expect("codegen error")
                )
            }};

            ($lit:literal, $x:ident, $y:ident, $z:ident, $w:ident) => {{
                format!(
                    $lit,
                    self.graph_atoms.get(&$x).expect("codegen error"),
                    self.graph_atoms.get(&$y).expect("codegen error"),
                    self.graph_atoms.get(&$z).expect("codegen error"),
                    self.graph_atoms.get(&$w).expect("codegen error")
                )
            }};
        }

        let ty = graph.type_of(op);
        match graph.value_of(op) {
            Position => format!("fragPosition"),
            Resolution => format!("uResolution"),
            QuadStart => format!("fragBounds.xy"),
            QuadEnd => format!("fragBounds.zw"),

            Input(OpInput::F32) => {
                let offset = self.graph_inputs.pop_front().expect("codegen error");
                self.emit_input_float(offset)
            }
            Input(OpInput::I32) => {
                let offset = self.graph_inputs.pop_front().expect("codegen error");
                format!("int({})", self.emit_input_int(offset, 4))
            }
            Input(OpInput::I16) => {
                let offset = self.graph_inputs.pop_front().expect("codegen error");
                format!("u2i({},32768u)", self.emit_input_int(offset, 2))
            }
            Input(OpInput::I8) => {
                let offset = self.graph_inputs.pop_front().expect("codegen error");
                format!("u2i({},128u)", self.emit_input_int(offset, 1))
            }
            Input(OpInput::U16) => {
                let offset = self.graph_inputs.pop_front().expect("codegen error");
                format!("int({})", self.emit_input_int(offset, 2))
            }
            Input(OpInput::U8) => {
                let offset = self.graph_inputs.pop_front().expect("codegen error");
                format!("int({})", self.emit_input_int(offset, 1))
            }
            Input(OpInput::TextureRender) | Input(OpInput::TextureStatic) => {
                let index = self.graph_textures.pop_front().expect("codegen error");
                format!("{}", index)
            }

            Literal(x) => match x {
                OpLiteral::Float(f32::INFINITY) => format!("4e+100"),
                OpLiteral::Float(f32::NEG_INFINITY) => format!("(-4e+100)"),
                OpLiteral::Float(x) if x.is_nan() => format!("(0.0/0.0)"),
                OpLiteral::Float(x) if x.is_sign_positive() => format!("{:?}", x),
                OpLiteral::Float(x) => format!("({:?})", x),
                OpLiteral::Int(x) if x >= 0 => format!("{:?}", x),
                OpLiteral::Int(x) => format!("({:?})", x),
                OpLiteral::Bool(true) => format!("true"),
                OpLiteral::Bool(false) => format!("false"),
            },

            Add(x, y) => emit!("({}+{})", x, y),
            Sub(x, y) => emit!("({}-{})", x, y),
            Mul(x, y) => emit!("({}*{})", x, y),
            Div(x, y) => emit!("({}/{})", x, y),
            Rem(x, y) => emit!("mod({},{})", x, y),

            Dot(x, y) if graph.type_of(x) == F1 => {
                emit!("({}*{})", x, y)
            }

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
            Abs(x) => emit!("abs({})", x),
            Sign(x) => emit!("sign({})", x),
            Floor(x) => emit!("floor({})", x),
            Lerp(x, y, z) => emit!("mix({},{},{})", y, z, x),
            Step(x, y) => emit!("step({},{})", y, x),
            Clamp(x, y, z) => emit!("clamp({},{},{})", x, y, z),
            Smoothstep(x, y, z) => emit!("smoothstep({},{},{})", y, z, x),

            Select(x, y, z) if ty.size() == 1 => emit!("mix({},{},{})", z, y, x),
            Select(x, y, z) if ty.size() == 2 => emit!("mix({},{},bvec2({}))", z, y, x),
            Select(x, y, z) if ty.size() == 3 => emit!("mix({},{},bvec3({}))", z, y, x),
            Select(x, y, z) if ty.size() == 4 => emit!("mix({},{},bvec4({}))", z, y, x),

            Eq(x, y) => emit!("({}=={})", x, y),
            Ne(x, y) => emit!("({}!={})", x, y),
            Lt(x, y) => emit!("({}<{})", x, y),
            Le(x, y) => emit!("({}<={})", x, y),
            Gt(x, y) => emit!("({}>{})", x, y),
            Ge(x, y) => emit!("({}>={})", x, y),
            And(x, y) => emit!("({}&{})", x, y),
            Or(x, y) => emit!("({}|{})", x, y),
            Xor(x, y) => emit!("({}^{})", x, y),
            Shl(x, y) => emit!("({}<<{})", x, y),
            Shr(x, y) => emit!("({}>>{})", x, y),

            Not(x) if ty == Boolean => emit!("(!{})", x),
            Not(x) => emit!("(~{})", x),

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

            ExtractX(x) => emit!("{}.x", x),
            ExtractY(x) => emit!("{}.y", x),
            ExtractZ(x) => emit!("{}.z", x),
            ExtractW(x) => emit!("{}.w", x),

            Normalize(x) if ty == F1 => emit!("sign({})", x),
            Length(x) if graph.type_of(x) == F1 => {
                emit!("abs({})", x)
            }

            Normalize(x) => emit!("normalize({})", x),
            Length(x) => emit!("length({})", x),

            DerivX(x) => emit!("dFdx({})", x),
            DerivY(x) => emit!("dFdy({})", x),
            DerivWidth(x) => emit!("fwidth({})", x),

            TextureSample(x, y, TextureFilter::Linear) => emit!("txl(uTextures[{}],{})", x, y),
            TextureSample(x, y, TextureFilter::Nearest) => emit!("txn(uTextures[{}],{})", x, y),
            TextureSize(x) => emit!("textureSize(uTextures[{}],0)", x),

            op => panic!("unreachable op: op={:?}; ty={:?}", op, ty),
        }
    }

    fn emit_input_int(&mut self, offset: u32, size: u32) -> String {
        let (b16, b4, b1) = (offset >> 4, (offset >> 2) & 3, (offset & 3) << 3);
        let b4 = match b4 {
            0 => "x",
            1 => "y",
            2 => "z",
            _ => "w",
        };

        match size {
            1 if b1 == 0 => format!("(di({}).{}&255u)", b16, b4),
            2 if b1 == 0 => format!("(di({}).{}&65535u)", b16, b4),
            1 => format!("(di({}).{}>>{}u)&255u", b16, b4, b1),
            2 => format!("(di({}).{}>>{}u)&65535u", b16, b4, b1),
            4 => format!("di({}).{}", b16, b4),
            _ => unreachable!(),
        }
    }

    fn emit_input_float(&mut self, offset: u32) -> String {
        let (b16, b4) = (offset >> 4, (offset >> 2) & 3);
        let b4 = match b4 {
            0 => "x",
            1 => "y",
            2 => "z",
            _ => "w",
        };

        format!("df({}).{}", b16, b4)
    }
}

fn emit_buffer_binding(buffer: &mut String, mode: CompilerBufferMode) {
    match mode {
        CompilerBufferMode::UniformBlock { size_bytes } => {
            let size = size_bytes / size_of::<[u32; 4]>() as u32;

            writeln!(
                buffer,
                "
layout(std140) uniform uBufferU32 {{uvec4 bufferU32[{}];}};
layout(std140) uniform uBufferF32 {{vec4 bufferF32[{}];}};
uvec4 ri(int i){{return bufferU32[i];}};
vec4 rf(int i){{return bufferF32[i];}};
                ",
                size, size
            )
            .ok();
        }
        CompilerBufferMode::TextureBuffer => {
            writeln!(
                buffer,
                "
uniform usamplerBuffer uBuffer;
uvec4 ri(int i){{return texelFetch(uBuffer,i);}};
vec4 rf(int i){{return uintBitsToFloat(ri(i));}};
                "
            )
            .ok();
        }
    }
}

fn emit_version_header(buffer: &mut String, version: u32, mode: CompilerBufferMode) {
    match mode {
        CompilerBufferMode::UniformBlock { .. } => {
            if version >= 140 {
                buffer.push_str("#version 140\n");
            } else {
                buffer.push_str("#version 130\n");
                buffer.push_str("#extension GL_ARB_uniform_buffer_object : require\n");
            }
        }
        CompilerBufferMode::TextureBuffer => {
            if version >= 330 {
                buffer.push_str("#version 330\n");
            } else if version >= 140 {
                buffer.push_str("#version 140\n");
                buffer.push_str("#extension GL_ARB_shader_bit_encoding : require\n");
            } else {
                buffer.push_str("#version 130\n");
                buffer.push_str("#extension GL_ARB_shader_bit_encoding : require\n");
                buffer.push_str("#extension GL_ARB_texture_buffer_object : enable\n");
                buffer.push_str("#extension GL_EXT_texture_buffer : enable\n");
            }
        }
    }
}

fn emit_texture_samplers(buffer: &mut String, texture_samplers: u32) {
    if texture_samplers > 0 {
        writeln!(buffer, "uniform sampler2D uTextures[{}];", texture_samplers).ok();
    }
}
