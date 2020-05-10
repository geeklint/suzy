
use crate::math::consts::WHITE;
use crate::math::Color;
use crate::graphics;

use super::Mat4;
use super::OpenGlRenderPlatform as Gl;
use super::Texture;
use super::shader::Shader;
use super::stdshaders::Shaders;
use super::bindings::{
    TEXTURE0,
};

#[derive(Clone)]
pub struct DrawParams {
    pub(super) shaders: Shaders,
    pub(super) transform: Mat4,
    pub(super) tint_color: Color,
    pub(super) texture: Texture,
}

impl DrawParams {
    pub(crate) fn new(shaders: Shaders) -> Self {
        Self {
            shaders,
            transform: Default::default(),
            tint_color: WHITE,
            texture: Default::default(),
        }
    }

    pub fn transform(&mut self, mat: Mat4) {
        self.transform *= mat;
    }

    pub fn tint(&mut self, tint: Color) {
        self.tint_color.tint(tint);
    }

    pub fn use_texture(&mut self, texture: Texture) {
        self.texture = texture;
    }
}

impl graphics::DrawParams for DrawParams {
    fn apply_all(&self) {
        self.shaders.std.make_current();
        Shader::set_mat4(
            self.shaders.std_uniforms.common.transform, 
            self.transform.as_ref(),
        );
        Shader::set_vec4(
            self.shaders.std_uniforms.tint_color,
            self.tint_color.rgba(),
        );
        Gl::global(|gl| unsafe {
            gl.ActiveTexture(TEXTURE0);
            self.texture.bind(gl);
        });
        Shader::set_opaque(
            self.shaders.std_uniforms.tex_id,
            0,
        );
        Shader::set_vec4(
            self.shaders.std_uniforms.common.tex_transform,
            (
                self.texture.offset[0],
                self.texture.offset[1],
                self.texture.scale[0],
                self.texture.scale[1],
            ),
        );
    }

    fn apply_change(current: &Self, new: &Self) {
        if new.transform != current.transform {
            Shader::set_mat4(
                new.shaders.std_uniforms.common.transform, 
                new.transform.as_ref(),
            );
        }
        if new.tint_color != current.tint_color {
            Shader::set_vec4(
                new.shaders.std_uniforms.tint_color,
                new.tint_color.rgba(),
            );
        }
        if new.texture != current.texture {
            Gl::global(|gl| unsafe {
                gl.ActiveTexture(TEXTURE0);
                new.texture.bind(gl);
            });
            Shader::set_opaque(
                new.shaders.std_uniforms.tex_id,
                0,
            );
            Shader::set_vec4(
                new.shaders.std_uniforms.common.tex_transform,
                (
                    new.texture.offset[0],
                    new.texture.offset[1],
                    new.texture.scale[0],
                    new.texture.scale[1],
                ),
            );
        }
    }
}
