pub mod opengl;

mod data;
mod graph;
mod types;

pub use data::{prefix_vars, prefix_writer, ShaderData, ShaderDataWriter, ShaderVars};
pub use image;
pub use picodraw_derive::ShaderData;
pub use types::{Bool, Float, Float2, Float3, Float4, GlFloat, Int, Texture};

#[non_exhaustive]
pub struct ShaderContext<T> {
    pub vars: T,
    pub position: Float2,
    pub resolution: Float2,
    pub bounds: Float4,
}

impl<T> std::ops::Deref for ShaderContext<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.vars
    }
}

pub trait Shader: ShaderData {
    fn bounds(&self) -> [f32; 4];
    fn draw(shader: ShaderContext<Self::ShaderVars>) -> Float4;
}
