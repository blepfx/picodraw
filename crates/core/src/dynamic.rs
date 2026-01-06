use crate::{
    Bounds, Color, Context, DrawTarget, FrameEncoder, ShaderData, ShaderError, Size, TextureData, TextureError,
    TextureFormat,
};
use std::any::Any;

/// A type-erased texture handle. Used with [`DynContext`].
pub struct DynTexture(Box<dyn Any>);

/// A type-erased shader handle. Used with [`DynContext`].
pub struct DynShader(Box<dyn Any>);

/// A type-erased [`Context`] trait.
///
/// This trait is used to abstract over different context implementations using dynamic dispatch.
pub trait DynContext {
    /// See [`Context::create_texture`]
    fn create_texture(&mut self, size: Size, format: TextureFormat) -> Result<DynTexture, TextureError>;

    /// See [`Context::upload_texture`]
    ///
    /// # Panics
    /// This function will panic if the texture passed belongs to a different context implementation.
    fn upload_texture(&mut self, texture: &mut DynTexture, data: TextureData) -> Result<(), TextureError>;

    /// See [`Context::delete_texture`]
    ///
    /// # Panics
    /// This function will panic if the texture passed belongs to a different context implementation.
    fn delete_texture(&mut self, texture: DynTexture);

    /// See [`Context::create_shader`]
    fn create_shader(&mut self, shader: &ShaderData) -> Result<DynShader, ShaderError>;

    /// See [`Context::delete_shader`]
    ///
    /// # Panics
    /// This function will panic if the shader passed belongs to a different context implementation.
    fn delete_shader(&mut self, shader: DynShader);

    /// See [`Context::draw`]
    ///
    /// # Panics
    /// This function will panic if the resources passed belong to a different context implementation.
    fn draw_dyn<'a>(
        &'a mut self,
        target: DrawTarget<'a, DynTexture>,
        f: &mut dyn FnMut(&mut dyn FrameEncoder<'a, Shader = DynShader, Texture = DynTexture>),
    );
}

impl<'a> dyn DynContext + 'a {
    /// See [`Context::draw`]
    ///
    /// # Panics
    /// This function will panic if the resources passed belong to a different context implementation.
    pub fn draw<'b>(
        &'b mut self,
        target: DrawTarget<'b, DynTexture>,
        f: impl FnOnce(&mut dyn FrameEncoder<'b, Shader = DynShader, Texture = DynTexture>),
    ) {
        let mut f = Some(f);
        self.draw_dyn(target, &mut |encoder| (f.take().unwrap())(encoder));
    }
}

impl<C: Context> DynContext for C {
    fn create_texture(&mut self, size: Size, format: TextureFormat) -> Result<DynTexture, TextureError> {
        Ok(DynTexture(Box::new(self.create_texture(size, format)?)))
    }

    fn upload_texture(&mut self, texture: &mut DynTexture, data: TextureData) -> Result<(), TextureError> {
        self.upload_texture(
            texture
                .0
                .downcast_mut::<C::Texture>()
                .expect("this resource does not belong to this context"),
            data,
        )
    }

    fn delete_texture(&mut self, texture: DynTexture) {
        self.delete_texture(
            *texture
                .0
                .downcast::<C::Texture>()
                .expect("this resource does not belong to this context"),
        );
    }

    fn create_shader(&mut self, shader: &ShaderData) -> Result<DynShader, ShaderError> {
        Ok(DynShader(Box::new(self.create_shader(shader)?)))
    }

    fn delete_shader(&mut self, shader: DynShader) {
        self.delete_shader(
            *shader
                .0
                .downcast::<C::Shader>()
                .expect("this resource does not belong to this context"),
        );
    }

    fn draw_dyn<'a>(
        &'a mut self,
        target: DrawTarget<'a, DynTexture>,
        f: &mut dyn FnMut(&mut dyn FrameEncoder<'a, Shader = DynShader, Texture = DynTexture>),
    ) {
        struct DynFrameEncoder<'a, E: ?Sized> {
            inner: &'a mut E,
        }

        impl<'a, E: FrameEncoder<'a> + ?Sized> FrameEncoder<'a> for DynFrameEncoder<'_, E> {
            type Shader = DynShader;
            type Texture = DynTexture;

            fn clear(&mut self, bounds: Bounds, color: Color) {
                self.inner.clear(bounds, color);
            }

            fn draw(&mut self, shader: &'a Self::Shader) {
                self.inner.draw(
                    shader
                        .0
                        .downcast_ref::<E::Shader>()
                        .expect("this resource does not belong to this context"),
                );
            }

            fn add_rect(&mut self, quad: Bounds) {
                self.inner.add_rect(quad);
            }

            fn add_data(&mut self, buffer: &[u8]) {
                self.inner.add_data(buffer);
            }

            fn add_texture(&mut self, texture: &'a Self::Texture) {
                self.inner.add_texture(
                    texture
                        .0
                        .downcast_ref::<E::Texture>()
                        .expect("this resource does not belong to this context"),
                );
            }
        }

        let target = match target {
            DrawTarget::Screen => DrawTarget::Screen,
            DrawTarget::Texture(tex) => DrawTarget::Texture(
                tex.0
                    .downcast_mut::<C::Texture>()
                    .expect("this resource does not belong to this context"),
            ),
        };

        self.draw(target, |encoder| {
            f(&mut DynFrameEncoder { inner: encoder });
        });
    }
}
