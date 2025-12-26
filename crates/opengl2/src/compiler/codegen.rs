use super::{
    CompilerBufferMode, CompilerOptions, CompilerShader,
    analysis::{ShaderStatement, ShaderStructure, ShaderValue},
};
use picodraw_core2::{ShaderOp, ShaderOpType};
use std::fmt::Write;

pub fn emit_shader_function(buffer: &mut String, shader: &CompilerShader, structure: &ShaderStructure) {
    write!(buffer, "void s_{:x}(){{", shader.index).ok();

    if structure.registers_float > 0 {
        buffer.push_str("float ");

        for i in 0..structure.registers_float {
            if i != 0 {
                buffer.push_str(",");
            }

            emit_shader_var_name(buffer, structure, i, ShaderOpType::Float);
        }

        buffer.push_str(";");
    }

    if structure.registers_int32 > 0 {
        buffer.push_str("int ");

        for i in 0..structure.registers_int32 {
            if i != 0 {
                buffer.push_str(",");
            }

            emit_shader_var_name(buffer, structure, i, ShaderOpType::Int32);
        }

        buffer.push_str(";");
    }

    if structure.registers_bool > 0 {
        buffer.push_str("bool ");

        for i in 0..structure.registers_bool {
            if i != 0 {
                buffer.push_str(",");
            }

            emit_shader_var_name(buffer, structure, i, ShaderOpType::Bool);
        }

        buffer.push_str(";");
    }

    for stmt in &structure.statements {
        emit_shader_statement(buffer, structure, stmt);
    }

    buffer.push_str("outColor=vec4(");
    emit_shader_var_read(buffer, structure, structure.outputs[0]);
    buffer.push_str(",");
    emit_shader_var_read(buffer, structure, structure.outputs[1]);
    buffer.push_str(",");
    emit_shader_var_read(buffer, structure, structure.outputs[2]);
    buffer.push_str(",");
    emit_shader_var_read(buffer, structure, structure.outputs[3]);
    buffer.push_str(");");

    buffer.push_str("}");
}

pub fn emit_shader_statement(buffer: &mut String, structure: &ShaderStructure, statement: &ShaderStatement) {
    match statement {
        ShaderStatement::Assign {
            register,
            value: ShaderValue::Inline(op),
        } => {
            emit_shader_var_name(buffer, structure, *register, op.output_type());
            buffer.push_str("=");
            emit_shader_op(buffer, structure, *op);
            buffer.push_str(";");
        }

        ShaderStatement::Assign {
            register,
            value: ShaderValue::Register(source, type_),
        } => {
            emit_shader_var_name(buffer, structure, *register, *type_);
            buffer.push_str("=");
            emit_shader_var_name(buffer, structure, *source, *type_);
            buffer.push_str(";");
        }

        ShaderStatement::Conditional {
            condition,
            if_true,
            if_false,
        } => {
            buffer.push_str("if(");
            emit_shader_var_read(buffer, structure, *condition);
            buffer.push_str("){");
            for stmt in if_true {
                emit_shader_statement(buffer, structure, stmt);
            }
            buffer.push_str("}");
            if if_false.len() > 0 {
                buffer.push_str("else {");
                for stmt in if_false {
                    emit_shader_statement(buffer, structure, stmt);
                }
                buffer.push_str("}");
            }
        }
    }
}

// name of var if not inline
pub fn emit_shader_var_name(buffer: &mut String, structure: &ShaderStructure, register: u32, type_: ShaderOpType) {
    write!(
        buffer,
        "_{:x}",
        match type_ {
            ShaderOpType::Float => register,
            ShaderOpType::Int32 => structure.registers_float + register,
            ShaderOpType::Bool => structure.registers_float + structure.registers_int32 + register,
        }
    )
    .ok();
}

// read from var, or inline
pub fn emit_shader_var_read(buffer: &mut String, structure: &ShaderStructure, value: u32) {
    match &structure.values[&value] {
        ShaderValue::Inline(op) => {
            emit_shader_op(buffer, structure, *op);
        }
        ShaderValue::Register(register, type_) => {
            emit_shader_var_name(buffer, structure, *register, *type_);
        }
    }
}

// operation over values, can be inlined or assigned to a var
pub fn emit_shader_op(buffer: &mut String, structure: &ShaderStructure, op: ShaderOp) {
    match op {
        ShaderOp::BLit(true) => buffer.push_str("true"),
        ShaderOp::BLit(false) => buffer.push_str("false"),

        ShaderOp::FLit(f32::INFINITY) => buffer.push_str("4.6e+18"), //2^62
        ShaderOp::FLit(f32::NEG_INFINITY) => buffer.push_str("(-4.6e+18)"),
        ShaderOp::FLit(x) if x.is_nan() => buffer.push_str("(0.0/0.0)"),

        ShaderOp::FLit(x) if x.is_sign_positive() => {
            let _ = write!(buffer, "{:?}", x);
        }
        ShaderOp::FLit(x) => {
            let _ = write!(buffer, "({:?})", x);
        }
        ShaderOp::ILit(x) if x >= 0 => {
            let _ = write!(buffer, "{:?}", x);
        }
        ShaderOp::ILit(x) => {
            let _ = write!(buffer, "({:?})", x);
        }

        ShaderOp::ReadF32(offset) => emit_shader_op_buffer_read(buffer, offset, true, 4),
        ShaderOp::ReadI32(offset) => emit_shader_op_buffer_read(buffer, offset, false, 4),
        ShaderOp::ReadU16(offset) => emit_shader_op_buffer_read(buffer, offset, false, 2),
        ShaderOp::ReadU8(offset) => emit_shader_op_buffer_read(buffer, offset, false, 1),

        ShaderOp::PosX => buffer.push_str("fragPos.x"),
        ShaderOp::PosY => buffer.push_str("fragPos.y"),
        ShaderOp::ResX => buffer.push_str("uResolution.x"),
        ShaderOp::ResY => buffer.push_str("uResolution.y"),
        ShaderOp::QuadB => buffer.push_str("fragBounds.w"),
        ShaderOp::QuadL => buffer.push_str("fragBounds.x"),
        ShaderOp::QuadT => buffer.push_str("fragBounds.y"),
        ShaderOp::QuadR => buffer.push_str("fragBounds.z"),

        ShaderOp::FAdd(left, right) | ShaderOp::IAdd(left, right) => {
            buffer.push_str("(");
            emit_shader_var_read(buffer, structure, left);
            buffer.push_str("+");
            emit_shader_var_read(buffer, structure, right);
            buffer.push_str(")");
        }

        ShaderOp::FSub(left, right) | ShaderOp::ISub(left, right) => {
            buffer.push_str("(");
            emit_shader_var_read(buffer, structure, left);
            buffer.push_str("-");
            emit_shader_var_read(buffer, structure, right);
            buffer.push_str(")");
        }

        ShaderOp::FMul(left, right) | ShaderOp::IMul(left, right) => {
            buffer.push_str("(");
            emit_shader_var_read(buffer, structure, left);
            buffer.push_str("*");
            emit_shader_var_read(buffer, structure, right);
            buffer.push_str(")");
        }

        ShaderOp::FDiv(left, right) | ShaderOp::IDiv(left, right) => {
            buffer.push_str("(");
            emit_shader_var_read(buffer, structure, left);
            buffer.push_str("/");
            emit_shader_var_read(buffer, structure, right);
            buffer.push_str(")");
        }

        ShaderOp::FMax(left, right) | ShaderOp::IMax(left, right) => {
            buffer.push_str("max(");
            emit_shader_var_read(buffer, structure, left);
            buffer.push_str(",");
            emit_shader_var_read(buffer, structure, right);
            buffer.push_str(")");
        }

        ShaderOp::FMin(left, right) | ShaderOp::IMin(left, right) => {
            buffer.push_str("min(");
            emit_shader_var_read(buffer, structure, left);
            buffer.push_str(",");
            emit_shader_var_read(buffer, structure, right);
            buffer.push_str(")");
        }

        ShaderOp::FMod(left, right) | ShaderOp::IMod(left, right) => {
            buffer.push_str("mod(");
            emit_shader_var_read(buffer, structure, left);
            buffer.push_str(",");
            emit_shader_var_read(buffer, structure, right);
            buffer.push_str(")");
        }

        ShaderOp::FNeg(arg) | ShaderOp::INeg(arg) => {
            buffer.push_str("(-");
            emit_shader_var_read(buffer, structure, arg);
            buffer.push_str(")");
        }

        ShaderOp::FAbs(arg) | ShaderOp::IAbs(arg) => {
            buffer.push_str("abs(");
            emit_shader_var_read(buffer, structure, arg);
            buffer.push_str(")");
        }

        ShaderOp::Sin(arg) => {
            buffer.push_str("sin(");
            emit_shader_var_read(buffer, structure, arg);
            buffer.push_str(")");
        }

        ShaderOp::Cos(arg) => {
            buffer.push_str("cos(");
            emit_shader_var_read(buffer, structure, arg);
            buffer.push_str(")");
        }

        ShaderOp::Tan(arg) => {
            buffer.push_str("tan(");
            emit_shader_var_read(buffer, structure, arg);
            buffer.push_str(")");
        }

        ShaderOp::Asin(arg) => {
            buffer.push_str("asin(");
            emit_shader_var_read(buffer, structure, arg);
            buffer.push_str(")");
        }

        ShaderOp::Acos(arg) => {
            buffer.push_str("acos(");
            emit_shader_var_read(buffer, structure, arg);
            buffer.push_str(")");
        }

        ShaderOp::Atan(arg) => {
            buffer.push_str("atan(");
            emit_shader_var_read(buffer, structure, arg);
            buffer.push_str(")");
        }

        ShaderOp::Sqrt(arg) => {
            buffer.push_str("sqrt(");
            emit_shader_var_read(buffer, structure, arg);
            buffer.push_str(")");
        }

        ShaderOp::Ln(arg) => {
            buffer.push_str("log(");
            emit_shader_var_read(buffer, structure, arg);
            buffer.push_str(")");
        }

        ShaderOp::Exp(arg) => {
            buffer.push_str("exp(");
            emit_shader_var_read(buffer, structure, arg);
            buffer.push_str(")");
        }

        ShaderOp::Floor(arg) => {
            buffer.push_str("floor(");
            emit_shader_var_read(buffer, structure, arg);
            buffer.push_str(")");
        }

        ShaderOp::Atan2(left, right) => {
            buffer.push_str("atan(");
            emit_shader_var_read(buffer, structure, left);
            buffer.push_str(",");
            emit_shader_var_read(buffer, structure, right);
            buffer.push_str(")");
        }

        ShaderOp::Pow(left, right) => {
            buffer.push_str("atan(");
            emit_shader_var_read(buffer, structure, left);
            buffer.push_str(",");
            emit_shader_var_read(buffer, structure, right);
            buffer.push_str(")");
        }

        ShaderOp::DerivX(arg) => {
            buffer.push_str("dFdx(");
            emit_shader_var_read(buffer, structure, arg);
            buffer.push_str(")");
        }

        ShaderOp::DerivY(arg) => {
            buffer.push_str("dFdy(");
            emit_shader_var_read(buffer, structure, arg);
            buffer.push_str(")");
        }

        ShaderOp::IOr(left, right) => {
            buffer.push_str("(");
            emit_shader_var_read(buffer, structure, left);
            buffer.push_str("|");
            emit_shader_var_read(buffer, structure, right);
            buffer.push_str(")");
        }

        ShaderOp::IAnd(left, right) => {
            buffer.push_str("(");
            emit_shader_var_read(buffer, structure, left);
            buffer.push_str("&");
            emit_shader_var_read(buffer, structure, right);
            buffer.push_str(")");
        }

        ShaderOp::IXor(left, right) => {
            buffer.push_str("(");
            emit_shader_var_read(buffer, structure, left);
            buffer.push_str("^");
            emit_shader_var_read(buffer, structure, right);
            buffer.push_str(")");
        }

        ShaderOp::IShl(left, right) => {
            buffer.push_str("(");
            emit_shader_var_read(buffer, structure, left);
            buffer.push_str("<<");
            emit_shader_var_read(buffer, structure, right);
            buffer.push_str(")");
        }

        ShaderOp::IShr(left, right) => {
            buffer.push_str("(");
            emit_shader_var_read(buffer, structure, left);
            buffer.push_str(">>");
            emit_shader_var_read(buffer, structure, right);
            buffer.push_str(")");
        }

        ShaderOp::INot(arg) => {
            buffer.push_str("(~");
            emit_shader_var_read(buffer, structure, arg);
            buffer.push_str(")");
        }

        ShaderOp::BOr(left, right) => {
            buffer.push_str("(");
            emit_shader_var_read(buffer, structure, left);
            buffer.push_str("||");
            emit_shader_var_read(buffer, structure, right);
            buffer.push_str(")");
        }

        ShaderOp::BAnd(left, right) => {
            buffer.push_str("(");
            emit_shader_var_read(buffer, structure, left);
            buffer.push_str("&&");
            emit_shader_var_read(buffer, structure, right);
            buffer.push_str(")");
        }

        ShaderOp::BXor(left, right) => {
            buffer.push_str("(");
            emit_shader_var_read(buffer, structure, left);
            buffer.push_str("!=");
            emit_shader_var_read(buffer, structure, right);
            buffer.push_str(")");
        }

        ShaderOp::BNot(arg) => {
            buffer.push_str("(!");
            emit_shader_var_read(buffer, structure, arg);
            buffer.push_str(")");
        }

        ShaderOp::IEq(left, right) | ShaderOp::FEq(left, right) => {
            buffer.push_str("(");
            emit_shader_var_read(buffer, structure, left);
            buffer.push_str("==");
            emit_shader_var_read(buffer, structure, right);
            buffer.push_str(")");
        }

        ShaderOp::INe(left, right) | ShaderOp::FNe(left, right) => {
            buffer.push_str("(");
            emit_shader_var_read(buffer, structure, left);
            buffer.push_str("==");
            emit_shader_var_read(buffer, structure, right);
            buffer.push_str(")");
        }

        ShaderOp::IGe(left, right) | ShaderOp::FGe(left, right) => {
            buffer.push_str("(");
            emit_shader_var_read(buffer, structure, left);
            buffer.push_str(">=");
            emit_shader_var_read(buffer, structure, right);
            buffer.push_str(")");
        }

        ShaderOp::ILe(left, right) | ShaderOp::FLe(left, right) => {
            buffer.push_str("(");
            emit_shader_var_read(buffer, structure, left);
            buffer.push_str("<=");
            emit_shader_var_read(buffer, structure, right);
            buffer.push_str(")");
        }

        ShaderOp::IGt(left, right) | ShaderOp::FGt(left, right) => {
            buffer.push_str("(");
            emit_shader_var_read(buffer, structure, left);
            buffer.push_str(">");
            emit_shader_var_read(buffer, structure, right);
            buffer.push_str(")");
        }

        ShaderOp::ILt(left, right) | ShaderOp::FLt(left, right) => {
            buffer.push_str("(");
            emit_shader_var_read(buffer, structure, left);
            buffer.push_str("<");
            emit_shader_var_read(buffer, structure, right);
            buffer.push_str(")");
        }

        ShaderOp::Lerp(t, left, right) => {
            buffer.push_str("lerp(");
            emit_shader_var_read(buffer, structure, t);
            buffer.push_str(",");
            emit_shader_var_read(buffer, structure, left);
            buffer.push_str(",");
            emit_shader_var_read(buffer, structure, right);
            buffer.push_str(")");
        }

        ShaderOp::FSelect(sel, left, right)
        | ShaderOp::ISelect(sel, left, right)
        | ShaderOp::BSelect(sel, left, right) => {
            buffer.push_str("(");
            emit_shader_var_read(buffer, structure, sel);
            buffer.push_str("?");
            emit_shader_var_read(buffer, structure, left);
            buffer.push_str(":");
            emit_shader_var_read(buffer, structure, right);
            buffer.push_str(")");
        }

        ShaderOp::FCastInt32(arg) => {
            buffer.push_str("float(");
            emit_shader_var_read(buffer, structure, arg);
            buffer.push_str(")");
        }

        ShaderOp::ICastFloat(arg) => {
            buffer.push_str("int(");
            emit_shader_var_read(buffer, structure, arg);
            buffer.push_str(")");
        }

        ShaderOp::TexSampleF32(_, _, _, texture_filter, texture_channel) => todo!(),
        ShaderOp::TexSampleU8(_, _, _, texture_filter, texture_channel) => todo!(),

        ShaderOp::TexW(id) => todo!(),
        ShaderOp::TexH(id) => todo!(),
    }
}

pub fn emit_shader_op_buffer_read(buffer: &mut String, offset: u32, float: bool, size: u32) {
    let (b16, b4, b1) = (offset >> 4, (offset >> 2) & 3, (offset & 3) << 3);
    let b4 = ["x", "y", "z", "w"][b4 as usize];

    match size {
        1 if b1 == 0 => write!(buffer, "int(di({}).{}&255)", b16, b4),
        1 => write!(buffer, "int((di({}).{}>>{}u)&255)", b16, b4, b1),
        2 if b1 == 0 => write!(buffer, "int(di({}).{}&65535)", b16, b4),
        2 => write!(buffer, "int((di({}).{}>>{}u)&65535)", b16, b4, b1),
        4 if float => write!(buffer, "df({}).{}", b16, b4),
        4 => write!(buffer, "di({}).{}", b16, b4),
        _ => unreachable!(),
    }
    .ok();
}

pub fn emit_fragment_header(buffer: &mut String, options: &CompilerOptions) {
    emit_header_version_decl(buffer, options.glsl_version, options.buffer_mode);
    emit_header_buffer_binding(buffer, options.buffer_mode);
    emit_header_texture_samplers(buffer, options.texture_units);
    buffer.push_str(
        r#"precision highp float;
precision highp int;

uniform vec2 uResolution;
uniform int uBufferDataOffset;

flat in int fragType;
flat in int fragData;
flat in vec4 fragBounds;
in vec2 fragPos;
out vec4 outColor;

vec4 sl(in sampler2D s,vec2 i){return textureLod(s,i/textureSize(s,0),0);}
vec4 sn(in sampler2D s,vec2 i){return texelFetch(s,ivec2(i),0);}
vec4 df(int i){return readF(uBufferDataOffset+fragData+i);}
ivec4 di(int i){return ivec4(readU(uBufferDataOffset+fragData+i));}"#,
    );
}

pub fn emit_fragment_dispatch<'a>(buffer: &mut String, branches: impl Iterator<Item = u32>) {
    buffer.push_str("void main(){switch(fragType){");

    for index in branches {
        write!(buffer, "case {index}:s_{index:x}();break;").ok();
    }

    buffer.push_str("default:outColor=vec4(1.0,0.0,1.0,1.0);break;}}");
}

pub fn emit_vertex_program(buffer: &mut String, options: &CompilerOptions) {
    emit_header_version_decl(buffer, options.glsl_version, options.buffer_mode);
    emit_header_buffer_binding(buffer, options.buffer_mode);

    buffer.push_str(
        r#"
precision highp float;
precision highp int;

uniform vec2 uResolution;
uniform bool uScreenTarget;
uniform int uBufferListOffset;

flat out int fragType;
flat out int fragData;
flat out vec4 fragBounds;
out vec2 fragPos;

void main() {
    int triangleId = gl_VertexID / 3;
    int vertexId = gl_VertexID % 3;
    int quadId = triangleId >> 1;
    int cornerId = (triangleId & 1) + vertexId;
    uvec4 packedData = readU(uBufferListOffset + quadId);
    vec2 topLeft = vec2(float(packedData.x & 65535u), float((packedData.x >> 16) & 65535u));
    vec2 bottomRight = vec2(float(packedData.y & 65535u), float((packedData.y >> 16) & 65535u));
    vec2 pos = vec2(float(cornerId >> 1), float(cornerId & 1)) * (bottomRight - topLeft) + topLeft;
    gl_Position = vec4((2 * pos / uResolution - 1) * vec2(1, uScreenTarget ? -1 : 1), 0, 1);
    fragBounds = vec4(topLeft, bottomRight);
    fragPos = pos;
    fragType = int(packedData.z);
    fragData = int(packedData.w);    
}"#,
    );
}

pub fn emit_header_buffer_binding(buffer: &mut String, mode: CompilerBufferMode) {
    match mode {
        CompilerBufferMode::UniformBlock { size_bytes } => {
            let size = size_bytes / size_of::<[u32; 4]>() as u32;

            writeln!(
                buffer,
                "
layout(std140) uniform uBufferU32 {{uvec4 bufferU32[{}];}};
layout(std140) uniform uBufferF32 {{vec4 bufferF32[{}];}};
uvec4 readU(int i){{return bufferU32[i%{}];}};
vec4 readF(int i){{return bufferF32[i%{}];}};
                ",
                size, size, size, size
            )
            .ok();
        }
        CompilerBufferMode::TextureBuffer => {
            writeln!(
                buffer,
                "
uniform usamplerBuffer uBuffer;
uvec4 readU(int i){{return texelFetch(uBuffer,i);}};
vec4 readF(int i){{return uintBitsToFloat(readU(i));}};
                "
            )
            .ok();
        }
    }
}

pub fn emit_header_version_decl(buffer: &mut String, version: u32, mode: CompilerBufferMode) {
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

pub fn emit_header_texture_samplers(buffer: &mut String, texture_samplers: u32) {
    if texture_samplers > 0 {
        writeln!(buffer, "uniform sampler2D uTextures[{}];", texture_samplers).ok();
    }
}
