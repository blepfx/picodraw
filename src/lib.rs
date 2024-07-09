pub mod opengl;

mod data;
mod graph;
mod types;

pub use data::{prefix_vars, prefix_writer, ShaderData, ShaderDataWriter, ShaderVars};
pub use image;
pub use picodraw_derive::ShaderData;
pub use types::{Bool, Float, Float2, Float3, Float4, GlFloat, Int, Texture};

pub trait Shader: ShaderData {
    fn id() -> &'static str;
    fn bounds(&self) -> [f32; 4];
    fn draw(pos: Float2, vars: Self::ShaderVars) -> Float4;
}
