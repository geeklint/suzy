use std::rc::Rc;
use std::cell::RefCell;

use crate::platform::opengl::Shader;
use crate::math::consts::WHITE;
use crate::math::Color;

use super::Texture;
use super::graphics::layout::StandardLayout;

#[derive(Clone)]
pub struct DrawParams {
    layout: StandardLayout,
    tint_color: Color,
    texture: Texture,
}

impl DrawParams {
    pub(crate) fn new(layout: StandardLayout) -> Self {
        Self {
            layout,
            tint_color: WHITE,
            texture: Default::default(),
        }
    }

    pub fn apply_change(current: &Self, new: &mut Self) {
        if new.tint_color != current.tint_color {
            new.layout.set_tint_color(new.tint_color);
        }
        if new.texture != current.texture {
            new.layout.set_texture(&new.texture);
        }
    }

    pub fn tint(&mut self, tint: Color) {
        self.tint_color.tint(tint);
    }

    pub fn use_texture(&mut self, texture: Texture) {
        self.texture = texture;
    }
}
