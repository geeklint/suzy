/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

#![allow(missing_docs)]

use crate::graphics;
use crate::graphics::Color;

use super::context::bindings::{
    CONSTANT_COLOR, FUNC_ADD, FUNC_REVERSE_SUBTRACT, ONE, ONE_MINUS_SRC_ALPHA,
    SRC_ALPHA, TEXTURE0, TEXTURE1, TEXTURE_2D,
};
use super::shader::Shader;
use super::texture::Texture;
use super::Mat4;
use super::OpenGlContext;

pub(super) const MASK_LEVELS: u8 = 4;

#[derive(Clone, Copy, Debug, PartialEq)]
enum MaskMode {
    Push,
    Pop,
    Masked,
}

impl MaskMode {
    fn apply_all(&self, mask_level: u8, ctx: &mut OpenGlContext) {
        Shader::set_opaque(&ctx.bindings, ctx.shaders.uniforms.mask_id, 1);
        Shader::set_vec4(
            &ctx.bindings,
            ctx.shaders.uniforms.mask_bounds,
            (-1.0, 1.0, ctx.mask.width, ctx.mask.height),
        );
        Self::apply_change(&Self::Masked, self, mask_level, ctx);
    }

    fn apply_change(
        current: &Self,
        new: &Self,
        mask_level: u8,
        ctx: &mut OpenGlContext,
    ) {
        let mask_bounds_zero = (-1.0, 1.0, ctx.mask.width, ctx.mask.height);
        match (current, new) {
            (Self::Masked, Self::Masked)
            | (Self::Push, Self::Push)
            | (Self::Pop, Self::Pop) => (),
            (Self::Push, Self::Pop) => unsafe {
                ctx.bindings.BlendEquation(FUNC_REVERSE_SUBTRACT);
            },
            (Self::Pop, Self::Push) => unsafe {
                ctx.bindings.BlendEquation(FUNC_ADD);
            },
            (Self::Masked, Self::Push) => {
                Shader::set_opaque(
                    &ctx.bindings,
                    ctx.shaders.uniforms.mask_id,
                    1,
                );
                Shader::set_vec4(
                    &ctx.bindings,
                    ctx.shaders.uniforms.mask_bounds,
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
                Shader::set_opaque(
                    &ctx.bindings,
                    ctx.shaders.uniforms.mask_id,
                    1,
                );
                Shader::set_vec4(
                    &ctx.bindings,
                    ctx.shaders.uniforms.mask_bounds,
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
                Shader::set_opaque(
                    &ctx.bindings,
                    ctx.shaders.uniforms.mask_id,
                    1,
                );
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
                    ctx.shaders.uniforms.mask_bounds,
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
    texture: Texture,
    tint_color: Color,
    mask_mode: MaskMode,
    mask_level: u8,
    sdf_values: (f32, f32, f32, f32),
    sdf_chan_mask: (f32, f32, f32, f32),
}

impl DrawParams {
    pub(crate) fn new() -> Self {
        Self {
            transform: Mat4::default(),
            texture: Texture::default(),
            tint_color: Color::WHITE,
            mask_mode: MaskMode::Masked,
            mask_level: 0,
            sdf_values: (0.0, 0.0, 0.0, 0.0),
            sdf_chan_mask: (0.0, 0.0, 0.0, 0.0),
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
        self.sdf_values.2 = 0.0;
    }

    pub fn sdf_mode(&mut self) {
        self.sdf_values.2 = 1.0;
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
    pub fn sdf_chan_mask(&mut self, mask: (u8, u8, u8, u8)) {
        if self.sdf_values.2 > 0.0 {
            self.sdf_chan_mask = (
                mask.0 as f32 / 255.0,
                mask.1 as f32 / 255.0,
                mask.2 as f32 / 255.0,
                mask.3 as f32 / 255.0,
            );
        } else {
            debug_assert!(
                false,
                "DrawParams::sdf_chan_mask should only be used in sdf mode",
            );
        }
    }

    #[rustfmt::skip]
    pub fn sdf_edge(&mut self, edge: f32, smoothing: f32) {
        if self.sdf_values.2 > 0.0 {
            let smoothing = smoothing.max(0.0) / 2.0;
            self.sdf_values.0 = (edge - smoothing).max(0.0);
            self.sdf_values.1 = edge + smoothing;
        } else {
            debug_assert!(
                false,
                "DrawParams::body_edge should only be used with Sdf shader",
            );
        }
    }
}

impl graphics::DrawParams<OpenGlContext> for DrawParams {
    fn apply_all(&mut self, ctx: &mut OpenGlContext) {
        ctx.shaders.shader.make_current(&ctx.bindings, None);
        Shader::set_mat4(
            &ctx.bindings,
            ctx.shaders.uniforms.transform,
            self.transform.as_ref(),
        );
        Shader::set_opaque(&ctx.bindings, ctx.shaders.uniforms.tex_id, 0);
        unsafe { ctx.bindings.ActiveTexture(TEXTURE0) };
        self.texture.bind(ctx);
        Shader::set_vec4(
            &ctx.bindings,
            ctx.shaders.uniforms.tint_color,
            self.tint_color.rgba(),
        );
        self.mask_mode.apply_all(self.mask_level, ctx);
        Shader::set_vec4(
            &ctx.bindings,
            ctx.shaders.uniforms.sdf_values,
            self.sdf_values,
        );
        Shader::set_vec4(
            &ctx.bindings,
            ctx.shaders.uniforms.sdf_chan_mask,
            self.sdf_chan_mask,
        );
    }

    fn apply_change(current: &Self, new: &mut Self, ctx: &mut OpenGlContext) {
        if new.transform != current.transform {
            Shader::set_mat4(
                &ctx.bindings,
                ctx.shaders.uniforms.transform,
                new.transform.as_ref(),
            );
        }
        if new.texture != current.texture {
            Shader::set_opaque(&ctx.bindings, ctx.shaders.uniforms.tex_id, 0);
            unsafe { ctx.bindings.ActiveTexture(TEXTURE0) };
            new.texture.bind(ctx);
        }
        if new.tint_color != current.tint_color {
            Shader::set_vec4(
                &ctx.bindings,
                ctx.shaders.uniforms.tint_color,
                new.tint_color.rgba(),
            );
        }
        MaskMode::apply_change(
            &current.mask_mode,
            &new.mask_mode,
            new.mask_level,
            ctx,
        );
        if new.sdf_values != current.sdf_values {
            Shader::set_vec4(
                &ctx.bindings,
                ctx.shaders.uniforms.sdf_values,
                new.sdf_values,
            );
        }
        if new.sdf_chan_mask != current.sdf_chan_mask {
            Shader::set_vec4(
                &ctx.bindings,
                ctx.shaders.uniforms.sdf_chan_mask,
                new.sdf_chan_mask,
            );
        }
    }
}
