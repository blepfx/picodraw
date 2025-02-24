use crate::graph::*;
use crate::shader::*;
use crate::*;

pub fn read<T: ShaderData>() -> T::Data {
    T::read()
}

/// Write a value to the shader output.
pub fn write_color(color: float4) {
    Graph::push_collect(Op::Output(color.0));
}

/// Get the current fragment position in physical pixels
pub fn position() -> float2 {
    types::float2(Graph::push_collect(Op::Position))
}

/// Get the current frame resolution in physical pixels.
///
/// Should be called inside of `ShaderGraph::collect`
pub fn resolution() -> float2 {
    types::float2(Graph::push_collect(Op::Resolution))
}

/// Get the current quad bounds in physical pixels.
/// Returns the position of the top left and bottom right corners.
///
/// Should be called inside of `ShaderGraph::collect`
pub fn bounds() -> (float2, float2) {
    let start = types::float2(Graph::push_collect(Op::QuadStart));
    let end = types::float2(Graph::push_collect(Op::QuadEnd));

    (start, end)
}

pub trait ShaderDataWriter {
    fn write_i32(&mut self, x: i32);
    fn write_f32(&mut self, x: f32);
    fn write_texture_static(&mut self, texture: Texture);
    fn write_texture_render(&mut self, texture: RenderTexture);

    fn resolution(&self) -> Size;
    fn quad_bounds(&self) -> Bounds;
}

pub trait ShaderData {
    type Data;

    fn read() -> Self::Data;
    fn write(&self, writer: &mut dyn ShaderDataWriter);
}

impl ShaderData for () {
    type Data = ();

    fn read() -> Self::Data {
        ()
    }

    fn write(&self, _: &mut dyn ShaderDataWriter) {}
}

impl ShaderData for bool {
    type Data = boolean;
    fn read() -> Self::Data {
        u8::read().ne(0)
    }

    fn write(&self, writer: &mut dyn ShaderDataWriter) {
        writer.write_i32(*self as i32);
    }
}

impl ShaderData for i8 {
    type Data = int1;

    fn read() -> Self::Data {
        types::int1(Graph::push_collect(Op::Input(OpInput::I8)))
    }

    fn write(&self, writer: &mut dyn ShaderDataWriter) {
        writer.write_i32(*self as i32);
    }
}

impl ShaderData for i16 {
    type Data = int1;

    fn read() -> Self::Data {
        types::int1(Graph::push_collect(Op::Input(OpInput::I16)))
    }

    fn write(&self, writer: &mut dyn ShaderDataWriter) {
        writer.write_i32(*self as i32);
    }
}

impl ShaderData for i32 {
    type Data = int1;

    fn read() -> Self::Data {
        types::int1(Graph::push_collect(Op::Input(OpInput::I32)))
    }

    fn write(&self, writer: &mut dyn ShaderDataWriter) {
        writer.write_i32(*self as i32);
    }
}

impl ShaderData for u8 {
    type Data = int1;

    fn read() -> Self::Data {
        types::int1(Graph::push_collect(Op::Input(OpInput::U8)))
    }

    fn write(&self, writer: &mut dyn ShaderDataWriter) {
        writer.write_i32(*self as i32);
    }
}

impl ShaderData for u16 {
    type Data = int1;

    fn read() -> Self::Data {
        types::int1(Graph::push_collect(Op::Input(OpInput::U16)))
    }

    fn write(&self, writer: &mut dyn ShaderDataWriter) {
        writer.write_i32(*self as i32);
    }
}

impl ShaderData for u32 {
    type Data = int1;

    fn read() -> Self::Data {
        types::int1(Graph::push_collect(Op::Input(OpInput::U32)))
    }

    fn write(&self, writer: &mut dyn ShaderDataWriter) {
        writer.write_i32(*self as i32);
    }
}

impl ShaderData for f32 {
    type Data = float1;

    fn read() -> Self::Data {
        types::float1(Graph::push_collect(Op::Input(OpInput::F32)))
    }

    fn write(&self, writer: &mut dyn ShaderDataWriter) {
        writer.write_f32(*self);
    }
}

impl ShaderData for f64 {
    type Data = float1;

    fn read() -> Self::Data {
        types::float1(Graph::push_collect(Op::Input(OpInput::F32)))
    }

    fn write(&self, writer: &mut dyn ShaderDataWriter) {
        writer.write_f32(*self as f32);
    }
}

impl ShaderData for RenderTexture {
    type Data = texture;

    fn read() -> Self::Data {
        types::texture(Graph::push_collect(Op::Input(OpInput::TextureRender)))
    }

    fn write(&self, writer: &mut dyn ShaderDataWriter) {
        writer.write_texture_render(*self);
    }
}

impl ShaderData for Texture {
    type Data = texture;

    fn read() -> Self::Data {
        types::texture(Graph::push_collect(Op::Input(OpInput::TextureStatic)))
    }

    fn write(&self, writer: &mut dyn ShaderDataWriter) {
        writer.write_texture_static(*self);
    }
}

impl<'a, T: ShaderData> ShaderData for &'a T {
    type Data = T::Data;

    fn read() -> Self::Data {
        T::read()
    }

    fn write(&self, writer: &mut dyn ShaderDataWriter) {
        T::write(&self, writer);
    }
}

impl<const N: usize, T: ShaderData> ShaderData for [T; N] {
    type Data = [T::Data; N];

    fn read() -> Self::Data {
        std::array::from_fn(|_| T::read())
    }

    fn write(&self, writer: &mut dyn ShaderDataWriter) {
        for i in 0..N {
            self[i].write(writer);
        }
    }
}

macro_rules! impl_tuple {
    ($($id:ident),*) => {
        impl<$($id: ShaderData),*> ShaderData for ($($id,)*) {
            type Data = ($($id::Data,)*);

            fn read() -> Self::Data {
                ($($id::read(),)*)
            }

            fn write(&self, writer: &mut dyn ShaderDataWriter) {
                #[allow(non_snake_case)]
                let ($($id,)*) = self;
                $($id.write(writer);)*
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
