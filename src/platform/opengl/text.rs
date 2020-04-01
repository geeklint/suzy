use gl::types::*;

use super::Texture;
use super::graphics::primitive::{Buffer, TextureBuilder};

type Coord = (u32, f32, f32, f32, f32);

pub struct FontSource<'a, 'b> {
    pub atlas_image: &'b [u8],
    pub coords: &'a [Coord],
    pub font_size: f32,
    pub image_width: GLsizei,
    pub image_height: GLsizei,
}

impl<'a, 'b> FontSource<'a, 'b> {
    pub fn load(&self) -> Font<'a> {
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
            font_size: self.font_size,
        }
    }
}

pub struct Font<'a> {
    texture: Texture,
    coords: &'a [Coord],
    font_size: f32,
}

impl Font<'_> {
    
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
        font: &Font<'_>,
        font_size: f32,
    ) {
        self.texture = font.texture.clone();
        let mut coord_data = vec![];
        let mut uv_data = vec![];
        let mut x_offset = 0.0;
        for ch in text.chars() {
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
                let width = width / height * font_size;
                let height = font_size;
                let x = x_offset;
                x_offset += width;
                // top-left triangle
                coord_data.push(x);
                coord_data.push(0.0);
                coord_data.push(x + width);
                coord_data.push(height);
                coord_data.push(x);
                coord_data.push(height);
                // bottom right triangle
                coord_data.push(x);
                coord_data.push(0.0);
                coord_data.push(x + width);
                coord_data.push(0.0);
                coord_data.push(x + width);
                coord_data.push(height);
            }
        }
        println!("{:?}", &coord_data);
        println!("{:?}", &uv_data);
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
