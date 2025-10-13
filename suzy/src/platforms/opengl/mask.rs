/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use std::rc::Rc;

use super::context::{
    bindings::{
        types::{GLint, GLuint},
        COLOR_ATTACHMENT0, FRAMEBUFFER, FRAMEBUFFER_BINDING,
        FRAMEBUFFER_COMPLETE, TEXTURE_2D, TEXTURE_MAG_FILTER,
        TEXTURE_MIN_FILTER, TEXTURE_WRAP_S, TEXTURE_WRAP_T, UNSIGNED_BYTE,
    },
    short_consts::{CLAMP_TO_EDGE, NEAREST, RGBA},
    OpenGlBindings,
};

pub struct Mask {
    pub texture: GLuint,
    pub fbo: GLuint,
    pub width: f32,
    pub height: f32,
}

impl Mask {
    pub fn new(gl: &OpenGlBindings) -> Self {
        let mut texture = 0;
        let mut fbo = 0;
        unsafe {
            gl.GenTextures(1, &raw mut texture);
            gl.GenFramebuffers(1, &raw mut fbo);
        }
        Self {
            texture,
            fbo,
            width: 0.0,
            height: 0.0,
        }
    }

    /*
    pub fn bind_fbo(&mut self, gl: &OpenGlBindings) {
        let color = 1.0 / f32::from(super::drawparams::MASK_LEVELS);
        unsafe {
            gl.GetIntegerv(FRAMEBUFFER_BINDING, &mut self.old_fbo as *mut _);
            gl.GetFloatv(BLEND_COLOR, self.old_blend_color.as_mut_ptr());
            gl.BindFramebuffer(FRAMEBUFFER, self.fbo.ids[0]);
            gl.BlendColor(color, color, color, color);
        }
    }

    pub fn restore_fbo(&mut self, gl: &OpenGlBindings) {
        assert_eq!(self.old_fbo, 0);
        let [r, g, b, a] = self.old_blend_color;
        unsafe {
            gl.BindFramebuffer(FRAMEBUFFER, self.old_fbo as _);
            gl.BlendColor(r, g, b, a);
        }
        self.old_fbo = 0;
    }
    */

    pub fn configure_for_size(
        &mut self,
        gl: &Rc<OpenGlBindings>,
        width: u16,
        height: u16,
    ) {
        unsafe {
            gl.BindTexture(TEXTURE_2D, self.texture);
            gl.TexImage2D(
                TEXTURE_2D,
                0,
                RGBA.into(),
                width.into(),
                height.into(),
                0,
                RGBA.into(),
                UNSIGNED_BYTE,
                std::ptr::null(),
            );
            gl.TexParameteri(TEXTURE_2D, TEXTURE_MIN_FILTER, NEAREST.into());
            gl.TexParameteri(TEXTURE_2D, TEXTURE_MAG_FILTER, NEAREST.into());
            gl.TexParameteri(TEXTURE_2D, TEXTURE_WRAP_S, CLAMP_TO_EDGE.into());
            gl.TexParameteri(TEXTURE_2D, TEXTURE_WRAP_T, CLAMP_TO_EDGE.into());

            let mut old_fbo: GLint = 0;
            gl.GetIntegerv(FRAMEBUFFER_BINDING, &raw mut old_fbo);
            gl.BindFramebuffer(FRAMEBUFFER, self.fbo);
            gl.FramebufferTexture2D(
                FRAMEBUFFER,
                COLOR_ATTACHMENT0,
                TEXTURE_2D,
                self.texture,
                0,
            );
            assert_eq!(
                gl.CheckFramebufferStatus(FRAMEBUFFER),
                FRAMEBUFFER_COMPLETE,
            );
            gl.BindFramebuffer(FRAMEBUFFER, old_fbo as GLuint);
        }
        self.width = width.into();
        self.height = height.into();
    }
}
