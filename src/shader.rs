use crate::{Float2, Float4, ShaderData};
use std::{any::TypeId, ops::Deref, u16};

pub struct ShaderContext<'a, T> {
    pub vars: &'a T,
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

impl<'a, T> Deref for ShaderContext<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.vars
    }
}

pub trait Shader: ShaderData {
    fn id() -> TypeId {
        fn id<T: 'static>(_: T) -> TypeId {
            TypeId::of::<T>()
        }

        id(|x| Self::draw(x))
    }

    fn draw(shader: ShaderContext<Self::ShaderVars>) -> Float4;
}

impl<'a, T: Shader> Shader for &'a T {
    fn draw(shader: ShaderContext<Self::ShaderVars>) -> Float4 {
        T::draw(shader)
    }

    fn id() -> TypeId {
        T::id()
    }
}
