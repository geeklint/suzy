use std::ffi::c_void;

use gl::types::*;

pub struct Texture {
    id: GLuint,
    width: f32,
    height: f32,
}

impl Texture {
    pub fn from_gl(id: GLuint, width: f32, height: f32) -> Self {
        Self { id, width, height }
    }

    pub unsafe fn create(
        width: GLsizei,
        height: GLsizei,
        format: GLenum,
        type_: GLenum,
        data: *const c_void,
    ) -> Self {
        Self::create_custom(
            gl::GL_RGBA,
            width, height,
            format, type_,
            data
        )
    }

    pub unsafe fn create_opaque(
        width: GLsizei,
        height: GLsizei,
        format: GLenum,
        type_: GLenum,
        data: *const c_void,
    ) -> Self {
        Self::create_custom(
            gl::GL_RGB,
            width, height,
            format, type_,
            data
        )
    }

    pub unsafe fn create_gray(
        width: GLsizei,
        height: GLsizei,
        format: GLenum,
        type_: GLenum,
        data: *const c_void,
    ) -> Self {
        Self::create_custom(
            gl::GL_RED,
            width, height,
            format, type_,
            data
        )
    }

    pub unsafe fn create_custom(
        internalformat: GLint,
        width: GLsizei,
        height: GLsizei,
        format: GLenum,
        type_: GLenum,
        data: *const c_void,
    ) -> Self {
        let id = 0;
        gl::GenTextures(1, &id as *mut _);
        gl::BindTexture(gl::GL_TEXTURE_2D, id);
        gl::TexImage2D(
            gl::GL_TEXTURE_2D,
            0,
            internalformat,
            width, height,
            0,
            format, type_,
            data,
        );
        Self::from_gl(id, width as f32, height as f32)
    }
}
