
use crate::platform::opengl;
use opengl::bindings::types::*;
use opengl::bindings::{
    ALPHA,
    UNSIGNED_BYTE,
};

use opengl::primitive::{
    Texture,
    TextureBuilder,
};

// 0: char
// 1,2: u,v
// 3,4: uv width,height
// 5-8: relative bb
// 9: relative advance width
type GlyphMetricsSource = (char, f32, f32, f32, f32, f32, f32, f32, f32, f32);

pub type FontSource = FontSourceDynamic<'static, 'static, 'static>;
pub type Font = FontDynamic<'static>;

pub struct FontSourceDynamic<'a, 'b, 'c> {
    pub atlas_image: &'b [u8],
    pub coords: &'a [GlyphMetricsSource],
    pub font_size: f32,
    pub image_width: GLsizei,
    pub image_height: GLsizei,
    pub padding_ratio: f32,
    pub kerning_pairs: &'c [(char, char, f32)],
}

impl<'a, 'b, 'c> FontSourceDynamic<'a, 'b, 'c> {
    pub fn load(&self) -> FontDynamic<'c> {
        let mut builder = TextureBuilder::create_custom(
            ALPHA,
            self.image_width,
            self.image_height,
        );
        builder.sub_image(
            0, 0,
            self.image_width, self.image_height,
            ALPHA, UNSIGNED_BYTE,
            self.atlas_image.as_ptr() as *const _,
        );
        let texture = builder.build();
        let font_size_ratio = self.font_size / (self.image_height as f32);
        FontDynamic {
            texture: texture,
            coords: self.coords.iter().copied().map(conv_glyph_metrics).collect(),
            font_size: self.font_size,
            padding_ratio: self.padding_ratio,
            kerning_pairs: self.kerning_pairs,
            space_ratio: 0.25,
        }
    }
}

struct GlyphMetrics {
    ch: char,
    uv_x: f32,
    uv_y: f32,
    uv_width: f32,
    uv_height: f32,
    bb_min_x: f32,
    bb_max_x: f32,
    bb_min_y: f32,
    bb_max_y: f32,
    advance_width: f32,
}

fn conv_glyph_metrics(source: GlyphMetricsSource) -> GlyphMetrics {
    GlyphMetrics {
        ch: source.0,
        uv_x: source.1,
        uv_y: source.2,
        uv_width: source.3,
        uv_height: source.4,
        bb_min_x: source.5,
        bb_max_x: source.6,
        bb_min_y: source.7,
        bb_max_y: source.8,
        advance_width: source.9,
    }
}

pub struct FontDynamic<'a> {
    texture: Texture,
    coords: Vec<GlyphMetrics>,
    font_size: f32,
    padding_ratio: f32,
    kerning_pairs: &'a [(char, char, f32)],
    space_ratio: f32,
}

impl<'a> FontDynamic<'a> {
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
    font: &'a FontDynamic<'a>,
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
            &ch,
            |coord| coord.ch
        ) {
            // uvs are provided by the font directly
            let metrics = &self.font.coords[index];
            self.uvs = Square {
                left: metrics.uv_x,
                right: metrics.uv_x + metrics.uv_width,
                top: metrics.uv_y,
                bottom: metrics.uv_y + metrics.uv_height,
            };
            // calculate the sizes to render at
            self.coords = Square {
                left: self.x_offset + metrics.bb_min_x * self.font_size,
                right: self.x_offset + metrics.bb_max_x * self.font_size,
                top: self.y_offset + metrics.bb_max_y * self.font_size,
                bottom: self.y_offset + metrics.bb_min_y * self.font_size,
            };
            // adjust x_offset forward (hopefully with kerning)
            self.x_offset += metrics.advance_width * self.font_size;
            if let Some(k_index) = next.and_then(|nch| {
                self.font.kerning_pairs.binary_search_by_key(
                    &(ch, nch),
                    |v| (v.0, v.1),
                ).ok()
            }) {
                self.x_offset += {
                    self.font.kerning_pairs[k_index].2 * self.font_size
                };
            }
        } else if ch == ' ' {
            // assume some things about spaces
            self.x_offset += self.font_size * self.font.space_ratio;
        }
    }
}
