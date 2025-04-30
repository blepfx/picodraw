use crate::graph::*;
use crate::shader::*;
use crate::*;

/// Read the shader data.
///
/// Should be called inside of [`Graph::collect`]
pub fn read<T: ShaderData>() -> T::Data {
    T::read()
}

/// Get the current fragment position in physical pixels
///
/// Should be called inside of [`Graph::collect`]
pub fn position() -> float2 {
    types::float2(Graph::push_collect(OpValue::Position))
}

/// Get the current frame resolution in physical pixels.
///
/// Should be called inside of [`Graph::collect`]
pub fn resolution() -> float2 {
    types::float2(Graph::push_collect(OpValue::Resolution))
}

/// Get the current quad bounds in physical pixels.
/// Returns the position of the top left and bottom right corners.
///
/// Should be called inside of [`Graph::collect`]
pub fn bounds() -> (float2, float2) {
    let start = types::float2(Graph::push_collect(OpValue::QuadStart));
    let end = types::float2(Graph::push_collect(OpValue::QuadEnd));

    (start, end)
}

/// Arbitrary data that is serializable and readable by a shader.
/// Shader data can be different per each rendered quad.
///
/// Use [`CommandBufferQuad`] (which implements [`ShaderDataWriter`]) to write data to the shader
/// that can be read in the shader graph context by [`io::read`] or [`ShaderData::read`].
pub trait ShaderData {
    type Data;

    /// Read the data in the shader graph context.
    ///
    /// The data should be read in the same order as it was written, failure to do so may result in backend implementation defined behavior (reading garbage data or panics, it shoult NOT cause _undefined behavior_)
    fn read() -> Self::Data;

    /// Serialize the object to a given [`ShaderDataWriter`].
    ///
    /// The data should be read in the same order as it was written, failure to do so may result in backend implementation defined behavior (reading garbage data or panics, it shoult NOT cause _undefined behavior_)
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
        types::int1(Graph::push_collect(OpValue::Input(OpInput::I8)))
    }

    fn write(&self, writer: &mut dyn ShaderDataWriter) {
        writer.write_i32(*self as i32);
    }
}

impl ShaderData for i16 {
    type Data = int1;

    fn read() -> Self::Data {
        types::int1(Graph::push_collect(OpValue::Input(OpInput::I16)))
    }

    fn write(&self, writer: &mut dyn ShaderDataWriter) {
        writer.write_i32(*self as i32);
    }
}

impl ShaderData for i32 {
    type Data = int1;

    fn read() -> Self::Data {
        types::int1(Graph::push_collect(OpValue::Input(OpInput::I32)))
    }

    fn write(&self, writer: &mut dyn ShaderDataWriter) {
        writer.write_i32(*self as i32);
    }
}

impl ShaderData for u8 {
    type Data = int1;

    fn read() -> Self::Data {
        types::int1(Graph::push_collect(OpValue::Input(OpInput::U8)))
    }

    fn write(&self, writer: &mut dyn ShaderDataWriter) {
        writer.write_i32(*self as i32);
    }
}

impl ShaderData for u16 {
    type Data = int1;

    fn read() -> Self::Data {
        types::int1(Graph::push_collect(OpValue::Input(OpInput::U16)))
    }

    fn write(&self, writer: &mut dyn ShaderDataWriter) {
        writer.write_i32(*self as i32);
    }
}

impl ShaderData for u32 {
    type Data = int1;

    fn read() -> Self::Data {
        types::int1(Graph::push_collect(OpValue::Input(OpInput::I32)))
    }

    fn write(&self, writer: &mut dyn ShaderDataWriter) {
        writer.write_i32(*self as i32);
    }
}

impl ShaderData for f32 {
    type Data = float1;

    fn read() -> Self::Data {
        types::float1(Graph::push_collect(OpValue::Input(OpInput::F32)))
    }

    fn write(&self, writer: &mut dyn ShaderDataWriter) {
        writer.write_f32(*self);
    }
}

impl ShaderData for f64 {
    type Data = float1;

    fn read() -> Self::Data {
        types::float1(Graph::push_collect(OpValue::Input(OpInput::F32)))
    }

    fn write(&self, writer: &mut dyn ShaderDataWriter) {
        writer.write_f32(*self as f32);
    }
}

impl ShaderData for RenderTexture {
    type Data = texture;

    fn read() -> Self::Data {
        types::texture(Graph::push_collect(OpValue::Input(OpInput::TextureRender)))
    }

    fn write(&self, writer: &mut dyn ShaderDataWriter) {
        writer.write_texture_render(*self);
    }
}

impl ShaderData for Texture {
    type Data = texture;

    fn read() -> Self::Data {
        types::texture(Graph::push_collect(OpValue::Input(OpInput::TextureStatic)))
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
impl_tuple!(A, B, C, D, E, F, G);
impl_tuple!(A, B, C, D, E, F, G, H);
