
use crate::math::Color;
use crate::graphics;

use crate::platform::opengl;
use opengl::Mat4;
use opengl::OpenGlRenderPlatform as Gl;
use opengl::Texture;
use opengl::shader::Shader;
use opengl::stdshaders::Shaders;
use opengl::bindings::{
    TEXTURE0,
};

#[derive(Clone)]
pub struct DrawParams {
    pub(super) shaders: Shaders,
    pub(super) transform: Mat4,
    pub(super) tint_color: Color,
    pub(super) text_color: Color,
    pub(super) outline_color: Color,
    pub(super) distance_edges: (f32, f32, f32, f32),
    pub(super) texture: Texture,
    pub(super) tex_chan_mask: (f32, f32, f32, f32),
}

impl DrawParams {
    pub fn transform(&mut self, mat: Mat4) {
        self.transform *= mat;
    }

    pub fn tint(&mut self, tint: Color) {
        self.tint_color.tint(tint);
    }

    pub fn use_text_color(&mut self, color: Color) {
        self.text_color = color;
    }

    pub fn use_outline_color(&mut self, color: Color) {
        self.outline_color = color;
    }

    pub fn use_tex_chan_mask(&mut self, mask: (u8, u8, u8, u8)) {
        self.tex_chan_mask = (
            mask.0 as f32 / 255.0,
            mask.1 as f32 / 255.0,
            mask.2 as f32 / 255.0,
            mask.3 as f32 / 255.0,
        );
    }

    pub fn body_edge(&mut self, edge: f32, smoothing: f32) {
        let smoothing = smoothing / 2.0;
        self.distance_edges.0 = edge - smoothing;
        self.distance_edges.1 = edge + smoothing;
    }

    pub fn outline_edge(&mut self, edge: f32, smoothing: f32) {
        let smoothing = smoothing / 2.0;
        self.distance_edges.2 = edge - smoothing;
        self.distance_edges.3 = edge + smoothing;
    }

    pub fn use_texture(&mut self, texture: Texture) {
        self.texture = texture;
    }
}

impl graphics::DrawParams for DrawParams {
    fn apply_all(&self) {
        self.shaders.sdf.make_current();
        Shader::set_mat4(
            self.shaders.sdf_uniforms.common.transform, 
            self.transform.as_ref(),
        );
        let mut text_color = self.text_color;
        let mut outline_color = self.outline_color;
        text_color.tint(self.tint_color);
        outline_color.tint(self.tint_color);
        Shader::set_vec4(
            self.shaders.sdf_uniforms.text_color,
            text_color.rgba(),
        );
        Shader::set_vec4(
            self.shaders.sdf_uniforms.outline_color,
            outline_color.rgba(),
        );
        Shader::set_vec4(
            self.shaders.sdf_uniforms.distance_edges,
            self.distance_edges,
        );
        Gl::global(|gl| unsafe {
            gl.ActiveTexture(TEXTURE0);
            self.texture.bind(gl);
        });
        Shader::set_opaque(
            self.shaders.sdf_uniforms.tex_id,
            0,
        );
        Shader::set_vec4(
            self.shaders.sdf_uniforms.common.tex_transform,
            (
                self.texture.offset[0],
                self.texture.offset[1],
                self.texture.scale[0],
                self.texture.scale[1],
            ),
        );
        Shader::set_vec4(
            self.shaders.sdf_uniforms.tex_chan_mask,
            self.tex_chan_mask,
        );
    }

    fn apply_change(current: &Self, new: &Self) {
        if new.transform != current.transform {
            Shader::set_mat4(
                new.shaders.sdf_uniforms.common.transform, 
                new.transform.as_ref(),
            );
        }
        if new.tint_color != current.tint_color
            || new.text_color != current.text_color
        {
            let mut text_color = new.text_color;
            text_color.tint(new.tint_color);
            Shader::set_vec4(
                new.shaders.sdf_uniforms.text_color,
                text_color.rgba(),
            );
        }
        if new.tint_color != current.tint_color
            || new.outline_color != current.outline_color
        {
            let mut outline_color = new.outline_color;
            outline_color.tint(new.tint_color);
            Shader::set_vec4(
                new.shaders.sdf_uniforms.outline_color,
                outline_color.rgba(),
            );
        }
        if new.distance_edges != current.distance_edges {
            Shader::set_vec4(
                new.shaders.sdf_uniforms.distance_edges,
                new.distance_edges,
            );
        }
        if new.texture != current.texture {
            Gl::global(|gl| unsafe {
                gl.ActiveTexture(TEXTURE0);
                new.texture.bind(gl);
            });
            Shader::set_opaque(
                new.shaders.sdf_uniforms.tex_id,
                0,
            );
            Shader::set_vec4(
                new.shaders.sdf_uniforms.common.tex_transform,
                (
                    new.texture.offset[0],
                    new.texture.offset[1],
                    new.texture.scale[0],
                    new.texture.scale[1],
                ),
            );
        }
        if new.tex_chan_mask != current.tex_chan_mask {
            Shader::set_vec4(
                new.shaders.sdf_uniforms.tex_chan_mask,
                new.tex_chan_mask,
            );
        }
    }
}
