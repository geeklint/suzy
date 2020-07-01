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
};

use super::TextureSize;

pub trait PopulateTextureDynClone {
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

pub trait PopulateTexture: PopulateTextureDynClone {
    fn populate(&self, gl: &OpenGlBindings) -> TextureSize;

    fn get_known_size(&self) -> Option<(f32, f32)> { None }
}

#[derive(Clone, Debug)]
struct FnPopulateTexture<F: 'static + Clone + Fn(&OpenGlBindings) -> TextureSize>(F);

impl<F> PopulateTexture for FnPopulateTexture<F>
where
    F: 'static + Clone + Fn(&OpenGlBindings) -> TextureSize
{
    fn populate(&self, gl: &OpenGlBindings) -> TextureSize {
        (self.0)(gl)
    }
}

pub struct TexturePopulator(Box<dyn PopulateTexture>);

impl TexturePopulator {
    pub fn new<T, U>(populator: T) -> Self
    where
        T: Into<Box<U>>,
        U: 'static + PopulateTexture,
    {
        TexturePopulator(populator.into())
    }
}

impl<F> From<F> for TexturePopulator
where
    F: Fn(&OpenGlBindings) -> TextureSize
{
    fn from(populator: F) -> Self {
        TexturePopulator(Box::new(FnPopulateTexture(populator)))
    }
}

impl Clone for Box<dyn PopulateTexture> {
    fn clone(&self) -> Self {
        self.clone_boxed()
    }
}

pub struct PopulateTextureUtil;

impl PopulateTextureUtil {
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
        pixels: &[u8],
    ) -> TextureSize {
        if width.is_power_of_two() && height.is_power_of_two() {
            unsafe {
                gl.TexImage2D(
                    TEXTURE_2D,
                    0,
                    format,
                    width.into(),
                    height.into(),
                    0,
                    format,
                    UNSIGNED_BYTE,
                    pixels.as_ptr() as *const _,
                );
            }
            TextureSize {
                image_width: width as f32,
                image_height: height as f32,
                texture_width: width as f32,
                texture_height: height as f32,
            }
        } else {
            let width = width.into::<GLsizei>();
            let height = height.into::<GLsizei>();
            let texture_width = width.next_power_of_two();
            let texture_height = height.next_power_of_two();
            unsafe {
                gl.TexImage2D(
                    TEXTURE_2D,
                    0,
                    format,
                    texture_width,
                    texture_height,
                    0,
                    format,
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
                    format,
                    UNSIGNED_BYTE,
                    pixels.as_ptr() as *const _,
                );
            }
            TextureSize {
                image_width: width as f32,
                image_height: height as f32,
                texture_width,
                texture_height,
            }
        }
    }

    pub fn populate_alpha(
        gl: &OpenGlBindings,
        width: u16,
        height: u16,
        pixels: &[u8],
    ) -> TextureSize {
        assert_eq!(width * height, pixels.len());
        Self::populate_format(gl, ALPHA as _, width, height, pixels)
    }

    pub fn populate_rgb(
        gl: &OpenGlBindings,
        width: u16,
        height: u16,
        pixels: &[u8],
    ) -> TextureSize {
        assert_eq!(width * height * 3, pixels.len());
        Self::populate_format(gl, RGB as _, width, height, pixels)
    }

    pub fn populate_rgba(
        gl: &OpenGlBindings,
        width: u16,
        height: u16,
        pixels: &[u8],
    ) -> TextureSize {
        assert_eq!(width * height * 4, pixels.len());
        Self::populate_format(gl, RGBA as _, width, height, pixels)
    }
}

#[derive(Clone, Copy, Default, PartialEq)]
pub(super) struct DefaultTexturePopulator;

impl PopulateTexture for DefaultTexturePopulator {
    fn get_known_size(&self) -> Option<(f32, f32)> {
        Some((2.0, 2.0))
    }

    fn populate(&self, gl: &OpenGlBindings) -> TextureSize {
        let pixels: [u8; 12] = [0xff; 12];
        let size = PopulateTextureUtil::populate_rgb(
            gl, 2, 2, &pixels
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
const ERRTEX: &'static [u8] = include_bytes!("errtex.data");

impl PopulateTexture for ErrorTexturePopulator {
    fn get_known_size(&self) -> Option<(f32, f32)> {
        Some((16.0, 16.0))
    }

    fn populate(&self, gl: &OpenGlBindings) -> TextureSize {
        let size = PopulateTextureUtil::populate_rgb(
            gl, ERRTEX_SIDE, ERRTEX_SIDE, ERRTEX
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
