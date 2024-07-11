use crate::*;
use std::{array::from_fn, sync::Arc};

pub trait ShaderData {
    type ShaderVars;
    fn shader_vars(vars: &mut dyn ShaderVars) -> Self::ShaderVars;
    fn write(&self, writer: &mut dyn ShaderDataWriter);
}

pub trait ShaderVars {
    fn int8(&mut self, id: &str) -> Int;
    fn int16(&mut self, id: &str) -> Int;
    fn int32(&mut self, id: &str) -> Int;
    fn uint8(&mut self, id: &str) -> Int;
    fn uint16(&mut self, id: &str) -> Int;
    fn uint32(&mut self, id: &str) -> Int;
    fn float(&mut self, id: &str) -> Float;
    fn texture(&mut self, tex: Arc<dyn Fn() -> image::DynamicImage>) -> Texture;

    fn position(&mut self) -> Float2;
    fn resolution(&mut self) -> Float2;
}

pub trait ShaderDataWriter {
    fn write_float(&mut self, id: &str, x: f32);
    fn write_int(&mut self, id: &str, x: i32);
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
        vars.uint8("").neq(0)
    }
    fn write(&self, writer: &mut dyn ShaderDataWriter) {
        writer.write_int("", if *self { 1 } else { 0 })
    }
}

impl ShaderData for u8 {
    type ShaderVars = Int;
    fn shader_vars(vars: &mut dyn ShaderVars) -> Self::ShaderVars {
        vars.uint8("")
    }
    fn write(&self, writer: &mut dyn ShaderDataWriter) {
        writer.write_int("", *self as i32)
    }
}

impl ShaderData for u16 {
    type ShaderVars = Int;
    fn shader_vars(vars: &mut dyn ShaderVars) -> Self::ShaderVars {
        vars.uint16("")
    }
    fn write(&self, writer: &mut dyn ShaderDataWriter) {
        writer.write_int("", *self as i32)
    }
}

impl ShaderData for u32 {
    type ShaderVars = Int;
    fn shader_vars(vars: &mut dyn ShaderVars) -> Self::ShaderVars {
        vars.uint32("")
    }
    fn write(&self, writer: &mut dyn ShaderDataWriter) {
        writer.write_int("", *self as i32)
    }
}

impl ShaderData for i8 {
    type ShaderVars = Int;
    fn shader_vars(vars: &mut dyn ShaderVars) -> Self::ShaderVars {
        vars.int8("")
    }
    fn write(&self, writer: &mut dyn ShaderDataWriter) {
        writer.write_int("", *self as i32)
    }
}

impl ShaderData for i16 {
    type ShaderVars = Int;
    fn shader_vars(vars: &mut dyn ShaderVars) -> Self::ShaderVars {
        vars.int16("")
    }
    fn write(&self, writer: &mut dyn ShaderDataWriter) {
        writer.write_int("", *self as i32)
    }
}

impl ShaderData for i32 {
    type ShaderVars = Int;
    fn shader_vars(vars: &mut dyn ShaderVars) -> Self::ShaderVars {
        vars.int32("")
    }
    fn write(&self, writer: &mut dyn ShaderDataWriter) {
        writer.write_int("", *self as i32)
    }
}

impl ShaderData for f32 {
    type ShaderVars = Float;
    fn shader_vars(vars: &mut dyn ShaderVars) -> Self::ShaderVars {
        vars.float("")
    }
    fn write(&self, writer: &mut dyn ShaderDataWriter) {
        writer.write_float("", *self)
    }
}

impl ShaderData for f64 {
    type ShaderVars = Float;
    fn shader_vars(vars: &mut dyn ShaderVars) -> Self::ShaderVars {
        vars.float("")
    }
    fn write(&self, writer: &mut dyn ShaderDataWriter) {
        writer.write_float("", *self as f32)
    }
}

impl<const N: usize, T: ShaderData> ShaderData for [T; N] {
    type ShaderVars = [T::ShaderVars; N];

    fn shader_vars(vars: &mut dyn ShaderVars) -> Self::ShaderVars {
        from_fn(|i| T::shader_vars(&mut prefix_vars(vars, &format!("{}", i))))
    }

    fn write(&self, writer: &mut dyn ShaderDataWriter) {
        for i in 0..N {
            self[i].write(&mut prefix_writer(writer, &format!("{}", i)));
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
                ($($id::shader_vars(&mut prefix_vars(vars, stringify!($id))),)*)
            }

            fn write(&self, writer: &mut dyn ShaderDataWriter) {
                #[allow(non_snake_case)]
                let ($($id,)*) = self;
                $($id::write($id, &mut prefix_writer(writer, stringify!($id)));)*
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

pub struct ShaderVarsPrefix<'a>(&'a mut dyn ShaderVars, &'a str);
pub struct ShaderWriterPrefix<'a>(&'a mut dyn ShaderDataWriter, &'a str);

pub fn prefix_vars<'a>(vars: &'a mut dyn ShaderVars, prefix: &'a str) -> ShaderVarsPrefix<'a> {
    ShaderVarsPrefix(vars, prefix)
}

pub fn prefix_writer<'a>(
    writer: &'a mut dyn ShaderDataWriter,
    prefix: &'a str,
) -> ShaderWriterPrefix<'a> {
    ShaderWriterPrefix(writer, prefix)
}

impl<'a> ShaderVars for ShaderVarsPrefix<'a> {
    fn int8(&mut self, id: &str) -> Int {
        self.0.int8(&format!("{}/{}", self.1, id))
    }

    fn int16(&mut self, id: &str) -> Int {
        self.0.int16(&format!("{}/{}", self.1, id))
    }

    fn int32(&mut self, id: &str) -> Int {
        self.0.int32(&format!("{}/{}", self.1, id))
    }

    fn uint8(&mut self, id: &str) -> Int {
        self.0.uint8(&format!("{}/{}", self.1, id))
    }

    fn uint16(&mut self, id: &str) -> Int {
        self.0.uint16(&format!("{}/{}", self.1, id))
    }

    fn uint32(&mut self, id: &str) -> Int {
        self.0.uint32(&format!("{}/{}", self.1, id))
    }

    fn float(&mut self, id: &str) -> Float {
        self.0.float(&format!("{}/{}", self.1, id))
    }

    fn texture(&mut self, tex: Arc<dyn Fn() -> image::DynamicImage>) -> Texture {
        self.0.texture(tex)
    }

    fn position(&mut self) -> Float2 {
        self.0.position()
    }

    fn resolution(&mut self) -> Float2 {
        self.0.resolution()
    }
}

impl<'a> ShaderDataWriter for ShaderWriterPrefix<'a> {
    fn write_float(&mut self, id: &str, x: f32) {
        self.0.write_float(&format!("{}/{}", self.1, id), x);
    }

    fn write_int(&mut self, id: &str, x: i32) {
        self.0.write_int(&format!("{}/{}", self.1, id), x);
    }
}
