/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use std::{
    convert::TryInto,
    fmt::{self, Display, Formatter},
    path::Path,
};

use sdl2::{image::LoadSurface, pixels::PixelFormatEnum, surface::Surface};

use crate::platforms::opengl::{
    bindings::{RGB, RGBA, UNSIGNED_BYTE},
    image::{TextureBuilder, TextureLoadResult},
};

#[derive(Debug)]
struct SdlImageLoadError(String);

impl Display for SdlImageLoadError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        std::fmt::Debug::fmt(self, f)
    }
}

impl std::error::Error for SdlImageLoadError {}

impl From<String> for SdlImageLoadError {
    fn from(msg: String) -> Self {
        Self(msg)
    }
}

pub fn load_texture(path: &Path) -> TextureLoadResult {
    let surf = Surface::from_file(path)
        .map_err(|msg| Box::new(SdlImageLoadError(msg)))?;
    let (fmt, type_, surf, opaque) = match surf.pixel_format_enum() {
        PixelFormatEnum::BGR24 => (RGB, UNSIGNED_BYTE, surf, true),
        PixelFormatEnum::ABGR8888 => (RGBA, UNSIGNED_BYTE, surf, false),
        _ => {
            let conv = surf
                .convert_format(PixelFormatEnum::ABGR8888)
                .map_err(|msg| Box::new(SdlImageLoadError(msg)))?;
            (RGBA, UNSIGNED_BYTE, conv, false)
        }
    };
    let (width, height) = surf.size();
    let width = width.try_into().map_err(Box::new)?;
    let height = height.try_into().map_err(Box::new)?;
    let mut builder = if opaque {
        TextureBuilder::create_opaque(width, height)
    } else {
        TextureBuilder::create(width, height)
    };
    surf.with_lock(|pixels| {
        let row_len = pixels.len() / (height as usize);
        for row_index in 0..height {
            let start = (row_index as usize) * row_len;
            let end = start + row_len;
            let row = &pixels[start..end];
            let yoffset = height - row_index;
            builder.sub_image(
                0,
                yoffset,
                width,
                1,
                fmt,
                type_,
                row.as_ptr() as *const _,
            );
        }
    });
    Ok(builder.build())
}
