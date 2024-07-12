use crunch::{Item, Rotation};
use image::{DynamicImage, GenericImageView, RgbaImage};
use rustc_hash::FxHashMap;
use std::mem::swap;

const PADDING: u32 = 1;

pub struct PackedTexture {
    pub rotated: bool,
    pub x: u32,
    pub y: u32,
    pub data: DynamicImage,
}

pub struct TextureAtlas {
    pub size: u32,
    pub textures: FxHashMap<(u32, String), PackedTexture>,
}

#[derive(Clone, Copy)]
pub struct ShaderTextures<'a> {
    pub index: u32,
    pub atlas: &'a TextureAtlas,
}

impl TextureAtlas {
    pub fn pack<'a>(
        data: impl IntoIterator<Item = (u32, &'a str, DynamicImage)>,
        max_size: u32,
    ) -> Self {
        let packed = crunch::pack_into_po2(
            max_size as usize,
            data.into_iter().map(|item| {
                let (width, height) = (item.2.width() + 2 * PADDING, item.2.height() + 2 * PADDING);
                Item::new(item, width as usize, height as usize, Rotation::Allowed)
            }),
        )
        .expect("failed to pack the textures");

        Self {
            size: packed.w as u32,
            textures: FxHashMap::from_iter(packed.items.into_iter().map(|packed| {
                (
                    (packed.data.0, packed.data.1.to_owned()),
                    PackedTexture {
                        rotated: packed.rect.w != (packed.data.2.width() + 2 * PADDING) as usize,
                        x: packed.rect.x as u32 + PADDING,
                        y: packed.rect.y as u32 + PADDING,
                        data: packed.data.2,
                    },
                )
            })),
        }
    }

    pub fn shader(&self, index: u32) -> ShaderTextures {
        ShaderTextures { index, atlas: self }
    }

    pub fn create_image_rgba(&self) -> RgbaImage {
        let mut image = RgbaImage::new(self.size, self.size);

        for (_, tex) in &self.textures {
            let x = tex.x - PADDING;
            let y = tex.y - PADDING;
            let mut w = tex.data.width() + 2 * PADDING;
            let mut h = tex.data.height() + 2 * PADDING;

            if tex.rotated {
                swap(&mut w, &mut h);
            }

            if tex.data.width() * tex.data.height() == 0 {
                continue;
            }

            for j in 0..h {
                for i in 0..w {
                    let (src_x, src_y) = if tex.rotated { (j, i) } else { (i, j) };
                    let src_x = src_x.saturating_sub(PADDING).min(tex.data.width() - 1);
                    let src_y = src_y.saturating_sub(PADDING).min(tex.data.height() - 1);

                    image.put_pixel(x + i, y + j, tex.data.get_pixel(src_x, src_y));
                }
            }
        }

        image
    }
}

impl<'a> ShaderTextures<'a> {
    pub fn get(&self, id: &str) -> &PackedTexture {
        self.atlas
            .textures
            .get(&(self.index, id.to_owned()))
            .unwrap_or_else(|| panic!("unknown texture index: {}", id))
    }
}
