
use gl::types::*;

use crate::platform::opengl::graphics::primitive::{
    Texture,
    Buffer,
};

mod font;

pub use font::{
    Font,
    FontDynamic,
    FontSource,
    FontSourceDynamic,
};

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
        font: &FontDynamic<'_, '_>,
        font_size: f32,
    ) {
        self.texture = font.texture().clone();
        let mut coord_data = vec![];
        let mut uv_data = vec![];
        let mut calc = font.get_char_calc(font_size);
        let mut iter = text.chars().peekable();
        while let Some(ch) = iter.next() {
            calc.push(ch, iter.peek().copied());
            let coord_sq = calc.current_coords();
            let uv_sq = calc.current_uvs();
            // top-left triangle
            coord_data.push(coord_sq.left);
            coord_data.push(coord_sq.bottom);
            coord_data.push(coord_sq.right);
            coord_data.push(coord_sq.top);
            coord_data.push(coord_sq.left);
            coord_data.push(coord_sq.top);
            uv_data.push(uv_sq.left);
            uv_data.push(uv_sq.bottom);
            uv_data.push(uv_sq.right);
            uv_data.push(uv_sq.top);
            uv_data.push(uv_sq.left);
            uv_data.push(uv_sq.top);
            // bottom right triangle
            coord_data.push(coord_sq.left);
            coord_data.push(coord_sq.bottom);
            coord_data.push(coord_sq.right);
            coord_data.push(coord_sq.bottom);
            coord_data.push(coord_sq.right);
            coord_data.push(coord_sq.top);
            uv_data.push(uv_sq.left);
            uv_data.push(uv_sq.bottom);
            uv_data.push(uv_sq.right);
            uv_data.push(uv_sq.bottom);
            uv_data.push(uv_sq.right);
            uv_data.push(uv_sq.top);
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
