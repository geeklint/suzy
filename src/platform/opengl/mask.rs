use std::rc::Rc;

use super::context::OpenGlBindings;
use super::context::bindings::{
    BLEND_COLOR,
    COLOR_ATTACHMENT0,
    CLAMP_TO_EDGE,
    FRAMEBUFFER_BINDING,
    FRAMEBUFFER,
    FRAMEBUFFER_COMPLETE,
    NEAREST,
    RGBA,
    TEXTURE_2D,
    TEXTURE_MAG_FILTER,
    TEXTURE_MIN_FILTER,
    TEXTURE_WRAP_S,
    TEXTURE_WRAP_T,
    UNSIGNED_BYTE,
};
use super::context::bindings::types::*;
use super::texture::TextureData;

gl_object! { FramebufferData, GenFramebuffers, DeleteFramebuffers, 1 }

pub struct Mask {
    tex: TextureData,
    fbo: FramebufferData,
    old_fbo: GLint,
    old_blend_color: [f32; 4],
    pub width: f32,
    pub height: f32,
}

impl Mask {
    pub fn new() -> Self {
        Self {
            tex: TextureData::new(),
            fbo: FramebufferData::new(),
            old_fbo: 0,
            old_blend_color: [0.0; 4],
            width: 0.0,
            height: 0.0,
        }
    }

    pub fn tex_id(&self) -> GLuint {
        self.tex.ids[0]
    }

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

    pub fn mask_size(
        &mut self,
        gl: &Rc<OpenGlBindings>,
        width: u32,
        height: u32,
    ) {
        self.tex.check_ready(gl);
        self.fbo.check_ready(gl);
        unsafe {
            gl.BindTexture(TEXTURE_2D, self.tex.ids[0]);
            gl.TexImage2D(
                TEXTURE_2D,
                0,
                RGBA as _,
                width as GLsizei,
                height as GLsizei,
                0,
                RGBA,
                UNSIGNED_BYTE,
                std::ptr::null(),
            );
            gl.TexParameteri(TEXTURE_2D, TEXTURE_MIN_FILTER, NEAREST as _);
            gl.TexParameteri(TEXTURE_2D, TEXTURE_MAG_FILTER, NEAREST as _);
            gl.TexParameteri(TEXTURE_2D, TEXTURE_WRAP_S, CLAMP_TO_EDGE as _);
            gl.TexParameteri(TEXTURE_2D, TEXTURE_WRAP_T, CLAMP_TO_EDGE as _);

            let mut old_fbo: GLint = 0;
            gl.GetIntegerv(FRAMEBUFFER_BINDING, &mut old_fbo as *mut _);
            gl.BindFramebuffer(FRAMEBUFFER, self.fbo.ids[0]);
            gl.FramebufferTexture2D(
                FRAMEBUFFER,
                COLOR_ATTACHMENT0,
                TEXTURE_2D,
                self.tex.ids[0],
                0,
            );
            assert_eq!(
                gl.CheckFramebufferStatus(FRAMEBUFFER),
                FRAMEBUFFER_COMPLETE,
            );
            gl.BindFramebuffer(FRAMEBUFFER, old_fbo as _);
        }
        self.width = width as f32;
        self.height = height as f32;
    }
}
