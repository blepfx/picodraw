use super::{
    atlas::{ShaderTextures, TextureAtlas},
    encoding::{
        InputField, InputRepr, InputStructure, BUILTIN_BOUNDS, BUILTIN_POSITION, BUILTIN_RESOLUTION,
    },
};
use crate::{
    graph::{Op, OpAddr, ShaderGraph, Swizzle, ValueType},
    Float4,
};
use std::{
    collections::HashMap,
    fmt::{self, Write},
};

pub const VERTEX_SHADER: &str = r#"
#version 330 core
precision highp float;
uniform int uBufferOffsetInstance;
uniform int uBufferOffsetData;
uniform usamplerBuffer uBuffer;
uniform vec2 uResolution;
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
    gl_Position = vec4((2.0 * pos / uResolution - 1.0) * vec2(1.0, -1.0), 0.0, 1.0);
    fragPosition = pos;
    fragBounds = vec4(topLeft, bottomRight);
    fragType = int(packedData.z);
    fragData = uBufferOffsetData + int(packedData.w);    
}"#;

const FRAGMENT_SHADER_HEADER: &str = r#"
#version 330 core
precision highp float;
uniform usamplerBuffer uBuffer;
uniform sampler2D uAtlas;
uniform vec2 uResolution;
flat in int fragType;
flat in int fragData;
flat in vec4 fragBounds;
in vec2 fragPosition;
out vec4 outColor;
int uint2int(uint x,uint m){return int(x)-int((x&m)<<1);}
void main(){
"#;

pub fn generate_fragment_shader<'a>(
    graphs: impl IntoIterator<Item = (u32, &'a ShaderGraph<Float4>, &'a InputStructure)>,
    atlas: &TextureAtlas,
) -> String {
    let mut result = String::from(FRAGMENT_SHADER_HEADER);

    for (order, (key, graph, input)) in graphs.into_iter().enumerate() {
        if order > 0 {
            write!(result, "else ").ok();
        }

        write!(result, "if(fragType == {}){{", key as i32).ok();

        let inputs = emit_decoder(
            &mut result,
            |f, offset| write!(f, "texelFetch(uBuffer,fragData+{})", offset),
            &input,
        )
        .unwrap();

        emit_graph_function(
            &mut result,
            graph,
            atlas.shader(key),
            |f, v| match v {
                BUILTIN_POSITION => write!(f, "fragPosition"),
                BUILTIN_RESOLUTION => write!(f, "uResolution"),
                BUILTIN_BOUNDS => write!(f, "fragBounds"),
                v => write!(f, "{}", inputs.get(&v).unwrap()),
            },
            |f, expr| write!(f, "outColor={};", expr),
        )
        .ok();

        write!(result, "}}").ok();
    }

    write!(result, "}}").ok();

    result
}

fn emit_decoder(
    f: &mut dyn Write,
    mut fetch: impl FnMut(&mut dyn Write, u32) -> fmt::Result,
    input: &InputStructure,
) -> Result<HashMap<usize, String>, fmt::Error> {
    let mut result = HashMap::new();

    for i in 0..input.size.div_ceil(16) {
        write!(f, "uvec4 _p{:x}=", i)?;
        fetch(f, i)?;
        write!(f, ";")?;
    }

    for (id, field) in input.inputs.iter().enumerate() {
        let expr = emit_decoder_for_type(f, id as u32, field)?;
        result.insert(id, expr);
    }

    Ok(result)
}

fn emit_decoder_for_type(
    f: &mut dyn Write,
    id: u32,
    field: &InputField,
) -> Result<String, fmt::Error> {
    fn extract8(f: &mut dyn Write, offset: u32) -> fmt::Result {
        let byte = offset & 3;
        let int = (offset >> 2) & 3;
        let vec = offset >> 4;
        write!(
            f,
            "((_p{:x}.{}>>{}u)&255u)",
            vec,
            match int {
                0 => "x",
                1 => "y",
                2 => "z",
                _ => "w",
            },
            byte * 8
        )
    }

    fn extract16(f: &mut dyn Write, offset: u32) -> fmt::Result {
        let short = (offset & 3) >> 1;
        let int = (offset >> 2) & 3;
        let vec = offset >> 4;
        write!(
            f,
            "((_p{:x}.{}>>{}u)&65535u)",
            vec,
            match int {
                0 => "x",
                1 => "y",
                2 => "z",
                _ => "w",
            },
            short * 16
        )
    }

    fn extract32(f: &mut dyn Write, offset: u32) -> fmt::Result {
        let int = (offset >> 2) & 3;
        let vec = offset >> 4;
        write!(
            f,
            "(_p{:x}.{})",
            vec,
            match int {
                0 => "x",
                1 => "y",
                2 => "z",
                _ => "w",
            }
        )
    }

    let id = format!("_i{:x}", id);
    match &field.repr {
        InputRepr::Int8 => {
            write!(f, "int {id}=uint2int(")?;
            extract8(f, field.offset)?;
            write!(f, ",256u);")?;
        }

        InputRepr::Int16 => {
            write!(f, "int {id}=uint2int(")?;
            extract16(f, field.offset)?;
            write!(f, ",32768u);")?;
        }

        InputRepr::Int32 => {
            write!(f, "int(")?;
            extract32(f, field.offset)?;
            write!(f, ");")?;
        }

        InputRepr::UInt8 => {
            write!(f, "int {id}=int(")?;
            extract8(f, field.offset)?;
            write!(f, ");")?;
        }

        InputRepr::UInt16 => {
            write!(f, "int {id}=int(")?;
            extract16(f, field.offset)?;
            write!(f, ");")?;
        }

        InputRepr::UInt32 => {
            write!(f, "int {id}=int(")?;
            extract32(f, field.offset)?;
            write!(f, ");")?;
        }

        InputRepr::Float32 => {
            write!(f, "float {id}=uintBitsToFloat(")?;
            extract32(f, field.offset)?;
            write!(f, ");")?;
        }
    }

    Ok(id)
}

fn emit_graph_function(
    f: &mut dyn Write,
    graph: &ShaderGraph<Float4>,
    atlas: ShaderTextures,
    mut write_input: impl FnMut(&mut dyn Write, usize) -> fmt::Result,
    mut write_output: impl FnMut(&mut dyn Write, &str) -> fmt::Result,
) -> fmt::Result {
    // usage analysis
    let usages = {
        let mut usages = HashMap::<OpAddr, u32, _>::new();
        *usages.entry(graph.result()).or_default() += 1;
        for (id, op, _) in graph.iter().rev() {
            if matches!(op, Op::SlotUpdate(_, _) | Op::LoopPush(_) | Op::LoopPop) {
                *usages.entry(id).or_default() += 1;
            }

            if usages.contains_key(&id) {
                op.visit_dependencies(|dep| {
                    *usages.entry(dep).or_default() += 1;
                });
            }
        }
        usages
    };

    let mut atoms = HashMap::new();
    for (id, op, ty) in graph.iter() {
        let usages = usages.get(&id).copied().unwrap_or_default();

        match op {
            Op::Input(ident) => {
                if ty != ValueType::Texture {
                    let mut string = String::new();
                    write_input(&mut string, ident)?;
                    atoms.insert(id, string);
                }
            }

            Op::SlotCreate(init) => {
                let name = format!("_{:x}", id.id());
                write!(
                    f,
                    "{} {}={};",
                    type_name(ty),
                    name,
                    atoms.get(&init).unwrap()
                )?;
                atoms.insert(id, name);
            }

            Op::SlotUpdate(slot, value) => {
                write!(f, "_{:x}={};", slot.id(), atoms.get(&value).unwrap())?;
            }

            Op::LoopPush(cond) => {
                write!(f, "while({}){{", atoms.get(&cond).unwrap())?;
            }

            Op::LoopPop => {
                write!(f, "}}")?;
            }

            Op::LitFloat(_) | Op::LitInt(_) | Op::LitBool(_) => {
                let mut f = String::new();
                match op {
                    Op::LitFloat(f32::INFINITY) => write!(f, "uintBitsToFloat(0x7F800000)")?,
                    Op::LitFloat(f32::NEG_INFINITY) => write!(f, "uintBitsToFloat(0xFF800000)")?,
                    Op::LitFloat(x) if x.is_nan() => write!(f, "intBitsToFloat(-1)")?,
                    Op::LitFloat(x) if x.is_sign_positive() => write!(f, "{x:?}")?,
                    Op::LitFloat(x) => write!(f, "({x:?})")?,
                    Op::LitInt(x) if x >= 0 => write!(f, "{x}")?,
                    Op::LitInt(x) => write!(f, "({x})")?,
                    Op::LitBool(true) => write!(f, "true")?,
                    Op::LitBool(false) => write!(f, "false")?,
                    _ => {}
                }
                atoms.insert(id, f);
            }

            _ if usages == 1 => {
                let mut string = String::new();
                emit_graph_atom(&mut string, op, graph, atlas, |f, value| {
                    write!(f, "{}", atoms.get(&value).unwrap())
                })?;
                atoms.insert(id, string);
            }

            _ if usages > 1 => {
                let name = format!("_{:x}", id.id());

                write!(f, "{} {}=", type_name(ty), name)?;
                emit_graph_atom(f, op, graph, atlas, |f, value| {
                    write!(f, "{}", atoms.get(&value).unwrap())
                })?;
                write!(f, ";")?;

                atoms.insert(id, name);
            }

            _ => {}
        }
    }

    write_output(f, atoms.get(&graph.result()).unwrap())?;

    Ok(())
}

fn emit_graph_atom<'a>(
    f: &mut dyn Write,
    op: Op,
    graph: &ShaderGraph<Float4>,
    atlas: ShaderTextures,
    mut dep: impl FnMut(&mut dyn Write, OpAddr) -> fmt::Result,
) -> fmt::Result {
    match op {
        Op::Add(a, b) => {
            write!(f, "(")?;
            dep(f, a)?;
            write!(f, "+")?;
            dep(f, b)?;
            write!(f, ")")?
        }
        Op::Sub(a, b) => {
            write!(f, "(")?;
            dep(f, a)?;
            write!(f, "-")?;
            dep(f, b)?;
            write!(f, ")")?
        }
        Op::Mul(a, b) => {
            write!(f, "(")?;
            dep(f, a)?;
            write!(f, "*")?;
            dep(f, b)?;
            write!(f, ")")?
        }
        Op::Div(a, b) => {
            write!(f, "(")?;
            dep(f, a)?;
            write!(f, "/")?;
            dep(f, b)?;
            write!(f, ")")?
        }
        Op::Rem(a, b) => {
            write!(f, "mod(")?;
            dep(f, a)?;
            write!(f, ",")?;
            dep(f, b)?;
            write!(f, ")")?
        }
        Op::Dot(a, b) => {
            write!(f, "dot(")?;
            dep(f, a)?;
            write!(f, ",")?;
            dep(f, b)?;
            write!(f, ")")?
        }
        Op::Cross(a, b) => {
            write!(f, "cross(")?;
            dep(f, a)?;
            write!(f, ",")?;
            dep(f, b)?;
            write!(f, ")")?
        }
        Op::Neg(a) => {
            write!(f, "(-")?;
            dep(f, a)?;
            write!(f, ")")?
        }
        Op::Sin(a) => {
            write!(f, "sin(")?;
            dep(f, a)?;
            write!(f, ")")?;
        }
        Op::Cos(a) => {
            write!(f, "cos(")?;
            dep(f, a)?;
            write!(f, ")")?;
        }
        Op::Tan(a) => {
            write!(f, "tan(")?;
            dep(f, a)?;
            write!(f, ")")?;
        }
        Op::Asin(a) => {
            write!(f, "asin(")?;
            dep(f, a)?;
            write!(f, ")")?;
        }
        Op::Acos(a) => {
            write!(f, "acos(")?;
            dep(f, a)?;
            write!(f, ")")?;
        }
        Op::Atan(a) => {
            write!(f, "atan(")?;
            dep(f, a)?;
            write!(f, ")")?;
        }
        Op::Atan2(a, b) => {
            write!(f, "atan(")?;
            dep(f, a)?;
            write!(f, ",")?;
            dep(f, b)?;
            write!(f, ")")?;
        }
        Op::Sqrt(a) => {
            write!(f, "sqrt(")?;
            dep(f, a)?;
            write!(f, ")")?;
        }
        Op::Pow(a, b) => {
            write!(f, "pow(")?;
            dep(f, a)?;
            write!(f, ",")?;
            dep(f, b)?;
            write!(f, ")")?;
        }
        Op::Exp(a) => {
            write!(f, "exp(")?;
            dep(f, a)?;
            write!(f, ")")?;
        }
        Op::Ln(a) => {
            write!(f, "log(")?;
            dep(f, a)?;
            write!(f, ")")?;
        }
        Op::Min(a, b) => {
            write!(f, "min(")?;
            dep(f, a)?;
            write!(f, ",")?;
            dep(f, b)?;
            write!(f, ")")?;
        }
        Op::Max(a, b) => {
            write!(f, "max(")?;
            dep(f, a)?;
            write!(f, ",")?;
            dep(f, b)?;
            write!(f, ")")?;
        }
        Op::Clamp(x, min, max) => {
            write!(f, "clamp(")?;
            dep(f, x)?;
            write!(f, ",")?;
            dep(f, min)?;
            write!(f, ",")?;
            dep(f, max)?;
            write!(f, ")")?;
        }
        Op::Abs(a) => {
            write!(f, "abs(")?;
            dep(f, a)?;
            write!(f, ")")?;
        }
        Op::Sign(a) => {
            write!(f, "sign(")?;
            dep(f, a)?;
            write!(f, ")")?;
        }
        Op::Floor(a) => {
            write!(f, "floor(")?;
            dep(f, a)?;
            write!(f, ")")?;
        }
        Op::Fract(a) => {
            write!(f, "fract(")?;
            dep(f, a)?;
            write!(f, ")")?;
        }
        Op::Lerp(x, min, max) => {
            write!(f, "mix(")?;
            dep(f, min)?;
            write!(f, ",")?;
            dep(f, max)?;
            write!(f, ",")?;
            dep(f, x)?;
            write!(f, ")")?;
        }
        Op::Select(x, tru, fls) => {
            write!(f, "(")?;
            dep(f, x)?;
            write!(f, "?")?;
            dep(f, tru)?;
            write!(f, ":")?;
            dep(f, fls)?;
            write!(f, ")")?;
        }
        Op::Smoothstep(x, min, max) => {
            write!(f, "smoothstep(")?;
            dep(f, min)?;
            write!(f, ",")?;
            dep(f, max)?;
            write!(f, ",")?;
            dep(f, x)?;
            write!(f, ")")?;
        }
        Op::Step(x, edge) => {
            write!(f, "step(")?;
            dep(f, edge)?;
            write!(f, ",")?;
            dep(f, x)?;
            write!(f, ")")?;
        }

        Op::Eq(a, b) => {
            write!(f, "(")?;
            dep(f, a)?;
            write!(f, "==")?;
            dep(f, b)?;
            write!(f, ")")?
        }
        Op::Ne(a, b) => {
            write!(f, "(")?;
            dep(f, a)?;
            write!(f, "!=")?;
            dep(f, b)?;
            write!(f, ")")?
        }
        Op::Lt(a, b) => {
            write!(f, "(")?;
            dep(f, a)?;
            write!(f, "<")?;
            dep(f, b)?;
            write!(f, ")")?
        }
        Op::Le(a, b) => {
            write!(f, "(")?;
            dep(f, a)?;
            write!(f, "<=")?;
            dep(f, b)?;
            write!(f, ")")?
        }
        Op::Gt(a, b) => {
            write!(f, "(")?;
            dep(f, a)?;
            write!(f, ">")?;
            dep(f, b)?;
            write!(f, ")")?
        }
        Op::Ge(a, b) => {
            write!(f, "(")?;
            dep(f, a)?;
            write!(f, ">=")?;
            dep(f, b)?;
            write!(f, ")")?
        }
        Op::And(a, b) => {
            write!(f, "(")?;
            dep(f, a)?;
            write!(f, "&&")?;
            dep(f, b)?;
            write!(f, ")")?
        }
        Op::Or(a, b) => {
            write!(f, "(")?;
            dep(f, a)?;
            write!(f, "||")?;
            dep(f, b)?;
            write!(f, ")")?
        }
        Op::Xor(a, b) => {
            write!(f, "(")?;
            dep(f, a)?;
            write!(f, "^^")?;
            dep(f, b)?;
            write!(f, ")")?
        }
        Op::Not(a) => {
            write!(f, "(!")?;
            dep(f, a)?;
            write!(f, ")")?
        }

        Op::NewVec2(a, b) => {
            write!(f, "vec2(")?;
            dep(f, a)?;
            write!(f, ",")?;
            dep(f, b)?;
            write!(f, ")")?;
        }
        Op::NewVec3(a, b, c) => {
            write!(f, "vec3(")?;
            dep(f, a)?;
            write!(f, ",")?;
            dep(f, b)?;
            write!(f, ",")?;
            dep(f, c)?;
            write!(f, ")")?;
        }
        Op::NewVec4(a, b, c, d) => {
            write!(f, "vec4(")?;
            dep(f, a)?;
            write!(f, ",")?;
            dep(f, b)?;
            write!(f, ",")?;
            dep(f, c)?;
            write!(f, ",")?;
            dep(f, d)?;
            write!(f, ")")?;
        }
        Op::SplatVec2(a) => {
            write!(f, "vec2(")?;
            dep(f, a)?;
            write!(f, ")")?;
        }
        Op::SplatVec3(a) => {
            write!(f, "vec3(")?;
            dep(f, a)?;
            write!(f, ")")?;
        }
        Op::SplatVec4(a) => {
            write!(f, "vec4(")?;
            dep(f, a)?;
            write!(f, ")")?;
        }
        Op::CastFloat(a) => {
            write!(f, "float(")?;
            dep(f, a)?;
            write!(f, ")")?;
        }
        Op::CastInt(a) => {
            write!(f, "int(")?;
            dep(f, a)?;
            write!(f, ")")?;
        }

        Op::Length(a) => {
            write!(f, "length(")?;
            dep(f, a)?;
            write!(f, ")")?;
        }
        Op::Normalize(a) => {
            write!(f, "normalize(")?;
            dep(f, a)?;
            write!(f, ")")?;
        }

        Op::Swizzle1(a, Swizzle::X) => {
            dep(f, a)?;
            write!(f, ".x")?;
        }

        Op::Swizzle1(a, Swizzle::Y) => {
            dep(f, a)?;
            write!(f, ".y")?;
        }

        Op::Swizzle1(a, Swizzle::Z) => {
            dep(f, a)?;
            write!(f, ".z")?;
        }

        Op::Swizzle1(a, Swizzle::W) => {
            dep(f, a)?;
            write!(f, ".w")?;
        }

        Op::DerivX(a) => {
            write!(f, "dFdx(")?;
            dep(f, a)?;
            write!(f, ")")?;
        }
        Op::DerivY(a) => {
            write!(f, "dFdy(")?;
            dep(f, a)?;
            write!(f, ")")?;
        }
        Op::DerivWidth(a) => {
            write!(f, "fwidth(")?;
            dep(f, a)?;
            write!(f, ")")?;
        }

        Op::TextureSampleLinear(index, b) => {
            let texture = match graph.get(index) {
                (Op::Input(id), _) => atlas.get(*id as u32),
                _ => unreachable!(),
            };

            let (sample, w, h) = if texture.rotated {
                (".yx", texture.data.height(), texture.data.width())
            } else {
                ("", texture.data.width(), texture.data.height())
            };

            write!(
                f,
                "texture(uAtlas,(vec2({}.0,{}.0)+clamp(0.5+",
                texture.x, texture.y
            )?;
            dep(f, b)?;
            write!(
                f,
                "{},vec2(0.0),vec2({}.0,{}.0)))/{}.0)",
                sample, w, h, atlas.atlas.size
            )?;
        }

        Op::TextureSampleNearest(index, b) => {
            let texture = match graph.get(index) {
                (Op::Input(id), _) => atlas.get(*id as u32),
                _ => unreachable!(),
            };

            let (sample, w, h) = if texture.rotated {
                (".yx", texture.data.height(), texture.data.width())
            } else {
                ("", texture.data.width(), texture.data.height())
            };

            write!(
                f,
                "texelFetch(uAtlas,ivec2({},{})+clamp(ivec2(",
                texture.x, texture.y
            )?;
            dep(f, b)?;
            write!(f, "){},ivec2(0),ivec2({},{})),0)", sample, w, h)?;
        }

        Op::TextureSize(index) => {
            let texture = match graph.get(index) {
                (Op::Input(id), _) => atlas.get(*id as u32),
                _ => unreachable!(),
            };

            write!(
                f,
                "vec2({}.0,{}.0)",
                texture.data.width(),
                texture.data.height()
            )?;
        }

        _ => unreachable!(),
    }

    Ok(())
}

fn type_name(ty: ValueType) -> &'static str {
    match ty {
        ValueType::Int1 => "int",
        ValueType::Bool1 => "bool",
        ValueType::Float1 => "float",
        ValueType::Float2 => "vec2",
        ValueType::Float3 => "vec3",
        ValueType::Float4 => "vec4",

        _ => todo!(),
    }
}
