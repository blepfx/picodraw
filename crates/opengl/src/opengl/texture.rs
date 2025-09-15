use glow::{
    CLAMP_TO_EDGE, COLOR_ATTACHMENT0, COLOR_BUFFER_BIT, FRAMEBUFFER, FRAMEBUFFER_COMPLETE, HasContext, LINEAR,
    PixelPackData, PixelUnpackData, R8, RED, RGB, RGB8, RGBA, RGBA8, SCISSOR_TEST, TEXTURE_2D, TEXTURE_MAG_FILTER,
    TEXTURE_MIN_FILTER, TEXTURE_WRAP_S, TEXTURE_WRAP_T, UNSIGNED_BYTE,
};
use picodraw_core::{TextureData, TextureFormat};
use std::cell::Cell;

pub struct GlTexture<T: HasContext> {
    pub(super) texture: T::Texture,
    framebuffer: Cell<Option<T::Framebuffer>>,
    width: u32,
    height: u32,
}

pub struct GlFramebufferBinding<'a, T: HasContext> {
    gl: &'a T,
}

impl<T: HasContext> GlTexture<T> {
    pub fn new(gl: &T, width: u32, height: u32, format: TextureFormat) -> Self {
        unsafe {
            let texture = gl.create_texture().unwrap();

            gl.bind_texture(TEXTURE_2D, Some(texture));

            gl.tex_parameter_i32(TEXTURE_2D, TEXTURE_MIN_FILTER, LINEAR as _);
            gl.tex_parameter_i32(TEXTURE_2D, TEXTURE_MAG_FILTER, LINEAR as _);
            gl.tex_parameter_i32(TEXTURE_2D, TEXTURE_WRAP_T, CLAMP_TO_EDGE as _);
            gl.tex_parameter_i32(TEXTURE_2D, TEXTURE_WRAP_S, CLAMP_TO_EDGE as _);

            gl.tex_image_2d(
                TEXTURE_2D,
                0,
                match format {
                    TextureFormat::R8 => R8 as _,
                    TextureFormat::RGB8 => RGB8 as _,
                    TextureFormat::RGBA8 => RGBA8 as _,
                },
                width as _,
                height as _,
                0,
                match format {
                    TextureFormat::R8 => RED as _,
                    TextureFormat::RGB8 => RGB as _,
                    TextureFormat::RGBA8 => RGBA as _,
                },
                UNSIGNED_BYTE,
                PixelUnpackData::Slice(None),
            );

            Self {
                texture,
                framebuffer: Cell::new(None),
                width,
                height,
            }
        }
    }

    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn texture(&self) -> T::Texture {
        self.texture
    }

    pub fn upload_subregion(&self, gl: &T, data: TextureData) -> bool {
        assert!(
            data.data.len()
                == data.bounds.width() as usize * data.bounds.height() as usize * data.format.bytes_per_pixel(),
            "invalid {:?} data length: {} != {} (width x height x {})",
            data.format,
            data.data.len(),
            data.bounds.width() as usize * data.bounds.height() as usize * data.format.bytes_per_pixel(),
            data.format.bytes_per_pixel()
        );

        assert!(
            data.bounds.right <= self.width && data.bounds.bottom <= self.height,
            "subregion is out of bounds"
        );

        if data.bounds.is_empty() {
            return false;
        }

        unsafe {
            gl.bind_texture(TEXTURE_2D, Some(self.texture));
            gl.tex_sub_image_2d(
                TEXTURE_2D,
                0,
                data.bounds.left as _,
                data.bounds.top as _,
                data.bounds.width() as _,
                data.bounds.height() as _,
                match data.format {
                    TextureFormat::R8 => RED as _,
                    TextureFormat::RGB8 => RGB as _,
                    TextureFormat::RGBA8 => RGBA as _,
                },
                UNSIGNED_BYTE,
                PixelUnpackData::Slice(Some(data.data)),
            );
        }

        true
    }

    pub fn bind_framebuffer<'a>(&'a self, gl: &'a T) -> GlFramebufferBinding<'a, T> {
        match self.framebuffer.get() {
            None => unsafe {
                let framebuffer = gl.create_framebuffer().unwrap();

                gl.bind_framebuffer(FRAMEBUFFER, Some(framebuffer));
                gl.framebuffer_texture_2d(FRAMEBUFFER, COLOR_ATTACHMENT0, TEXTURE_2D, Some(self.texture), 0);

                if gl.check_framebuffer_status(FRAMEBUFFER) != FRAMEBUFFER_COMPLETE {
                    panic!("framebuffer incomplete");
                }
            },

            Some(framebuffer) => unsafe {
                gl.bind_framebuffer(FRAMEBUFFER, Some(framebuffer));
            },
        }

        GlFramebufferBinding { gl }
    }

    pub fn delete(self, gl: &T) {
        unsafe {
            gl.delete_texture(self.texture);

            if let Some(framebuffer) = self.framebuffer.take() {
                gl.delete_framebuffer(framebuffer);
            }
        }
    }
}

impl<'a, T: HasContext> GlFramebufferBinding<'a, T> {
    pub fn default(gl: &'a T) -> Self {
        unsafe {
            gl.bind_framebuffer(FRAMEBUFFER, None);
        }

        GlFramebufferBinding { gl }
    }

    pub fn screenshot(&self, x: u32, y: u32, width: u32, height: u32) -> Vec<u8> {
        let mut data = vec![0; (width * height * 4) as usize];

        unsafe {
            self.gl.read_pixels(
                x as _,
                y as _,
                width as _,
                height as _,
                RGBA as _,
                UNSIGNED_BYTE,
                PixelPackData::Slice(Some(&mut data[..])),
            );
        }

        data
    }

    pub fn clear(&self, x: u32, y: u32, width: u32, height: u32) {
        unsafe {
            self.gl.enable(SCISSOR_TEST);
            self.gl.scissor(x as _, y as _, width as _, height as _);
            self.gl.clear_color(0.0, 0.0, 0.0, 0.0);
            self.gl.clear(COLOR_BUFFER_BIT);
            self.gl.disable(SCISSOR_TEST);
        }
    }
}
