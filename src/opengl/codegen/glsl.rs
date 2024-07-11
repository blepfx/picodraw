use super::{
    atlas::{ShaderTextures, TextureAtlas},
    encoding::{InputField, InputRepr, InputStructure},
};
use crate::{
    graph::{ShaderGraph, Swizzle, Value, ValueSource, ValueType},
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
out vec4 fragBounds;
out vec2 fragPosition;
void main() {
    int triangleId = gl_VertexID / 3;
    int vertexId = gl_VertexID % 3;
    int quadId = triangleId >> 1;
    int cornerId = (triangleId & 1) + vertexId;
    uvec4 packedData = texelFetch(uBuffer, uBufferOffsetInstance + quadId);
    vec2 topLeft = vec2(float(packedData.x & 65535u) / 65535.0, float((packedData.x >> 16) & 65535u) / 65535.0);
    vec2 bottomRight = vec2(float(packedData.y & 65535u) / 65535.0, float((packedData.y >> 16) & 65535u) / 65535.0);
    vec2 pos = vec2(float(cornerId >> 1), float(cornerId & 1)) * (bottomRight - topLeft) + topLeft;
    gl_Position = vec4((2.0 * pos - 1.0) * vec2(1.0, -1.0), 0.0, 1.0);
    fragPosition = pos * uResolution;
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
                "@pos" => write!(f, "fragPosition"),
                "@res" => write!(f, "uResolution"),
                v => write!(f, "{}", inputs.get(v).unwrap()),
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
) -> Result<HashMap<String, String>, fmt::Error> {
    let mut result = HashMap::new();

    for i in 0..input.size.div_ceil(16) {
        write!(f, "uvec4 _p{:x}=", i)?;
        fetch(f, i)?;
        write!(f, ";")?;
    }

    for (id, (ident, field)) in input.inputs.iter().enumerate() {
        let expr = emit_decoder_for_type(f, id as u32, field)?;
        result.insert(ident.clone(), expr);
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
    mut write_input: impl FnMut(&mut dyn Write, &str) -> fmt::Result,
    mut write_output: impl FnMut(&mut dyn Write, &str) -> fmt::Result,
) -> fmt::Result {
    // usage analysis
    let usages = {
        let mut usages = HashMap::<Value, u32, _>::new();
        *usages.entry(graph.result()).or_default() += 1;
        for (id, source, _) in graph.iter().rev() {
            if usages.contains_key(&id) {
                source.visit_dependencies(|dep| {
                    *usages.entry(dep).or_default() += 1;
                })
            }
        }
        usages
    };

    let mut atoms = HashMap::new();
    for (id, source, ty) in graph.iter() {
        let usages = usages.get(&id).copied().unwrap_or_default();
        let is_simple = matches!(
            source,
            ValueSource::LitFloat(_) | ValueSource::LitInt(_) | ValueSource::LitBool(_)
        );

        if let ValueSource::Input(ident) = source {
            if ty != ValueType::Texture {
                let mut string = String::new();
                write_input(&mut string, ident)?;
                atoms.insert(id, string);
            }
        } else if is_simple || usages == 1 {
            let mut string = String::new();
            emit_graph_atom(&mut string, graph, source, atlas, |f, value| {
                write!(f, "{}", atoms.get(&value).unwrap())
            })?;
            atoms.insert(id, string);
        } else if usages > 0 {
            let name = format!("_{:x}", id.id());

            write!(f, "{} {}=", type_name(ty), name)?;
            emit_graph_atom(f, graph, source, atlas, |f, value| {
                write!(f, "{}", atoms.get(&value).unwrap())
            })?;
            write!(f, ";")?;

            atoms.insert(id, name);
        }
    }

    write_output(f, atoms.get(&graph.result()).unwrap())?;

    Ok(())
}

fn emit_graph_atom<'a>(
    f: &mut dyn Write,
    graph: &ShaderGraph<Float4>,
    source: &ValueSource,
    atlas: ShaderTextures,
    mut dep: impl FnMut(&mut dyn Write, Value) -> fmt::Result,
) -> fmt::Result {
    match source {
        ValueSource::Input(_) => unreachable!(),

        ValueSource::Add(a, b) => {
            write!(f, "(")?;
            dep(f, *a)?;
            write!(f, "+")?;
            dep(f, *b)?;
            write!(f, ")")?
        }
        ValueSource::Sub(a, b) => {
            write!(f, "(")?;
            dep(f, *a)?;
            write!(f, "-")?;
            dep(f, *b)?;
            write!(f, ")")?
        }
        ValueSource::Mul(a, b) => {
            write!(f, "(")?;
            dep(f, *a)?;
            write!(f, "*")?;
            dep(f, *b)?;
            write!(f, ")")?
        }
        ValueSource::Div(a, b) => {
            write!(f, "(")?;
            dep(f, *a)?;
            write!(f, "/")?;
            dep(f, *b)?;
            write!(f, ")")?
        }
        ValueSource::Rem(a, b) => {
            write!(f, "mod(")?;
            dep(f, *a)?;
            write!(f, ",")?;
            dep(f, *b)?;
            write!(f, ")")?
        }
        ValueSource::Dot(a, b) => {
            write!(f, "dot(")?;
            dep(f, *a)?;
            write!(f, ",")?;
            dep(f, *b)?;
            write!(f, ")")?
        }
        ValueSource::Cross(a, b) => {
            write!(f, "cross(")?;
            dep(f, *a)?;
            write!(f, ",")?;
            dep(f, *b)?;
            write!(f, ")")?
        }
        ValueSource::Neg(a) => {
            write!(f, "(-")?;
            dep(f, *a)?;
            write!(f, ")")?
        }
        ValueSource::Sin(a) => {
            write!(f, "sin(")?;
            dep(f, *a)?;
            write!(f, ")")?;
        }
        ValueSource::Cos(a) => {
            write!(f, "cos(")?;
            dep(f, *a)?;
            write!(f, ")")?;
        }
        ValueSource::Tan(a) => {
            write!(f, "tan(")?;
            dep(f, *a)?;
            write!(f, ")")?;
        }
        ValueSource::Asin(a) => {
            write!(f, "asin(")?;
            dep(f, *a)?;
            write!(f, ")")?;
        }
        ValueSource::Acos(a) => {
            write!(f, "acos(")?;
            dep(f, *a)?;
            write!(f, ")")?;
        }
        ValueSource::Atan(a) => {
            write!(f, "atan(")?;
            dep(f, *a)?;
            write!(f, ")")?;
        }
        ValueSource::Atan2(a, b) => {
            write!(f, "atan(")?;
            dep(f, *a)?;
            write!(f, ",")?;
            dep(f, *b)?;
            write!(f, ")")?;
        }
        ValueSource::Sqrt(a) => {
            write!(f, "sqrt(")?;
            dep(f, *a)?;
            write!(f, ")")?;
        }
        ValueSource::Pow(a, b) => {
            write!(f, "pow(")?;
            dep(f, *a)?;
            write!(f, ",")?;
            dep(f, *b)?;
            write!(f, ")")?;
        }
        ValueSource::Exp(a) => {
            write!(f, "exp(")?;
            dep(f, *a)?;
            write!(f, ")")?;
        }
        ValueSource::Ln(a) => {
            write!(f, "log(")?;
            dep(f, *a)?;
            write!(f, ")")?;
        }
        ValueSource::Min(a, b) => {
            write!(f, "min(")?;
            dep(f, *a)?;
            write!(f, ",")?;
            dep(f, *b)?;
            write!(f, ")")?;
        }
        ValueSource::Max(a, b) => {
            write!(f, "max(")?;
            dep(f, *a)?;
            write!(f, ",")?;
            dep(f, *b)?;
            write!(f, ")")?;
        }
        ValueSource::Clamp(x, min, max) => {
            write!(f, "clamp(")?;
            dep(f, *x)?;
            write!(f, ",")?;
            dep(f, *min)?;
            write!(f, ",")?;
            dep(f, *max)?;
            write!(f, ")")?;
        }
        ValueSource::Abs(a) => {
            write!(f, "abs(")?;
            dep(f, *a)?;
            write!(f, ")")?;
        }
        ValueSource::Sign(a) => {
            write!(f, "sign(")?;
            dep(f, *a)?;
            write!(f, ")")?;
        }
        ValueSource::Floor(a) => {
            write!(f, "floor(")?;
            dep(f, *a)?;
            write!(f, ")")?;
        }
        ValueSource::Fract(a) => {
            write!(f, "fract(")?;
            dep(f, *a)?;
            write!(f, ")")?;
        }
        ValueSource::Lerp(x, min, max) => {
            write!(f, "mix(")?;
            dep(f, *min)?;
            write!(f, ",")?;
            dep(f, *max)?;
            write!(f, ",")?;
            dep(f, *x)?;
            write!(f, ")")?;
        }
        ValueSource::Smoothstep(x, min, max) => {
            write!(f, "smoothstep(")?;
            dep(f, *min)?;
            write!(f, ",")?;
            dep(f, *max)?;
            write!(f, ",")?;
            dep(f, *x)?;
            write!(f, ")")?;
        }
        ValueSource::Step(x, edge) => {
            write!(f, "step(")?;
            dep(f, *edge)?;
            write!(f, ",")?;
            dep(f, *x)?;
            write!(f, ")")?;
        }

        ValueSource::LitFloat(x) if x.is_sign_positive() => write!(f, "{x:?}")?,
        ValueSource::LitFloat(x) => write!(f, "({x:?})")?,

        ValueSource::LitInt(x) if *x >= 0 => write!(f, "{x}")?,
        ValueSource::LitInt(x) => write!(f, "({x})")?,

        ValueSource::LitBool(true) => write!(f, "true")?,
        ValueSource::LitBool(false) => write!(f, "false")?,

        ValueSource::Eq(a, b) => {
            write!(f, "(")?;
            dep(f, *a)?;
            write!(f, "==")?;
            dep(f, *b)?;
            write!(f, ")")?
        }
        ValueSource::Ne(a, b) => {
            write!(f, "(")?;
            dep(f, *a)?;
            write!(f, "!=")?;
            dep(f, *b)?;
            write!(f, ")")?
        }
        ValueSource::Lt(a, b) => {
            write!(f, "(")?;
            dep(f, *a)?;
            write!(f, "<")?;
            dep(f, *b)?;
            write!(f, ")")?
        }
        ValueSource::Le(a, b) => {
            write!(f, "(")?;
            dep(f, *a)?;
            write!(f, "<=")?;
            dep(f, *b)?;
            write!(f, ")")?
        }
        ValueSource::Gt(a, b) => {
            write!(f, "(")?;
            dep(f, *a)?;
            write!(f, ">")?;
            dep(f, *b)?;
            write!(f, ")")?
        }
        ValueSource::Ge(a, b) => {
            write!(f, "(")?;
            dep(f, *a)?;
            write!(f, ">=")?;
            dep(f, *b)?;
            write!(f, ")")?
        }
        ValueSource::And(a, b) => {
            write!(f, "(")?;
            dep(f, *a)?;
            write!(f, "&&")?;
            dep(f, *b)?;
            write!(f, ")")?
        }
        ValueSource::Or(a, b) => {
            write!(f, "(")?;
            dep(f, *a)?;
            write!(f, "||")?;
            dep(f, *b)?;
            write!(f, ")")?
        }
        ValueSource::Xor(a, b) => {
            write!(f, "(")?;
            dep(f, *a)?;
            write!(f, "^^")?;
            dep(f, *b)?;
            write!(f, ")")?
        }
        ValueSource::Not(a) => {
            write!(f, "(!")?;
            dep(f, *a)?;
            write!(f, ")")?
        }

        ValueSource::NewVec2(a, b) => {
            write!(f, "vec2(")?;
            dep(f, *a)?;
            write!(f, ",")?;
            dep(f, *b)?;
            write!(f, ")")?;
        }
        ValueSource::NewVec3(a, b, c) => {
            write!(f, "vec3(")?;
            dep(f, *a)?;
            write!(f, ",")?;
            dep(f, *b)?;
            write!(f, ",")?;
            dep(f, *c)?;
            write!(f, ")")?;
        }
        ValueSource::NewVec4(a, b, c, d) => {
            write!(f, "vec4(")?;
            dep(f, *a)?;
            write!(f, ",")?;
            dep(f, *b)?;
            write!(f, ",")?;
            dep(f, *c)?;
            write!(f, ",")?;
            dep(f, *d)?;
            write!(f, ")")?;
        }
        ValueSource::SplatVec2(a) => {
            write!(f, "vec2(")?;
            dep(f, *a)?;
            write!(f, ")")?;
        }
        ValueSource::SplatVec3(a) => {
            write!(f, "vec3(")?;
            dep(f, *a)?;
            write!(f, ")")?;
        }
        ValueSource::SplatVec4(a) => {
            write!(f, "vec4(")?;
            dep(f, *a)?;
            write!(f, ")")?;
        }
        ValueSource::CastFloat(a) => {
            write!(f, "float(")?;
            dep(f, *a)?;
            write!(f, ")")?;
        }
        ValueSource::CastInt(a) => {
            write!(f, "int(")?;
            dep(f, *a)?;
            write!(f, ")")?;
        }

        ValueSource::Length(a) => {
            write!(f, "length(")?;
            dep(f, *a)?;
            write!(f, ")")?;
        }
        ValueSource::Normalize(a) => {
            write!(f, "normalize(")?;
            dep(f, *a)?;
            write!(f, ")")?;
        }

        ValueSource::Swizzle1(a, Swizzle::X) => {
            dep(f, *a)?;
            write!(f, ".x")?;
        }

        ValueSource::Swizzle1(a, Swizzle::Y) => {
            dep(f, *a)?;
            write!(f, ".y")?;
        }

        ValueSource::Swizzle1(a, Swizzle::Z) => {
            dep(f, *a)?;
            write!(f, ".z")?;
        }

        ValueSource::Swizzle1(a, Swizzle::W) => {
            dep(f, *a)?;
            write!(f, ".w")?;
        }

        ValueSource::DerivX(a) => {
            write!(f, "dFdx(")?;
            dep(f, *a)?;
            write!(f, ")")?;
        }
        ValueSource::DerivY(a) => {
            write!(f, "dFdy(")?;
            dep(f, *a)?;
            write!(f, ")")?;
        }
        ValueSource::DerivWidth(a) => {
            write!(f, "fwidth(")?;
            dep(f, *a)?;
            write!(f, ")")?;
        }

        ValueSource::TextureSampleLinear(index, b) => {
            let texture = match graph.get(*index) {
                (ValueSource::Input(id), _) => atlas.get(id),
                _ => unreachable!(),
            };

            write!(
                f,
                "texture(uAtlas,vec2({}.0,{}.0)+clamp(0.5+",
                texture.x, texture.y
            )?;
            dep(f, *b)?;
            write!(
                f,
                "{},vec2(0.0),vec2({}.0,{}.0))/{}.0)",
                if texture.rotated { ".yx" } else { "" },
                texture.data.width(),
                texture.data.height(),
                atlas.atlas.size
            )?;
        }

        ValueSource::TextureSampleNearest(index, b) => {
            let texture = match graph.get(*index) {
                (ValueSource::Input(id), _) => atlas.get(id),
                _ => unreachable!(),
            };

            write!(
                f,
                "texelFetch(uAtlas,ivec2({},{})+clamp(ivec2(",
                texture.x, texture.y
            )?;
            dep(f, *b)?;
            write!(
                f,
                "){},ivec2(0),ivec2({},{})),0)",
                if texture.rotated { ".yx" } else { "" },
                texture.data.width(),
                texture.data.height()
            )?;
        }

        ValueSource::TextureSize(index) => {
            let texture = match graph.get(*index) {
                (ValueSource::Input(id), _) => atlas.get(id),
                _ => unreachable!(),
            };

            write!(
                f,
                "vec2({}.0,{}.0)",
                texture.data.width(),
                texture.data.height()
            )?;
        }
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
