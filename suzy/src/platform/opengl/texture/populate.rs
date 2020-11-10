/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::platform::opengl;
use opengl::context::OpenGlBindings;
use opengl::context::bindings::types::*;
use opengl::context::bindings::{
    ALPHA,
    CLAMP_TO_EDGE,
    LINEAR,
    NEAREST,
    REPEAT,
    RGB,
    RGBA,
    TEXTURE_2D,
    TEXTURE_MAG_FILTER,
    TEXTURE_MIN_FILTER,
    TEXTURE_WRAP_S,
    TEXTURE_WRAP_T,
    UNSIGNED_BYTE,
    UNPACK_ALIGNMENT,
};

use super::TextureSize;

static DEFAULT_POPULATE_DEBUG: DefaultPopulateDebug = DefaultPopulateDebug;

struct DefaultPopulateDebug;

impl std::fmt::Debug for DefaultPopulateDebug {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        f.write_str("dyn PopulateTexture")
    }
}

/// A trait which allows cloning a PopulateTexture trait object.
pub trait PopulateTextureDynClone {
    /// Create a clone of this texture populator, as a boxed trait object.
    fn clone_boxed(&self) -> Box<dyn PopulateTexture>;
}

impl<T> PopulateTextureDynClone for T
where
    T: 'static + PopulateTexture + Clone
{
    fn clone_boxed(&self) -> Box<dyn PopulateTexture> {
        Box::new(self.clone())
    }
}

/// A trait which describes how to populate a texture.
///
/// This is automatically implemented for closures which match the signature
/// of the `populate` method, which may be considered the simplest type of
/// texture populator.
pub trait PopulateTexture: PopulateTextureDynClone {
    /// Execute the necesary opengl commands to populate a texture.
    fn populate(&self, gl: &OpenGlBindings) -> Result<TextureSize, ()>;

    /// This function should return Some, if the populator can perfectly
    /// determine the size the texture will be without loading it.
    fn get_known_size(&self) -> Option<(f32, f32)> { None }

    /// An implementation may override this with a better debug implementation.
    fn debug(&self) -> &dyn std::fmt::Debug { &DEFAULT_POPULATE_DEBUG }
}

impl<F> PopulateTexture for F
where
    F: 'static + Clone,
    F: for<'a> Fn(&'a OpenGlBindings) -> Result<TextureSize, ()>
{
    fn populate(&self, gl: &OpenGlBindings) -> Result<TextureSize, ()> {
        (self)(gl)
    }
}

impl Clone for Box<dyn PopulateTexture> {
    fn clone(&self) -> Self {
        self.clone_boxed()
    }
}

impl std::fmt::Debug for Box<dyn PopulateTexture> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        self.debug().fmt(f)
    }
}

/// This type provides some helper functions for common texture populator
/// needs.
pub struct PopulateTextureUtil;

impl PopulateTextureUtil {
    /// Set the default texture parameters for magnification and wrapping.
    pub fn default_params(gl: &OpenGlBindings) {
        unsafe {
            gl.TexParameteri(TEXTURE_2D, TEXTURE_MIN_FILTER, LINEAR as _);
            gl.TexParameteri(TEXTURE_2D, TEXTURE_MAG_FILTER, LINEAR as _);
            gl.TexParameteri(TEXTURE_2D, TEXTURE_WRAP_S, CLAMP_TO_EDGE as _);
            gl.TexParameteri(TEXTURE_2D, TEXTURE_WRAP_T, CLAMP_TO_EDGE as _);
        }
    }

    fn populate_format(
        gl: &OpenGlBindings,
        format: GLint,
        width: u16,
        height: u16,
        alignment: u16,
        pixels: &[u8],
    ) -> Result<TextureSize, ()> {
        unsafe {
            gl.PixelStorei(UNPACK_ALIGNMENT, alignment.into());
        }
        if width.is_power_of_two() && height.is_power_of_two() {
            unsafe {
                gl.TexImage2D(
                    TEXTURE_2D,
                    0,
                    format,
                    width.into(),
                    height.into(),
                    0,
                    format as GLenum,
                    UNSIGNED_BYTE,
                    pixels.as_ptr() as *const _,
                );
            }
            Ok(TextureSize {
                image_width: width as f32,
                image_height: height as f32,
                texture_width: width as f32,
                texture_height: height as f32,
            })
        } else {
            let texture_width = width.next_power_of_two().into();
            let texture_height = height.next_power_of_two().into();
            let width: GLsizei = width.into();
            let height: GLsizei = height.into();
            unsafe {
                gl.TexImage2D(
                    TEXTURE_2D,
                    0,
                    format,
                    texture_width,
                    texture_height,
                    0,
                    format as GLenum,
                    UNSIGNED_BYTE,
                    std::ptr::null(),
                );
                gl.TexSubImage2D(
                    TEXTURE_2D,
                    0,
                    0,
                    0,
                    width,
                    height,
                    format as GLenum,
                    UNSIGNED_BYTE,
                    pixels.as_ptr() as *const _,
                );
            }
            Ok(TextureSize {
                image_width: width as f32,
                image_height: height as f32,
                texture_width: texture_width as f32,
                texture_height: texture_height as f32,
            })
        }
    }

    #[doc(hidden)]
    pub fn data_len(width: u16, height: u16, alignment: u16, channels: u16)
        -> usize
    {
        assert!(match alignment { 1 | 2 | 4 => true, _ => false });
        let pixel_row_len = width * channels;
        let padding = (alignment - (pixel_row_len % alignment)) % alignment;
        let full_row_len = pixel_row_len + padding;
        (full_row_len as usize) * (height as usize)
    }

    /// Populate an image with a single channel.
    pub fn populate_alpha(
        gl: &OpenGlBindings,
        width: u16,
        height: u16,
        alignment: u16,
        pixels: &[u8],
    ) -> Result<TextureSize, ()> {
        assert_eq!(pixels.len(), Self::data_len(width, height, alignment, 1));
        Self::populate_format(gl, ALPHA as _, width, height, alignment, pixels)
    }

    /// Populate an texture with three channels.
    pub fn populate_rgb(
        gl: &OpenGlBindings,
        width: u16,
        height: u16,
        alignment: u16,
        pixels: &[u8],
    ) -> Result<TextureSize, ()> {
        assert_eq!(pixels.len(), Self::data_len(width, height, alignment, 3));
        Self::populate_format(gl, RGB as _, width, height, alignment, pixels)
    }

    /// Populate an texture with four channels.
    pub fn populate_rgba(
        gl: &OpenGlBindings,
        width: u16,
        height: u16,
        alignment: u16,
        pixels: &[u8],
    ) -> Result<TextureSize, ()> {
        assert_eq!(pixels.len(), Self::data_len(width, height, alignment, 4));
        Self::populate_format(gl, RGBA as _, width, height, alignment, pixels)
    }
}

#[derive(Clone, Copy, Default, PartialEq)]
pub(super) struct DefaultTexturePopulator;

impl PopulateTexture for DefaultTexturePopulator {
    fn get_known_size(&self) -> Option<(f32, f32)> {
        Some((2.0, 2.0))
    }

    fn populate(&self, gl: &OpenGlBindings) -> Result<TextureSize, ()> {
        let pixels: [u8; 12] = [0xff; 12];
        unsafe {
            gl.PixelStorei(UNPACK_ALIGNMENT, 1);
        }
        let size = PopulateTextureUtil::populate_rgb(
            gl, 2, 2, 1, &pixels
        );
        unsafe {
            gl.TexParameteri(TEXTURE_2D, TEXTURE_MIN_FILTER, NEAREST as _);
            gl.TexParameteri(TEXTURE_2D, TEXTURE_MAG_FILTER, NEAREST as _);
            gl.TexParameteri(TEXTURE_2D, TEXTURE_WRAP_S, CLAMP_TO_EDGE as _);
            gl.TexParameteri(TEXTURE_2D, TEXTURE_WRAP_T, CLAMP_TO_EDGE as _);
        }
        size
    }
}

#[derive(Clone, Copy, Default, PartialEq)]
pub(super) struct ErrorTexturePopulator;

const ERRTEX_SIDE: u16 = 16;
const ERRTEX: &[u8] = include_bytes!("errtex.data");

impl PopulateTexture for ErrorTexturePopulator {
    fn get_known_size(&self) -> Option<(f32, f32)> {
        Some((16.0, 16.0))
    }

    fn populate(&self, gl: &OpenGlBindings) -> Result<TextureSize, ()> {
        unsafe {
            gl.PixelStorei(UNPACK_ALIGNMENT, 1);
        }
        let size = PopulateTextureUtil::populate_rgb(
            gl, ERRTEX_SIDE, ERRTEX_SIDE, 1, ERRTEX
        );
        unsafe {
            gl.TexParameteri(TEXTURE_2D, TEXTURE_MIN_FILTER, NEAREST as _);
            gl.TexParameteri(TEXTURE_2D, TEXTURE_MAG_FILTER, NEAREST as _);
            gl.TexParameteri(TEXTURE_2D, TEXTURE_WRAP_S, REPEAT as _);
            gl.TexParameteri(TEXTURE_2D, TEXTURE_WRAP_T, REPEAT as _);
        }
        size
    }
}

#[derive(Clone, Debug)]
pub(super) struct AlphaTexturePopulator {
    pub(super) width: u16,
    pub(super) height: u16,
    pub(super) alignment: u16,
    pub(super) pixels: std::borrow::Cow<'static, [u8]>,
}

impl PopulateTexture for AlphaTexturePopulator {
    fn get_known_size(&self) -> Option<(f32, f32)> {
        Some((self.width as f32, self.height as f32))
    }

    fn populate(&self, gl: &OpenGlBindings) -> Result<TextureSize, ()> {
        unsafe {
            gl.PixelStorei(UNPACK_ALIGNMENT, self.alignment.into());
        }
        let size = PopulateTextureUtil::populate_alpha(
            gl, self.width, self.height, self.alignment, &self.pixels
        );
        PopulateTextureUtil::default_params(gl);
        size
    }
}

#[derive(Clone, Debug)]
pub(super) struct RGBTexturePopulator {
    pub(super) width: u16,
    pub(super) height: u16,
    pub(super) alignment: u16,
    pub(super) pixels: std::borrow::Cow<'static, [u8]>,
}

impl PopulateTexture for RGBTexturePopulator {
    fn get_known_size(&self) -> Option<(f32, f32)> {
        Some((self.width as f32, self.height as f32))
    }

    fn populate(&self, gl: &OpenGlBindings) -> Result<TextureSize, ()> {
        unsafe {
            gl.PixelStorei(UNPACK_ALIGNMENT, self.alignment.into());
        }
        let size = PopulateTextureUtil::populate_rgb(
            gl, self.width, self.height, self.alignment, &self.pixels
        );
        PopulateTextureUtil::default_params(gl);
        size
    }
}

#[derive(Clone, Debug)]
pub(super) struct RGBATexturePopulator {
    pub(super) width: u16,
    pub(super) height: u16,
    pub(super) alignment: u16,
    pub(super) pixels: std::borrow::Cow<'static, [u8]>,
}

impl PopulateTexture for RGBATexturePopulator {
    fn get_known_size(&self) -> Option<(f32, f32)> {
        Some((self.width as f32, self.height as f32))
    }

    fn populate(&self, gl: &OpenGlBindings) -> Result<TextureSize, ()> {
        let size = PopulateTextureUtil::populate_rgba(
            gl, self.width, self.height, self.alignment, &self.pixels
        );
        PopulateTextureUtil::default_params(gl);
        size
    }
}
