use std::rc::Rc;
use std::cell::RefCell;

use crate::platform::opengl::Shader;
use crate::graphics::image::{Texture};
use crate::math::consts::WHITE;
use crate::math::Color;

#[derive(Clone)]
pub struct DrawParams {
    shader: Rc<RefCell<Shader>>,
    tint_color: Color,
}

impl DrawParams {
    pub(crate) fn new(shader: Rc<RefCell<Shader>>) -> Self {
        shader.borrow_mut().set_uniform_vec4(
            "TINT_COLOR",
            WHITE.rgba(),
        );
        Self {
            shader,
            tint_color: WHITE,
        }
    }

    pub fn tint(&mut self, tint: Color) {
        self.tint_color.tint(tint);
        self.shader.borrow_mut().set_uniform_vec4(
            "TINT_COLOR",
            self.tint_color.rgba(),
        );
    }

    pub fn use_texture(&mut self, texture: &Texture) {
        let mut shader = self.shader.borrow_mut();
        shader.set_uniform_opaque("TEX_ID", texture.id);
        shader.set_uniform_vec2(
            "TEX_OFFSET",
            (texture.offset[0], texture.offset[1]),
        );
        shader.set_uniform_vec2(
            "TEX_SCALE",
            (texture.scale[0], texture.scale[1]),
        );
    }
}
