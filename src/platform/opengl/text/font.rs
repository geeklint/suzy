
use gl::types::*;

use crate::platform::opengl::graphics::primitive::{
    Texture,
    TextureBuilder,
};

type Coord = (u32, f32, f32, f32);

pub type FontSource = FontSourceDynamic<'static, 'static, 'static>;
pub type Font = FontDynamic<'static, 'static>;

pub struct FontSourceDynamic<'a, 'b, 'c> {
    pub atlas_image: &'b [u8],
    pub coords: &'a [Coord],
    pub font_size: f32,
    pub char_height: f32,
    pub image_width: GLsizei,
    pub image_height: GLsizei,
    pub padding_ratio: f32,
    pub kerning_pairs: &'c [(char, char, f32)],
}

impl<'a, 'b, 'c> FontSourceDynamic<'a, 'b, 'c> {
    pub fn load(&self) -> FontDynamic<'a, 'c> {
        let mut builder = TextureBuilder::create_custom(
            gl::R8,
            self.image_width,
            self.image_height,
        );
        builder.sub_image(
            0, 0,
            self.image_width, self.image_height,
            gl::RED, gl::UNSIGNED_BYTE,
            self.atlas_image.as_ptr() as *const _,
        );
        let texture = builder.build();
        let font_size_ratio = self.font_size / (self.image_height as f32);
        FontDynamic {
            texture,
            coords: self.coords,
            font_size: self.font_size,
            char_height: self.char_height,
            padding_ratio: self.padding_ratio,
            kerning_pairs: self.kerning_pairs,
            space_ratio: 0.25,
            padding_amount: (self.char_height - font_size_ratio),
        }
    }
}

pub struct FontDynamic<'a, 'b> {
    texture: Texture,
    coords: &'a [Coord],
    font_size: f32,
    char_height: f32,
    padding_ratio: f32,
    kerning_pairs: &'b [(char, char, f32)],
    space_ratio: f32,
    padding_amount: f32,
}

impl<'a, 'b> FontDynamic<'a, 'b> {
    pub fn texture(&self) -> &Texture { &self.texture }

    pub(super) fn get_char_calc(&self, font_size: f32) -> FontCharCalc {
        let render_size = font_size * self.padding_ratio;
        let offset = (render_size - font_size) / -2.0;
        FontCharCalc {
            font: self,
            font_size,
            x_offset: offset,
            y_offset: offset,
            coords: Default::default(),
            uvs: Default::default(),
        }
    }

    pub fn measure(&self, text: &str, font_size: f32) -> f32 {
        let mut calc = self.get_char_calc(font_size);
        let mut iter = text.chars().peekable();
        while let Some(ch) = iter.next() {
            calc.push(ch, iter.peek().copied());
        }
        calc.x_offset
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Square {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

pub(super) struct FontCharCalc<'a> {
    font: &'a FontDynamic<'a, 'a>,
    font_size: f32,
    x_offset: f32,
    y_offset: f32,
    coords: Square,
    uvs: Square,
}

impl<'a> FontCharCalc<'a> {
    pub fn current_coords(&self) -> &Square { &self.coords }
    pub fn current_uvs(&self) -> &Square { &self.uvs }

    pub fn push(&mut self, ch: char, next: Option<char>) {
        if let Ok(index) = self.font.coords.binary_search_by_key(
            &(ch as u32),
            |coord| coord.0
        ) {
            // uvs are provided by the font directly
            let (_, x, y, width) = self.font.coords[index];
            let height = self.font.char_height;
            self.uvs = Square {
                left: x,
                right: x + width,
                top: y,
                bottom: y + height,
            };
            // calculate the sizes to render at
            let render_size = self.font_size * self.font.padding_ratio;
            let render_width = render_size * width / height;
            let render_height = render_size;
            self.coords = Square {
                left: self.x_offset,
                right: self.x_offset + render_width,
                top: self.y_offset + render_height,
                bottom: self.y_offset,
            };
            // adjust x_offset forward (hopefully with kerning)
            let width_unpadded = width - self.font.padding_amount;
            let height_unpadded = height - self.font.padding_amount;
            let ch_width = self.font_size * width_unpadded / height_unpadded;
            if let Some(k_index) = next.and_then(|nch| {
                self.font.kerning_pairs.binary_search_by_key(
                    &(ch, nch),
                    |v| (v.0, v.1),
                ).ok()
            }) {
                let kern_ratio = self.font.kerning_pairs[k_index].2;
                self.x_offset += ch_width * kern_ratio;
            } else {
                self.x_offset += ch_width;
            }
        } else if ch == ' ' {
            // assume some things about spaces
            self.x_offset += self.font_size * self.font.space_ratio;
        }
    }
}
