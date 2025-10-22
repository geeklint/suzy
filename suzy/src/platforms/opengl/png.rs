/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2025 Violet Leonard */

use std::{convert::TryInto, fs::File, io::BufReader, path::Path, rc::Rc};

use png::{BitDepth, ColorType, Decoder, Transformations};

use crate::platforms::opengl;

use opengl::{
    context::{
        bindings::{
            types::{GLenum, GLint},
            UNPACK_ALIGNMENT, UNSIGNED_BYTE,
        },
        short_consts::{LUMINANCE, LUMINANCE_ALPHA, RGB, RGBA},
    },
    OpenGlBindings, PopulateTexture, PopulateTextureUtil, Texture,
    TextureSize,
};

pub trait LoadPng {
    fn load_png(self) -> Texture;
    fn load_png_as_sdf(self) -> Texture;
}

impl<T> LoadPng for T
where
    T: 'static + AsRef<Path>,
{
    fn load_png(self) -> Texture {
        Texture::new(Rc::new(Populator {
            path: self,
            sdf: false,
        }))
    }

    fn load_png_as_sdf(self) -> Texture {
        Texture::new(Rc::new(Populator {
            path: self,
            sdf: true,
        }))
    }
}

struct Populator<T> {
    path: T,
    sdf: bool,
}

impl<T> std::fmt::Debug for Populator<T>
where
    T: AsRef<Path>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoadPngPopulator")
            .field("path", &self.path.as_ref())
            .finish()
    }
}

impl<T> PopulateTexture for Populator<T>
where
    T: AsRef<Path>,
{
    fn populate(
        &self,
        gl: &OpenGlBindings,
        target: GLenum,
    ) -> Result<TextureSize, String> {
        let mut decoder = Decoder::new(BufReader::new(
            File::open(&self.path).map_err(|e| e.to_string())?,
        ));
        decoder.set_transformations(Transformations::normalize_to_color8());
        let mut reader = decoder.read_info().map_err(|e| e.to_string())?;
        let mut pixels =
            vec![0; reader.output_buffer_size().ok_or("image was too big")?];
        let info =
            reader.next_frame(&mut pixels).map_err(|e| e.to_string())?;
        assert_eq!(info.bit_depth, BitDepth::Eight, "png crate didn't return eight-bit data despite normalize_to_color8");
        let (samples, format) = match info.color_type {
            ColorType::Grayscale => (1, LUMINANCE),
            ColorType::GrayscaleAlpha => (2, LUMINANCE_ALPHA),
            ColorType::Rgb => (3, RGB),
            ColorType::Rgba => (4, RGBA),
            _ => panic!("png crate didn't return color data despite normalize_to_color8"),
        };
        let short_width: u16 =
            info.width.try_into().map_err(|_| "image was too wide")?;
        let short_height: u16 =
            info.height.try_into().map_err(|_| "image was too tall")?;
        let texture_width: u16 = info
            .width
            .next_power_of_two()
            .try_into()
            .map_err(|_| "image was too wide")?;
        let texture_height: u16 = info
            .height
            .next_power_of_two()
            .try_into()
            .map_err(|_| "image was too tall")?;
        let row = usize::from(short_width) * samples;
        let alignment: GLint = if row == info.line_size {
            1
        } else if row.next_multiple_of(2) == info.line_size {
            2
        } else if row.next_multiple_of(4) == info.line_size {
            4
        } else if row.next_multiple_of(8) == info.line_size {
            8
        } else {
            // TODO: we could realign the pixels instead of failing
            return Err("png had incompatible alignment".to_string());
        };
        // swap the png rows since we use a bottom-left origin but png is a
        // top-left origin
        let mut rows = pixels.chunks_exact_mut(info.line_size);
        while let [Some(a), Some(b)] = [rows.next(), rows.next_back()] {
            a.swap_with_slice(b);
        }
        unsafe {
            gl.PixelStorei(UNPACK_ALIGNMENT, alignment);
        }
        if texture_width == short_width && texture_height == short_height {
            unsafe {
                gl.TexImage2D(
                    target,
                    0,
                    format.into(),
                    texture_width.into(),
                    texture_height.into(),
                    0,
                    format.into(),
                    UNSIGNED_BYTE,
                    pixels.as_ptr().cast(),
                );
            }
            PopulateTextureUtil::default_params(gl, target);
            Ok(TextureSize {
                image_width: short_width.into(),
                image_height: short_height.into(),
                texture_width,
                texture_height,
                is_sdf: self.sdf,
            })
        } else {
            unsafe {
                gl.TexImage2D(
                    target,
                    0,
                    format.into(),
                    texture_width.into(),
                    texture_height.into(),
                    0,
                    format.into(),
                    UNSIGNED_BYTE,
                    std::ptr::null(),
                );
                gl.TexSubImage2D(
                    target,
                    0,
                    0,
                    0,
                    info.width
                        .try_into()
                        .expect("width fit in a u16 but not an i32"),
                    info.height
                        .try_into()
                        .expect("height fit in a u16 but not an i32"),
                    format.into(),
                    UNSIGNED_BYTE,
                    pixels.as_ptr().cast(),
                );
            }
            PopulateTextureUtil::default_params(gl, target);
            Ok(TextureSize {
                image_width: short_width.into(),
                image_height: short_height.into(),
                texture_width,
                texture_height,
                is_sdf: self.sdf,
            })
        }
    }

    fn texture_key(&self) -> &[u8] {
        self.path.as_ref().as_os_str().as_encoded_bytes()
    }

    fn debug(&self) -> &dyn std::fmt::Debug {
        self
    }
}
