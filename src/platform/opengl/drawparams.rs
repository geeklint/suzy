/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::math::consts::WHITE;
use crate::math::Color;
use crate::graphics;

use super::Mat4;
use super::OpenGlContext;
use super::texture::Texture;
use super::shader::Shader;
use super::context::bindings::types::GLuint;
use super::context::bindings::TEXTURE0;

#[derive(Clone)]
enum ShaderExclusive {
    Standard,
    Sdf {
        text_color: Color,
        outline_color: Color,
        distance_edges: (f32, f32, f32, f32),
        tex_chan_mask: (f32, f32, f32, f32),
    }
}

impl ShaderExclusive {
    fn make_standard(&mut self) {
        *self = Self::Standard;
    }

    fn make_sdf(&mut self) {
        if let Self::Standard = self {
            *self = Self::Sdf {
                text_color: WHITE,
                outline_color: Color::create_rgba8(0xff, 0xff, 0xff, 0),
                distance_edges: (0.49, 0.51, 0.0, 0.0),
                tex_chan_mask: (0.0, 0.0, 0.0, 0.0),
            };
        }
    }

    fn apply_all(
        &self,
        tint_color: Color,
        ctx: &OpenGlContext,
        prev_attrs: Option<GLuint>,
    ) {
        match self {
            Self::Standard => {
                ctx.shaders.std.make_current(&ctx.bindings, prev_attrs);
                Shader::set_opaque(
                    &ctx.bindings,
                    ctx.shaders.std_uniforms.tex_id,
                    0,
                );
                Shader::set_vec4(
                    &ctx.bindings,
                    ctx.shaders.std_uniforms.tint_color,
                    tint_color.rgba(),
                );
            },
            Self::Sdf {
                mut text_color,
                mut outline_color,
                distance_edges,
                tex_chan_mask,
            } => {
                ctx.shaders.sdf.make_current(&ctx.bindings, prev_attrs);
                Shader::set_opaque(
                    &ctx.bindings,
                    ctx.shaders.sdf_uniforms.tex_id,
                    0,
                );
                text_color.tint(tint_color);
                outline_color.tint(tint_color);
                Shader::set_vec4(
                    &ctx.bindings,
                    ctx.shaders.sdf_uniforms.text_color,
                    text_color.rgba(),
                );
                Shader::set_vec4(
                    &ctx.bindings,
                    ctx.shaders.sdf_uniforms.outline_color,
                    outline_color.rgba(),
                );
                Shader::set_vec4(
                    &ctx.bindings,
                    ctx.shaders.sdf_uniforms.distance_edges,
                    *distance_edges,
                );
                Shader::set_vec4(
                    &ctx.bindings,
                    ctx.shaders.sdf_uniforms.tex_chan_mask,
                    *tex_chan_mask,
                );
            },
        }
    }

    fn apply_change(
        current: &Self,
        new: &Self,
        current_tint_color: Color,
        new_tint_color: Color,
        ctx: &OpenGlContext,
    ) -> bool {
        match (current, new) {
            (Self::Standard, Self::Sdf { .. }) => {
                let prev_attrs = Some(ctx.shaders.std.attrs());
                new.apply_all(new_tint_color, ctx, prev_attrs);
                true
            },
            (Self::Sdf { .. }, Self::Standard) => {
                let prev_attrs = Some(ctx.shaders.sdf.attrs());
                new.apply_all(new_tint_color, ctx, prev_attrs);
                true
            },
            (Self::Standard, Self::Standard) => {
                if new_tint_color != current_tint_color {
                    Shader::set_vec4(
                        &ctx.bindings,
                        ctx.shaders.std_uniforms.tint_color,
                        new_tint_color.rgba(),
                    );
                }
                false
            },
            (
                Self::Sdf {
                    text_color: current_text_color,
                    outline_color: current_outline_color,
                    distance_edges: current_distance_edges,
                    tex_chan_mask: current_tex_chan_mask,
                },
                Self::Sdf {
                    text_color: mut new_text_color,
                    outline_color: mut new_outline_color,
                    distance_edges: new_distance_edges,
                    tex_chan_mask: new_tex_chan_mask,
                },
            ) => {
                if new_tint_color != current_tint_color
                    || new_text_color != *current_text_color
                {
                    new_text_color.tint(new_tint_color);
                    Shader::set_vec4(
                        &ctx.bindings,
                        ctx.shaders.sdf_uniforms.text_color,
                        new_text_color.rgba(),
                    );
                }
                if new_tint_color != current_tint_color
                    || new_outline_color != *current_outline_color
                {
                    new_outline_color.tint(new_tint_color);
                    Shader::set_vec4(
                        &ctx.bindings,
                        ctx.shaders.sdf_uniforms.outline_color,
                        new_outline_color.rgba(),
                    );
                }
                if new_distance_edges != current_distance_edges {
                    Shader::set_vec4(
                        &ctx.bindings,
                        ctx.shaders.sdf_uniforms.distance_edges,
                        *new_distance_edges,
                    );
                }
                if new_tex_chan_mask != current_tex_chan_mask {
                    Shader::set_vec4(
                        &ctx.bindings,
                        ctx.shaders.sdf_uniforms.tex_chan_mask,
                        *new_tex_chan_mask,
                    );
                }
                false
            },
        }
    }
}

#[derive(Clone)]
pub struct DrawParams {
    transform: Mat4,
    tint_color: Color,
    texture: Texture,
    shader_exclusive: ShaderExclusive,
}

impl DrawParams {
    pub(crate) fn new() -> Self {
        Self {
            transform: Mat4::default(),
            tint_color: WHITE,
            texture: Texture::default(),
            shader_exclusive: ShaderExclusive::Standard,
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

    pub fn standard_mode(&mut self) {
        self.shader_exclusive.make_standard();
    }

    pub fn sdf_mode(&mut self) {
        self.shader_exclusive.make_sdf();
    }

    pub fn text_color(&mut self, color: Color) {
        use ShaderExclusive::*;
        if let Sdf { ref mut text_color, .. } = self.shader_exclusive {
            *text_color = color;
        } else {
            debug_assert!(
                false,
                "DrawParams::text_color should only be used with Sdf shader",
            );
        }
    }

    pub fn outline_color(&mut self, color: Color) {
        use ShaderExclusive::*;
        if let Sdf { ref mut outline_color, .. } = self.shader_exclusive {
            *outline_color = color;
        } else {
            debug_assert!(
                false,
                "DrawParams::outline_color should only be used with Sdf shader",
            );
        }
    }

    pub fn tex_chan_mask(&mut self, mask: (u8, u8, u8, u8)) {
        use ShaderExclusive::*;
        if let Sdf { ref mut tex_chan_mask, .. } = self.shader_exclusive {
            *tex_chan_mask = (
                mask.0 as f32 / 255.0,
                mask.1 as f32 / 255.0,
                mask.2 as f32 / 255.0,
                mask.3 as f32 / 255.0,
            );
        } else {
            debug_assert!(
                false,
                "DrawParams::tex_chan_mask should only be used with Sdf shader",
            );
        }
    }

    pub fn body_edge(&mut self, edge: f32, smoothing: f32) {
        use ShaderExclusive::*;
        if let Sdf { ref mut distance_edges, .. } = self.shader_exclusive {
            let smoothing = smoothing / 2.0;
            distance_edges.0 = edge - smoothing;
            distance_edges.1 = edge + smoothing;
        } else {
            debug_assert!(
                false,
                "DrawParams::body_edge should only be used with Sdf shader",
            );
        }
    }

    pub fn outline_edge(&mut self, edge: f32, smoothing: f32) {
        use ShaderExclusive::*;
        if let Sdf { ref mut distance_edges, .. } = self.shader_exclusive {
            let smoothing = smoothing / 2.0;
            distance_edges.2 = edge - smoothing;
            distance_edges.3 = edge + smoothing;
        } else {
            debug_assert!(
                false,
                "DrawParams::outline_edge should only be used with Sdf shader",
            );
        }
    }
}

impl graphics::DrawParams<OpenGlContext> for DrawParams {
    fn apply_all(&mut self, ctx: &mut OpenGlContext) {
        self.shader_exclusive.apply_all(self.tint_color, &ctx, None);
        let uniforms = match &self.shader_exclusive {
            ShaderExclusive::Standard => ctx.shaders.std_uniforms.common,
            ShaderExclusive::Sdf { .. } => ctx.shaders.sdf_uniforms.common,
        };
        Shader::set_mat4(
            &ctx.bindings,
            uniforms.transform, 
            self.transform.as_ref(),
        );
        unsafe { ctx.bindings.ActiveTexture(TEXTURE0) };
        let tex_trans = self.texture.bind(ctx);
        Shader::set_vec4(
            &ctx.bindings,
            uniforms.tex_transform,
            tex_trans,
        );
    }

    fn apply_change(current: &Self, new: &mut Self, ctx: &mut OpenGlContext) {
        let shader_changed = ShaderExclusive::apply_change(
            &current.shader_exclusive,
            &new.shader_exclusive,
            current.tint_color,
            new.tint_color,
            &ctx,
        );
        if shader_changed || new.transform != current.transform {
            Shader::set_mat4(
                &ctx.bindings,
                ctx.shaders.std_uniforms.common.transform, 
                new.transform.as_ref(),
            );
        }
        if shader_changed || new.texture != current.texture {
            unsafe { ctx.bindings.ActiveTexture(TEXTURE0) };
            let tex_trans = new.texture.bind(ctx);
            Shader::set_vec4(
                &ctx.bindings,
                ctx.shaders.std_uniforms.common.tex_transform,
                tex_trans,
            );
        }
    }
}
