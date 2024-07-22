use crate::{Float2, Float4, ShaderData};
use std::{
    ops::{Deref, DerefMut},
    u16,
};

#[non_exhaustive]
pub struct ShaderContext<T> {
    pub vars: T,
    pub position: Float2,
    pub resolution: Float2,
    pub bounds: Float4,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Bounds {
    pub top: u16,
    pub left: u16,
    pub bottom: u16,
    pub right: u16,
}

impl Bounds {
    pub fn infinite() -> Self {
        Self {
            top: 0,
            left: 0,
            bottom: u16::MAX,
            right: u16::MAX,
        }
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
