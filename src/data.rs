use crate::*;
use std::{array::from_fn, sync::Arc};

pub trait ShaderData {
    type ShaderVars;

    fn shader_vars(vars: &mut dyn ShaderVars) -> Self::ShaderVars;
    fn write(&self, writer: &mut dyn ShaderDataWriter);
}

pub trait ShaderVars {
    fn read_int8(&mut self) -> Int;
    fn read_int16(&mut self) -> Int;
    fn read_int32(&mut self) -> Int;
    fn read_uint8(&mut self) -> Int;
    fn read_uint16(&mut self) -> Int;
    fn read_uint32(&mut self) -> Int;
    fn read_float(&mut self) -> Float;
    fn texture(&mut self, tex: Arc<dyn Fn() -> image::DynamicImage>) -> Texture;
    fn resolution(&mut self) -> Float2;
}

pub trait ShaderDataWriter {
    fn resolution(&self) -> (f32, f32);
    fn write_float(&mut self, x: f32);
    fn write_int(&mut self, x: i32);
}

impl ShaderData for () {
    type ShaderVars = ();
    fn shader_vars(_vars: &mut dyn ShaderVars) -> Self::ShaderVars {
        ()
    }
    fn write(&self, _writer: &mut dyn ShaderDataWriter) {}
}

impl ShaderData for bool {
    type ShaderVars = Bool;
    fn shader_vars(vars: &mut dyn ShaderVars) -> Self::ShaderVars {
        vars.read_uint8().neq(0)
    }
    fn write(&self, writer: &mut dyn ShaderDataWriter) {
        writer.write_int(if *self { 1 } else { 0 })
    }
}

impl ShaderData for u8 {
    type ShaderVars = Int;
    fn shader_vars(vars: &mut dyn ShaderVars) -> Self::ShaderVars {
        vars.read_uint8()
    }
    fn write(&self, writer: &mut dyn ShaderDataWriter) {
        writer.write_int(*self as i32)
    }
}

impl ShaderData for u16 {
    type ShaderVars = Int;
    fn shader_vars(vars: &mut dyn ShaderVars) -> Self::ShaderVars {
        vars.read_uint16()
    }
    fn write(&self, writer: &mut dyn ShaderDataWriter) {
        writer.write_int(*self as i32)
    }
}

impl ShaderData for u32 {
    type ShaderVars = Int;
    fn shader_vars(vars: &mut dyn ShaderVars) -> Self::ShaderVars {
        vars.read_uint32()
    }
    fn write(&self, writer: &mut dyn ShaderDataWriter) {
        writer.write_int(*self as i32)
    }
}

impl ShaderData for i8 {
    type ShaderVars = Int;
    fn shader_vars(vars: &mut dyn ShaderVars) -> Self::ShaderVars {
        vars.read_int8()
    }
    fn write(&self, writer: &mut dyn ShaderDataWriter) {
        writer.write_int(*self as i32)
    }
}

impl ShaderData for i16 {
    type ShaderVars = Int;
    fn shader_vars(vars: &mut dyn ShaderVars) -> Self::ShaderVars {
        vars.read_int16()
    }
    fn write(&self, writer: &mut dyn ShaderDataWriter) {
        writer.write_int(*self as i32)
    }
}

impl ShaderData for i32 {
    type ShaderVars = Int;
    fn shader_vars(vars: &mut dyn ShaderVars) -> Self::ShaderVars {
        vars.read_int32()
    }
    fn write(&self, writer: &mut dyn ShaderDataWriter) {
        writer.write_int(*self as i32)
    }
}

impl ShaderData for f32 {
    type ShaderVars = Float;
    fn shader_vars(vars: &mut dyn ShaderVars) -> Self::ShaderVars {
        vars.read_float()
    }
    fn write(&self, writer: &mut dyn ShaderDataWriter) {
        writer.write_float(*self)
    }
}

impl ShaderData for f64 {
    type ShaderVars = Float;
    fn shader_vars(vars: &mut dyn ShaderVars) -> Self::ShaderVars {
        vars.read_float()
    }
    fn write(&self, writer: &mut dyn ShaderDataWriter) {
        writer.write_float(*self as f32)
    }
}

impl<const N: usize, T: ShaderData> ShaderData for [T; N] {
    type ShaderVars = [T::ShaderVars; N];

    fn shader_vars(vars: &mut dyn ShaderVars) -> Self::ShaderVars {
        from_fn(|_| T::shader_vars(vars))
    }

    fn write(&self, writer: &mut dyn ShaderDataWriter) {
        for i in 0..N {
            self[i].write(writer);
        }
    }
}

impl<'a, T: ShaderData> ShaderData for &'a T {
    type ShaderVars = T::ShaderVars;
    fn shader_vars(vars: &mut dyn ShaderVars) -> Self::ShaderVars {
        T::shader_vars(vars)
    }
    fn write(&self, writer: &mut dyn ShaderDataWriter) {
        T::write(&self, writer)
    }
}

macro_rules! impl_tuple {
    ($($id:ident),*) => {
        impl<$($id: ShaderData),*> ShaderData for ($($id,)*) {
            type ShaderVars = ($($id::ShaderVars,)*);

            fn shader_vars(vars: &mut dyn ShaderVars) -> Self::ShaderVars {
                ($($id::shader_vars(vars),)*)
            }

            fn write(&self, writer: &mut dyn ShaderDataWriter) {
                #[allow(non_snake_case)]
                let ($($id,)*) = self;
                $($id::write($id, writer);)*
            }
        }
    };
}

impl_tuple!(A);
impl_tuple!(A, B);
impl_tuple!(A, B, C);
impl_tuple!(A, B, C, D);
impl_tuple!(A, B, C, D, E);
impl_tuple!(A, B, C, D, E, F);
