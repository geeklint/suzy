use gl::types::*;

use super::Texture;
use super::graphics::primitive::{Buffer, TextureBuilder};

type Coord = (u32, f32, f32, f32, f32);

pub struct FontSource<'a, 'b, 'c> {
    pub atlas_image: &'b [u8],
    pub coords: &'a [Coord],
    pub font_size: f32,
    pub image_width: GLsizei,
    pub image_height: GLsizei,
    pub padding_ratio: f32,
    pub kerning_pairs: &'c [(char, char, f32)],
}

impl<'a, 'b, 'c> FontSource<'a, 'b, 'c> {
    pub fn load(&self) -> Font<'a, 'c> {
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
        Font {
            texture,
            coords: self.coords,
            font_size: self.font_size / (self.image_height as f32),
            padding_ratio: self.padding_ratio,
            kerning_pairs: self.kerning_pairs,
            space_ratio: 0.25,
        }
    }
}

pub struct Font<'a, 'b> {
    texture: Texture,
    coords: &'a [Coord],
    font_size: f32,
    padding_ratio: f32,
    kerning_pairs: &'b [(char, char, f32)],
    space_ratio: f32,
}

pub struct Text {
    coords: Buffer<GLfloat>,
    uvs: Buffer<GLfloat>,
    texture: Texture,
}

impl Text {
    pub fn new() -> Self {
        Text {
            coords: Buffer::new(gl::ARRAY_BUFFER, true, 0),
            uvs: Buffer::new(gl::ARRAY_BUFFER, true, 0),
            texture: Default::default(),
        }
    }

    pub fn render(
        &mut self,
        text: &str,
        font: &Font<'_, '_>,
        font_size: f32,
    ) {
        self.texture = font.texture.clone();
        let mut coord_data = vec![];
        let mut uv_data = vec![];
        let mut iter = text.chars().peekable();
        let render_size = font_size * font.padding_ratio;
        let offset = (render_size - font_size) / -2.0;
        let mut x_offset = offset;
        let padding_amount = font.font_size / font.padding_ratio;
        while let Some(ch) = iter.next() {
            if let Ok(index) = font.coords.binary_search_by_key(
                &(ch as u32),
                |coord| coord.0
            ) {
                let (_, x, y, width, height) = font.coords[index];
                // top-left triangle
                uv_data.push(x);
                uv_data.push(y + height);
                uv_data.push(x + width);
                uv_data.push(y);
                uv_data.push(x);
                uv_data.push(y);
                // bottom right triangle
                uv_data.push(x);
                uv_data.push(y + height);
                uv_data.push(x + width);
                uv_data.push(y + height);
                uv_data.push(x + width);
                uv_data.push(y);
                let width_unpadded = width - padding_amount;
                let height_unpadded = height - padding_amount;
                let ch_width = font_size * width_unpadded / height_unpadded;
                let width = render_size * width / height;
                let height = render_size;
                let x = x_offset;
                if let Some(next) = iter.peek() {
                    if let Ok(index) = font.kerning_pairs.binary_search_by_key(
                        &(ch, *next),
                        |v| (v.0, v.1),
                    ) {
                        let kern_ratio = font.kerning_pairs[index].2;
                        x_offset += ch_width * kern_ratio;
                    } else {
                        x_offset += ch_width;
                    }
                }
                // top-left triangle
                coord_data.push(x);
                coord_data.push(offset);
                coord_data.push(x + width);
                coord_data.push(offset + height);
                coord_data.push(x);
                coord_data.push(offset + height);
                // bottom right triangle
                coord_data.push(x);
                coord_data.push(offset);
                coord_data.push(x + width);
                coord_data.push(offset);
                coord_data.push(x + width);
                coord_data.push(offset + height);
            } else if ch == ' ' {
                x_offset += font_size * font.space_ratio;
            }
        }
        self.coords.set_data(&coord_data);
        self.uvs.set_data(&uv_data);
    }
}

use crate::graphics::{DrawContext, Graphic};
use crate::math::consts::WHITE;

impl Graphic for Text {
    fn draw(&self, ctx: &mut DrawContext) {
        let mut params = ctx.clone_current();
        params.with_text(|layout| {
            layout.set_text_color(WHITE);
            layout.set_texture(&self.texture);
            unsafe {
                self.coords.bind();
                gl::VertexAttribPointer(
                    0,
                    2,
                    gl::FLOAT,
                    gl::FALSE,
                    0,
                    std::ptr::null(),
                );
                self.uvs.bind();
                gl::VertexAttribPointer(
                    1,
                    2,
                    gl::FLOAT,
                    gl::FALSE,
                    0,
                    std::ptr::null(),
                );
                gl::DrawArrays(
                    gl::TRIANGLES,
                    0,
                    self.coords.len() as GLsizei,
                );
            }
        });
    }
}
