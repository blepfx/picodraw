#![allow(clippy::single_char_add_str)]

use super::{
    CompilerBufferMode, CompilerOptions, CompilerShader,
    analysis::{GlslExpr, GlslStmt, GlslType},
};
use picodraw_core::TextureFilter;
use std::{collections::HashMap, fmt::Write};

pub struct ShaderContext<'a> {
    pub metadata: &'a CompilerShader,
    pub expressions: &'a HashMap<u32, GlslExpr>,
    pub statements: &'a Vec<GlslStmt>,
}

pub fn emit_shader_function(buffer: &mut String, shader: &ShaderContext) {
    write!(buffer, "void s_{:x}(){{", shader.metadata.index).ok();

    for stmt in shader.statements {
        emit_shader_statement(buffer, shader, stmt);
    }

    buffer.push_str("}");
}

pub fn emit_shader_statement(buffer: &mut String, shader: &ShaderContext, statement: &GlslStmt) {
    match statement {
        GlslStmt::DeclareVars { type_, count } => {
            let type_str = match type_ {
                GlslType::Bool1 => "bool",
                GlslType::Int1 => "int",
                GlslType::Int2 => "ivec2",
                GlslType::Int4 => "ivec4",
                GlslType::Float1 => "float",
                GlslType::Float2 => "vec2",
                GlslType::Float4 => "vec4",
            };

            write!(buffer, "{} ", type_str).ok();
            for i in 0..*count {
                emit_shader_variable(buffer, i, *type_);
                if i + 1 < *count {
                    buffer.push_str(",");
                }
            }
            buffer.push_str(";");
        }

        GlslStmt::AssignExpr { var, expr, type_ } => {
            emit_shader_variable(buffer, *var, *type_);
            buffer.push_str("=");
            emit_shader_expression(buffer, shader, expr);
            buffer.push_str(";");
        }

        GlslStmt::ReturnExpr { expr } => {
            buffer.push_str("outColor=");
            emit_shader_expression(buffer, shader, expr);
            buffer.push_str(";");
        }

        GlslStmt::Branch {
            condition,
            then_branch,
            else_branch,
        } => {
            buffer.push_str("if(");
            emit_shader_expression(buffer, shader, condition);
            buffer.push_str("){");
            for stmt in then_branch {
                emit_shader_statement(buffer, shader, stmt);
            }
            buffer.push_str("}else{");
            for stmt in else_branch {
                emit_shader_statement(buffer, shader, stmt);
            }
            buffer.push_str("}");
        }
    }
}

// name of var if not inline
pub fn emit_shader_variable(buffer: &mut String, register: u32, type_: GlslType) {
    match type_ {
        GlslType::Bool1 => write!(buffer, "b{:x}", register).ok(),
        GlslType::Int1 => write!(buffer, "i{:x}", register).ok(),
        GlslType::Int2 => write!(buffer, "iY{:x}", register).ok(),
        GlslType::Int4 => write!(buffer, "iW{:x}", register).ok(),
        GlslType::Float1 => write!(buffer, "f{:x}", register).ok(),
        GlslType::Float2 => write!(buffer, "fY{:x}", register).ok(),
        GlslType::Float4 => write!(buffer, "fW{:x}", register).ok(),
    };
}

// operation over values, can be inlined or assigned to a var
pub fn emit_shader_expression(buffer: &mut String, shader: &ShaderContext, expr: &GlslExpr) {
    match expr {
        GlslExpr::LitBool(true) => buffer.push_str("true"),
        GlslExpr::LitBool(false) => buffer.push_str("false"),
        GlslExpr::LitFloat(x) => match f32::from_bits(*x) {
            f32::INFINITY => buffer.push_str("4.6e+18"),
            f32::NEG_INFINITY => buffer.push_str("(-4.6e+18)"),
            x if x.is_nan() => buffer.push_str("(0.0/0.0)"),
            x if x.is_sign_positive() => {
                let _ = write!(buffer, "{:?}", x);
            }
            x => {
                let _ = write!(buffer, "({:?})", x);
            }
        },

        GlslExpr::LitInt(x) if !x.is_negative() => {
            let _ = write!(buffer, "{:?}", x);
        }
        GlslExpr::LitInt(x) => {
            let _ = write!(buffer, "({:?})", x);
        }

        GlslExpr::Position => buffer.push_str("fragPos"),
        GlslExpr::Resolution => buffer.push_str("uResolution"),
        GlslExpr::QuadBounds => buffer.push_str("fragBounds"),

        GlslExpr::Neg(a) | GlslExpr::INot(a) | GlslExpr::BNot(a) => {
            let op = match expr {
                GlslExpr::Neg(_) => "-",
                GlslExpr::INot(_) => "~",
                GlslExpr::BNot(_) => "!",
                _ => unreachable!(),
            };

            buffer.push_str("(");
            buffer.push_str(op);
            emit_shader_expression(buffer, shader, &shader.expressions[a]);
            buffer.push_str(")");
        }

        GlslExpr::Add(a, b)
        | GlslExpr::Sub(a, b)
        | GlslExpr::Mul(a, b)
        | GlslExpr::Div(a, b)
        | GlslExpr::IRem(a, b)
        | GlslExpr::BAnd(a, b)
        | GlslExpr::BOr(a, b)
        | GlslExpr::IXor(a, b)
        | GlslExpr::IAnd(a, b)
        | GlslExpr::IOr(a, b)
        | GlslExpr::IShl(a, b)
        | GlslExpr::IShr(a, b)
        | GlslExpr::Eq(a, b)
        | GlslExpr::Ne(a, b)
        | GlslExpr::Lt(a, b)
        | GlslExpr::Gt(a, b)
        | GlslExpr::Le(a, b)
        | GlslExpr::Ge(a, b) => {
            let op = match expr {
                GlslExpr::Add(_, _) => "+",
                GlslExpr::Sub(_, _) => "-",
                GlslExpr::Mul(_, _) => "*",
                GlslExpr::Div(_, _) => "/",
                GlslExpr::IRem(_, _) => "%",
                GlslExpr::BAnd(_, _) => "&&",
                GlslExpr::BOr(_, _) => "||",
                GlslExpr::IXor(_, _) => "^",
                GlslExpr::IAnd(_, _) => "&",
                GlslExpr::IOr(_, _) => "|",
                GlslExpr::IShl(_, _) => "<<",
                GlslExpr::IShr(_, _) => ">>",
                GlslExpr::Eq(_, _) => "==",
                GlslExpr::Ne(_, _) => "!=",
                GlslExpr::Lt(_, _) => "<",
                GlslExpr::Gt(_, _) => ">",
                GlslExpr::Le(_, _) => "<=",
                GlslExpr::Ge(_, _) => ">=",
                _ => unreachable!(),
            };

            buffer.push_str("(");
            emit_shader_expression(buffer, shader, &shader.expressions[a]);
            buffer.push_str(op);
            emit_shader_expression(buffer, shader, &shader.expressions[b]);
            buffer.push_str(")");
        }

        GlslExpr::Abs(a)
        | GlslExpr::Sign(a)
        | GlslExpr::Sin(a)
        | GlslExpr::Cos(a)
        | GlslExpr::Tan(a)
        | GlslExpr::Asin(a)
        | GlslExpr::Acos(a)
        | GlslExpr::Atan(a)
        | GlslExpr::Sqrt(a)
        | GlslExpr::Log(a)
        | GlslExpr::Exp(a)
        | GlslExpr::Floor(a)
        | GlslExpr::DerivX(a)
        | GlslExpr::DerivY(a)
        | GlslExpr::AsFloat(a)
        | GlslExpr::AsInt(a) => {
            let func = match expr {
                GlslExpr::Abs(_) => "abs",
                GlslExpr::Sign(_) => "sign",
                GlslExpr::Sin(_) => "sin",
                GlslExpr::Cos(_) => "cos",
                GlslExpr::Tan(_) => "tan",
                GlslExpr::Asin(_) => "asin",
                GlslExpr::Acos(_) => "acos",
                GlslExpr::Atan(_) => "atan",
                GlslExpr::Sqrt(_) => "sqrt",
                GlslExpr::Log(_) => "log",
                GlslExpr::Exp(_) => "exp",
                GlslExpr::Floor(_) => "floor",
                GlslExpr::DerivX(_) => "dFdx",
                GlslExpr::DerivY(_) => "dFdy",
                GlslExpr::AsFloat(_) => "float",
                GlslExpr::AsInt(_) => "int",
                _ => unreachable!(),
            };

            buffer.push_str(func);
            buffer.push_str("(");
            emit_shader_expression(buffer, shader, &shader.expressions[a]);
            buffer.push_str(")");
        }

        GlslExpr::Min(a, b)
        | GlslExpr::Max(a, b)
        | GlslExpr::Pow(a, b)
        | GlslExpr::Atan2(a, b)
        | GlslExpr::FRem(a, b) => {
            let func = match expr {
                GlslExpr::Min(_, _) => "min",
                GlslExpr::Max(_, _) => "max",
                GlslExpr::Pow(_, _) => "pow",
                GlslExpr::Atan2(_, _) => "atan",
                GlslExpr::FRem(_, _) => "mod",
                _ => unreachable!(),
            };

            buffer.push_str(func);
            buffer.push_str("(");
            emit_shader_expression(buffer, shader, &shader.expressions[a]);
            buffer.push_str(",");
            emit_shader_expression(buffer, shader, &shader.expressions[b]);
            buffer.push_str(")");
        }

        GlslExpr::Mix(a, b, c) => {
            buffer.push_str("mix(");
            emit_shader_expression(buffer, shader, &shader.expressions[a]);
            buffer.push_str(",");
            emit_shader_expression(buffer, shader, &shader.expressions[b]);
            buffer.push_str(",");
            emit_shader_expression(buffer, shader, &shader.expressions[c]);
            buffer.push_str(")");
        }

        GlslExpr::Select(a, b, c) => {
            buffer.push_str("(");
            emit_shader_expression(buffer, shader, &shader.expressions[a]);
            buffer.push_str("?");
            emit_shader_expression(buffer, shader, &shader.expressions[b]);
            buffer.push_str(":");
            emit_shader_expression(buffer, shader, &shader.expressions[c]);
            buffer.push_str(")");
        }

        GlslExpr::Vec4([a, b, c, d]) => {
            buffer.push_str("vec4(");
            emit_shader_expression(buffer, shader, &shader.expressions[a]);
            buffer.push_str(",");
            emit_shader_expression(buffer, shader, &shader.expressions[b]);
            buffer.push_str(",");
            emit_shader_expression(buffer, shader, &shader.expressions[c]);
            buffer.push_str(",");
            emit_shader_expression(buffer, shader, &shader.expressions[d]);
            buffer.push_str(")");
        }

        GlslExpr::Swizzle1(a, b) => {
            emit_shader_expression(buffer, shader, &shader.expressions[a]);

            match b {
                0 => buffer.push_str(".x"),
                1 => buffer.push_str(".y"),
                2 => buffer.push_str(".z"),
                3 => buffer.push_str(".w"),
                _ => {}
            }
        }

        GlslExpr::DataFloat(offset) => {
            write!(buffer, "df({})", offset).ok();
        }

        GlslExpr::DataInt(offset) => {
            write!(buffer, "di({})", offset).ok();
        }

        GlslExpr::TextureSize(slot) => {
            let texture_id = shader.metadata.texture_slots[*slot as usize];
            write!(buffer, "textureSize(uTextures[{}],0)", texture_id).ok();
        }

        GlslExpr::TextureSample(slot, filter, x, y) => {
            let texture_id = shader.metadata.texture_slots[*slot as usize];
            let filter_str = match filter {
                TextureFilter::Linear => "sl",
                TextureFilter::Nearest => "sn",
            };

            write!(buffer, "{}(uTextures[{}", filter_str, texture_id).ok();
            buffer.push_str("],vec2(");
            emit_shader_expression(buffer, shader, &shader.expressions[x]);
            buffer.push_str(",");
            emit_shader_expression(buffer, shader, &shader.expressions[y]);
            buffer.push_str("))");
        }

        GlslExpr::Variable(var, ty) => {
            emit_shader_variable(buffer, *var, *ty);
        }

        GlslExpr::Output(_) => {}
    }
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

pub fn emit_fragment_dispatch(buffer: &mut String, branches: impl Iterator<Item = u32>) {
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
