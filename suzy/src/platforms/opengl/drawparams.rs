/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(missing_docs)]

use crate::graphics;
use crate::graphics::Color;

use super::context::bindings::types::GLuint;
use super::context::bindings::{
    CONSTANT_COLOR, FUNC_ADD, FUNC_REVERSE_SUBTRACT, ONE, ONE_MINUS_SRC_ALPHA,
    SRC_ALPHA, TEXTURE0, TEXTURE1, TEXTURE_2D,
};
use super::shader::Shader;
use super::texture::Texture;
use super::Mat4;
use super::OpenGlContext;

pub(super) const MASK_LEVELS: u8 = 4;

#[derive(Clone)]
enum ShaderExclusive {
    Standard,
    Sdf {
        text_color: Color,
        outline_color: Color,
        distance_edges: (f32, f32, f32, f32),
        tex_chan_mask: (f32, f32, f32, f32),
    },
}

impl ShaderExclusive {
    fn make_standard(&mut self) {
        *self = Self::Standard;
    }

    fn make_sdf(&mut self) {
        if let Self::Standard = self {
            *self = Self::Sdf {
                text_color: Color::WHITE,
                outline_color: Color::create_rgba8(0xff, 0xff, 0xff, 0),
                distance_edges: (0.465, 0.535, 0.0, 0.0),
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
            }
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
            }
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
            }
            (Self::Sdf { .. }, Self::Standard) => {
                let prev_attrs = Some(ctx.shaders.sdf.attrs());
                new.apply_all(new_tint_color, ctx, prev_attrs);
                true
            }
            (Self::Standard, Self::Standard) => {
                if new_tint_color != current_tint_color {
                    Shader::set_vec4(
                        &ctx.bindings,
                        ctx.shaders.std_uniforms.tint_color,
                        new_tint_color.rgba(),
                    );
                }
                false
            }
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
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum MaskMode {
    Push,
    Pop,
    Masked,
}

impl MaskMode {
    fn apply_all(
        &self,
        uniforms: &super::stdshaders::SharedUniforms,
        mask_level: u8,
        ctx: &mut OpenGlContext,
    ) {
        Shader::set_opaque(&ctx.bindings, uniforms.mask_id, 1);
        Shader::set_vec4(
            &ctx.bindings,
            uniforms.mask_bounds,
            (-1.0, 1.0, ctx.mask.width, ctx.mask.height),
        );
        Self::apply_change(
            &Self::Masked,
            self,
            uniforms,
            mask_level,
            ctx,
            false,
        );
    }

    fn apply_change(
        current: &Self,
        new: &Self,
        uniforms: &super::stdshaders::SharedUniforms,
        mask_level: u8,
        ctx: &mut OpenGlContext,
        shader_changed: bool,
    ) {
        let mask_bounds_zero = (-1.0, 1.0, ctx.mask.width, ctx.mask.height);
        match (current, new) {
            (Self::Masked, Self::Masked) if !shader_changed => (),
            (Self::Push, Self::Push) | (Self::Pop, Self::Pop) => (),
            (Self::Push, Self::Pop) => unsafe {
                ctx.bindings.BlendEquation(FUNC_REVERSE_SUBTRACT);
            },
            (Self::Pop, Self::Push) => unsafe {
                ctx.bindings.BlendEquation(FUNC_ADD);
            },
            (Self::Masked, Self::Push) => {
                Shader::set_opaque(&ctx.bindings, uniforms.mask_id, 1);
                Shader::set_vec4(
                    &ctx.bindings,
                    uniforms.mask_bounds,
                    mask_bounds_zero,
                );
                ctx.mask.bind_fbo(&ctx.bindings);
                unsafe {
                    ctx.bindings.ActiveTexture(TEXTURE1);
                    ctx.bindings.BindTexture(TEXTURE_2D, 0);
                    ctx.bindings.BlendEquation(FUNC_ADD);
                    ctx.bindings.BlendFunc(CONSTANT_COLOR, ONE);
                }
            }
            (Self::Masked, Self::Pop) => {
                Shader::set_opaque(&ctx.bindings, uniforms.mask_id, 1);
                Shader::set_vec4(
                    &ctx.bindings,
                    uniforms.mask_bounds,
                    mask_bounds_zero,
                );
                ctx.mask.bind_fbo(&ctx.bindings);
                unsafe {
                    ctx.bindings.ActiveTexture(TEXTURE1);
                    ctx.bindings.BindTexture(TEXTURE_2D, 0);
                    ctx.bindings.BlendEquation(FUNC_REVERSE_SUBTRACT);
                    ctx.bindings.BlendFunc(CONSTANT_COLOR, ONE);
                }
            }
            (_, Self::Masked) => {
                ctx.mask.restore_fbo(&ctx.bindings);
                Shader::set_opaque(&ctx.bindings, uniforms.mask_id, 1);
                let mask_bounds = if mask_level == 0 {
                    mask_bounds_zero
                } else {
                    let num_layers = f32::from(MASK_LEVELS);
                    let prev_layer = f32::from(mask_level - 1) / num_layers;
                    (
                        prev_layer,
                        num_layers,
                        mask_bounds_zero.2,
                        mask_bounds_zero.3,
                    )
                };
                Shader::set_vec4(
                    &ctx.bindings,
                    uniforms.mask_bounds,
                    mask_bounds,
                );
                unsafe {
                    ctx.bindings.ActiveTexture(TEXTURE1);
                    ctx.bindings.BindTexture(TEXTURE_2D, ctx.mask.tex_id());
                    ctx.bindings.BlendEquation(FUNC_ADD);
                    ctx.bindings.BlendFunc(SRC_ALPHA, ONE_MINUS_SRC_ALPHA);
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct DrawParams {
    transform: Mat4,
    tint_color: Color,
    texture: Texture,
    shader_exclusive: ShaderExclusive,
    mask_mode: MaskMode,
    mask_level: u8,
}

impl DrawParams {
    pub(crate) fn new() -> Self {
        Self {
            transform: Mat4::default(),
            tint_color: Color::WHITE,
            texture: Texture::default(),
            shader_exclusive: ShaderExclusive::Standard,
            mask_mode: MaskMode::Masked,
            mask_level: 0,
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

    pub fn push_mask(&mut self) {
        self.mask_mode = MaskMode::Push;
        self.mask_level += 1;
    }

    pub fn pop_mask(&mut self) {
        self.mask_mode = MaskMode::Pop;
        self.mask_level -= 1;
    }

    pub fn commit_mask(&mut self) {
        self.mask_mode = MaskMode::Masked;
    }

    #[rustfmt::skip]
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

    #[rustfmt::skip]
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

    #[rustfmt::skip]
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

    #[rustfmt::skip]
    pub fn body_edge(&mut self, edge: f32, smoothing: f32) {
        use ShaderExclusive::*;
        if let Sdf { ref mut distance_edges, .. } = self.shader_exclusive {
            let smoothing = smoothing.max(0.0) / 2.0;
            distance_edges.0 = (edge - smoothing).max(0.0);
            distance_edges.1 = edge + smoothing;
        } else {
            debug_assert!(
                false,
                "DrawParams::body_edge should only be used with Sdf shader",
            );
        }
    }

    #[rustfmt::skip]
    pub fn outline_edge(&mut self, edge: f32, smoothing: f32) {
        use ShaderExclusive::*;
        if let Sdf { ref mut distance_edges, .. } = self.shader_exclusive {
            let smoothing = smoothing.max(0.0) / 2.0;
            distance_edges.2 = (edge - smoothing).max(0.0);
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
        self.texture.bind(ctx);
        self.mask_mode.apply_all(&uniforms, self.mask_level, ctx);
    }

    fn apply_change(current: &Self, new: &mut Self, ctx: &mut OpenGlContext) {
        let shader_changed = ShaderExclusive::apply_change(
            &current.shader_exclusive,
            &new.shader_exclusive,
            current.tint_color,
            new.tint_color,
            &ctx,
        );
        let uniforms = match &new.shader_exclusive {
            ShaderExclusive::Standard => ctx.shaders.std_uniforms.common,
            ShaderExclusive::Sdf { .. } => ctx.shaders.sdf_uniforms.common,
        };
        if shader_changed || new.transform != current.transform {
            Shader::set_mat4(
                &ctx.bindings,
                uniforms.transform,
                new.transform.as_ref(),
            );
        }
        if shader_changed || new.texture != current.texture {
            unsafe { ctx.bindings.ActiveTexture(TEXTURE0) };
            new.texture.bind(ctx);
        }
        MaskMode::apply_change(
            &current.mask_mode,
            &new.mask_mode,
            &uniforms,
            new.mask_level,
            ctx,
            shader_changed,
        );
    }
}
