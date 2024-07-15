use crate::{Float2, Float4, ShaderData};
use std::ops::{Deref, DerefMut};

#[non_exhaustive]
pub struct ShaderContext<T> {
    pub vars: T,
    pub position: Float2,
    pub resolution: Float2,
    pub bounds: Float4,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Bounds {
    pub top: f32,
    pub left: f32,
    pub bottom: f32,
    pub right: f32,
}

impl Bounds {
    pub fn infinite() -> Self {
        Self {
            top: f32::NEG_INFINITY,
            left: f32::NEG_INFINITY,
            bottom: f32::INFINITY,
            right: f32::INFINITY,
        }
    }

    pub fn extend(self, a: f32) -> Self {
        Self {
            top: self.top - a,
            left: self.left - a,
            bottom: self.bottom + a,
            right: self.right + a,
        }
    }

    pub fn width(&self) -> f32 {
        self.right - self.left
    }

    pub fn height(&self) -> f32 {
        self.bottom - self.top
    }
}

impl<T> Deref for ShaderContext<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.vars
    }
}

impl<T> DerefMut for ShaderContext<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.vars
    }
}

pub trait Shader: ShaderData {
    fn draw(shader: ShaderContext<Self::ShaderVars>) -> Float4;
}
