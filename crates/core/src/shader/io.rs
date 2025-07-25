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
    types::float2(Graph::push_scope(OpValue::Position).unwrap())
}

/// Get the current frame resolution in physical pixels.
///
/// Should be called inside of [`Graph::collect`]
pub fn resolution() -> float2 {
    types::float2(Graph::push_scope(OpValue::Resolution).unwrap())
}

/// Get the current quad bounds in physical pixels.
/// Returns the position of the top left and bottom right corners.
///
/// Should be called inside of [`Graph::collect`]
pub fn bounds() -> (float2, float2) {
    let start = types::float2(Graph::push_scope(OpValue::QuadStart).unwrap());
    let end = types::float2(Graph::push_scope(OpValue::QuadEnd).unwrap());

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
    fn write(&self, writer: impl ShaderDataWriter);
}

pub trait ShaderDataWriter {
    fn write_i32(&mut self, value: i32);
    fn write_f32(&mut self, value: f32);
    fn write_texture_static(&mut self, value: Texture);
    fn write_texture_render(&mut self, value: RenderTexture);
}

impl<'a, T: ShaderDataWriter> ShaderDataWriter for &'a mut T {
    #[inline]
    fn write_i32(&mut self, value: i32) {
        (*self).write_i32(value);
    }

    #[inline]
    fn write_f32(&mut self, value: f32) {
        (*self).write_f32(value);
    }

    #[inline]
    fn write_texture_static(&mut self, value: Texture) {
        (*self).write_texture_static(value);
    }

    #[inline]
    fn write_texture_render(&mut self, value: RenderTexture) {
        (*self).write_texture_render(value);
    }
}

impl ShaderDataWriter for Vec<Command> {
    fn write_i32(&mut self, value: i32) {
        self.push(Command::WriteInt(value));
    }

    fn write_f32(&mut self, value: f32) {
        self.push(Command::WriteFloat(value));
    }

    fn write_texture_static(&mut self, value: Texture) {
        self.push(Command::WriteStaticTexture(value));
    }

    fn write_texture_render(&mut self, value: RenderTexture) {
        self.push(Command::WriteRenderTexture(value));
    }
}

impl ShaderData for () {
    type Data = ();

    fn read() -> Self::Data {
        ()
    }

    fn write(&self, _: impl ShaderDataWriter) {}
}

impl ShaderData for bool {
    type Data = boolean;

    fn read() -> Self::Data {
        u8::read().ne(0)
    }

    #[inline]
    fn write(&self, mut writer: impl ShaderDataWriter) {
        writer.write_i32(*self as i32);
    }
}

impl ShaderData for i8 {
    type Data = int1;

    fn read() -> Self::Data {
        types::int1(Graph::push_scope(OpValue::Input(OpInput::I8)).unwrap())
    }

    #[inline]
    fn write(&self, mut writer: impl ShaderDataWriter) {
        writer.write_i32(*self as i32);
    }
}

impl ShaderData for i16 {
    type Data = int1;

    fn read() -> Self::Data {
        types::int1(Graph::push_scope(OpValue::Input(OpInput::I16)).unwrap())
    }

    #[inline]
    fn write(&self, mut writer: impl ShaderDataWriter) {
        writer.write_i32(*self as i32);
    }
}

impl ShaderData for i32 {
    type Data = int1;

    fn read() -> Self::Data {
        types::int1(Graph::push_scope(OpValue::Input(OpInput::I32)).unwrap())
    }

    #[inline]
    fn write(&self, mut writer: impl ShaderDataWriter) {
        writer.write_i32(*self as i32);
    }
}

impl ShaderData for u8 {
    type Data = int1;

    fn read() -> Self::Data {
        types::int1(Graph::push_scope(OpValue::Input(OpInput::U8)).unwrap())
    }

    #[inline]
    fn write(&self, mut writer: impl ShaderDataWriter) {
        writer.write_i32(*self as i32);
    }
}

impl ShaderData for u16 {
    type Data = int1;

    fn read() -> Self::Data {
        types::int1(Graph::push_scope(OpValue::Input(OpInput::U16)).unwrap())
    }

    #[inline]
    fn write(&self, mut writer: impl ShaderDataWriter) {
        writer.write_i32(*self as i32);
    }
}

impl ShaderData for u32 {
    type Data = int1;

    fn read() -> Self::Data {
        types::int1(Graph::push_scope(OpValue::Input(OpInput::I32)).unwrap())
    }

    #[inline]
    fn write(&self, mut writer: impl ShaderDataWriter) {
        writer.write_i32(*self as i32);
    }
}

impl ShaderData for f32 {
    type Data = float1;

    fn read() -> Self::Data {
        types::float1(Graph::push_scope(OpValue::Input(OpInput::F32)).unwrap())
    }

    #[inline]
    fn write(&self, mut writer: impl ShaderDataWriter) {
        writer.write_f32(*self);
    }
}

impl ShaderData for f64 {
    type Data = float1;

    fn read() -> Self::Data {
        types::float1(Graph::push_scope(OpValue::Input(OpInput::F32)).unwrap())
    }

    #[inline]
    fn write(&self, mut writer: impl ShaderDataWriter) {
        writer.write_f32(*self as f32);
    }
}

impl ShaderData for RenderTexture {
    type Data = texture;

    fn read() -> Self::Data {
        types::texture(Graph::push_scope(OpValue::Input(OpInput::TextureRender)).unwrap())
    }

    #[inline]
    fn write(&self, mut writer: impl ShaderDataWriter) {
        writer.write_texture_render(*self);
    }
}

impl ShaderData for Texture {
    type Data = texture;

    fn read() -> Self::Data {
        types::texture(Graph::push_scope(OpValue::Input(OpInput::TextureStatic)).unwrap())
    }

    #[inline]
    fn write(&self, mut writer: impl ShaderDataWriter) {
        writer.write_texture_static(*self);
    }
}

impl<'a, T: ShaderData> ShaderData for &'a T {
    type Data = T::Data;

    fn read() -> Self::Data {
        T::read()
    }

    #[inline]
    fn write(&self, writer: impl ShaderDataWriter) {
        T::write(&self, writer);
    }
}

impl<const N: usize, T: ShaderData> ShaderData for [T; N] {
    type Data = [T::Data; N];

    fn read() -> Self::Data {
        std::array::from_fn(|_| T::read())
    }

    #[inline]
    fn write(&self, mut writer: impl ShaderDataWriter) {
        for i in 0..N {
            self[i].write(&mut writer);
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

            #[inline]
            fn write(&self, mut writer: impl ShaderDataWriter) {
                #[allow(non_snake_case)]
                let ($($id,)*) = self;
                $($id.write(&mut writer);)*
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
