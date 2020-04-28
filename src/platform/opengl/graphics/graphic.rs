use crate::graphics::{self, DrawContext};

use crate::platform::opengl;

use opengl::bindings::types::*;
use opengl::bindings::{
    ARRAY_BUFFER,
    ELEMENT_ARRAY_BUFFER,
    FALSE,
    FLOAT,
    TRIANGLES,
    UNSIGNED_BYTE,
};
use opengl::OpenGlRenderPlatform as Gl;
use opengl::primitive::{Buffer, Texture};


#[derive(Clone)]
pub struct Graphic {
    coords: Buffer<GLfloat>,
    uvs: Buffer<GLfloat>,
    tris: Buffer<u8>,
    texture: Texture,
    verts: usize,
    dynamic: (bool, bool, bool),
}

impl Graphic {
    pub fn new(
        verts: usize,
        tris: usize,
        dynamic: (bool, bool, bool),
    ) -> Self {
        Graphic {
            coords: Buffer::new(ARRAY_BUFFER, dynamic.0, verts * 2),
            uvs: Buffer::new(ARRAY_BUFFER, dynamic.1, verts * 2),
            tris: Buffer::new(
                ELEMENT_ARRAY_BUFFER,
                dynamic.2,
                tris * 3,
            ),
            texture: Default::default(),
            verts,
            dynamic,
        }
    }

    pub fn set_texture(&mut self, texture: Texture) {
        self.texture = texture;
    }

    pub fn set_coords(&mut self, coords: &[GLfloat]) {
        self.coords.set_data(coords);
    }

    pub fn set_uvs(&mut self, uvs: &[GLfloat]) {
        self.uvs.set_data(uvs);
    }

    pub fn set_tris(&mut self, tris: &[u8]) {
        self.tris.set_data(tris);
    }
}

impl graphics::Graphic<Gl> for Graphic {
    fn draw(&self, ctx: &mut DrawContext<Gl>) {
        let mut params = ctx.clone_current();
        params.use_texture(self.texture.clone());
        DrawContext::push(ctx, params);
        Gl::global(|gl| unsafe {
            self.coords.bind();
            gl.VertexAttribPointer(
                0,
                2,
                FLOAT,
                FALSE,
                0,
                std::ptr::null(),
            );
            self.uvs.bind();
            gl.VertexAttribPointer(
                1,
                2,
                FLOAT,
                FALSE,
                0,
                std::ptr::null(),
            );
            self.tris.bind();
            gl.DrawElements(
                TRIANGLES,
                self.tris.len() as i32,
                UNSIGNED_BYTE,
                std::ptr::null(),
            );
        });
        DrawContext::pop(ctx);
    }
}
