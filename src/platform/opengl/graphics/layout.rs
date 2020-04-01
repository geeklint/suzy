use std::rc::Rc;

use gl::types::*;

use super::primitive::{VertexArrayObject, Texture};
use super::super::Shader;
use super::super::shader::UniformLoc;
use crate::math::Color;

#[derive(Clone)]
pub struct Layout {
    vao: Rc<VertexArrayObject>,
    shader: Shader,
}

impl Layout {
    pub fn create(shader: Shader, enabled_attribs: &[u32]) -> Self {
        let vao = Rc::new(VertexArrayObject::new());
        unsafe {
            gl::BindVertexArray(vao.id);
        }
        for index in enabled_attribs {
            unsafe {
                gl::EnableVertexAttribArray(*index);
            }
        }
        Layout { vao, shader }
    }

    pub fn make_current(&self) {
        unsafe {
            gl::BindVertexArray(self.vao.id);
        }
        self.shader.make_current();
    }

    pub fn uniform(&mut self, name: &str) -> UniformLoc {
        self.shader.uniform(name)
    }

    pub fn set_opaque(&mut self, loc: UniformLoc, value: GLuint) {
        self.shader.set_opaque(loc, value);
    }

    pub fn set_float(&mut self, loc: UniformLoc, value: GLfloat) {
        self.shader.set_float(loc, value);
    }

    pub fn set_vec2(&mut self, loc: UniformLoc, value: (GLfloat, GLfloat)) {
        self.shader.set_vec2(loc, value);
    }

    pub fn set_vec4(
        &mut self,
        loc: UniformLoc,
        value: (GLfloat, GLfloat, GLfloat, GLfloat),
    ) {
        self.shader.set_vec4(loc, value);
    }
}

#[derive(Copy, Clone, Debug)]
struct StandardUniforms {
    screen_size: UniformLoc,
    tex_offset: UniformLoc,
    tex_scale: UniformLoc,
    tint_color: UniformLoc,
    tex_id: UniformLoc,
}

#[derive(Clone)]
pub struct StandardLayout {
    layout: Layout,
    uniforms: StandardUniforms,
    text_layout: TextLayout,
    screen_size: (f32, f32),
}

impl StandardLayout {
    pub fn new() -> Self {
        let mut layout = Layout::create(Shader::standard(), &[0, 1]);
        let uniforms = StandardUniforms {
            screen_size: layout.uniform("SCREEN_SIZE"),
            tex_offset: layout.uniform("TEX_OFFSET"),
            tex_scale: layout.uniform("TEX_SCALE"),
            tint_color: layout.uniform("TINT_COLOR"),
            tex_id: layout.uniform("TEX_ID"),
        };
        Self {
            layout,
            uniforms,
            text_layout: TextLayout::new(),
            screen_size: (0.0, 0.0),
        }
    }

    pub fn make_current(&mut self) {
        self.layout.make_current();
        self.layout.set_opaque(self.uniforms.tex_id, 0);
    }

    pub fn set_screen_size(&mut self, value: (f32, f32)) {
        self.layout.set_vec2(self.uniforms.screen_size, value);
        self.screen_size = value;
    }

    pub fn set_texture(&mut self, texture: &Texture) {
        self.layout.set_vec2(
            self.uniforms.tex_offset,
            (texture.offset[0], texture.offset[1]),
        );
        self.layout.set_vec2(
            self.uniforms.tex_scale,
            (texture.scale[0], texture.scale[1]),
        );
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, texture.id);
        }
    }

    pub fn set_tint_color(&mut self, value: Color) {
        let value = value.rgba();
        self.layout.set_vec4(self.uniforms.tint_color, value);
    }

    pub(crate) fn with_text<F, R>(&mut self, func: F) -> R
        where F: FnOnce(&mut TextLayout) -> R
    {
        self.text_layout.make_current();
        self.text_layout.set_screen_size(self.screen_size);
        let res = (func)(&mut self.text_layout);
        self.make_current();
        res
    }
}

#[derive(Copy, Clone, Debug)]
struct TextUniforms {
    screen_size: UniformLoc,
    tex_id: UniformLoc,
    text_color: UniformLoc,
}

#[derive(Clone)]
pub(crate) struct TextLayout {
    layout: Layout,
    uniforms: TextUniforms,
}

impl TextLayout {
    pub fn new() -> Self {
        let mut layout = Layout::create(Shader::text(), &[0, 1]);
        let uniforms = TextUniforms {
            screen_size: layout.uniform("SCREEN_SIZE"),
            tex_id: layout.uniform("TEX_ID"),
            text_color: layout.uniform("TEXT_COLOR"),
        };
        Self {
            layout,
            uniforms,
        }
    }

    pub fn make_current(&mut self) {
        self.layout.make_current();
        self.layout.set_opaque(self.uniforms.tex_id, 0);
    }

    pub fn set_screen_size(&mut self, value: (f32, f32)) {
        self.layout.set_vec2(self.uniforms.screen_size, value);
    }

    pub fn set_texture(&mut self, texture: &Texture) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, texture.id);
        }
    }

    pub fn set_text_color(&mut self, value: Color) {
        let value = value.rgba();
        self.layout.set_vec4(self.uniforms.text_color, value);
    }
}
