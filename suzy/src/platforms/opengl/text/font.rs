/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use std::{cell::OnceCell, rc::Rc};

use crate::platforms::opengl::{
    opengl_bindings::types::GLenum, OpenGlBindings, Texture, TextureSize,
};

#[derive(Debug)]
pub struct Font {
    pub data: FontData,
    pub bold: OnceCell<Rc<Font>>,
    pub italic: OnceCell<Rc<Font>>,
}

#[derive(Debug)]
pub struct FontData {
    pub texture: Texture,
    pub padding_ratio: f32,
    pub glyphs: Box<[Glyph]>,
    pub kerning: Box<[KerningPair]>,
    pub line_spacing: f32,
    pub ascent: f32,
    pub capline: f32,
    pub descent: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct Glyph {
    pub ch: char,
    pub advance: f32,
    pub bb_left: f32,
    pub bb_right: f32,
    pub bb_bottom: f32,
    pub bb_top: f32,
    pub tex_left: u16,
    pub tex_right: u16,
    pub tex_bottom: u16,
    pub tex_top: u16,
}

#[derive(Clone, Copy, Debug)]
pub struct KerningPair {
    pub left: char,
    pub right: char,
    pub kerning: f32,
}

impl FontData {
    pub fn kerning(&self, left: char, right: char) -> Option<f32> {
        self.kerning
            .binary_search_by_key(&(left, right), |item| {
                (item.left, item.right)
            })
            .map(|index| self.kerning[index].kerning)
            .ok()
    }

    pub fn glyph(&self, ch: char) -> Option<&Glyph> {
        self.glyphs
            .binary_search_by_key(&ch, |glyph| glyph.ch)
            .map(|index| &self.glyphs[index])
            .ok()
    }
}

impl Font {
    pub fn populate_font_atlas(
        gl: &OpenGlBindings,
        target: GLenum,
        width: u16,
        height: u16,
        data: &[u8],
    ) -> TextureSize {
        use crate::platforms::opengl::{
            context::short_consts::{ALPHA, CLAMP_TO_EDGE, LINEAR},
            opengl_bindings::{
                TEXTURE_MAG_FILTER, TEXTURE_MIN_FILTER, TEXTURE_WRAP_S,
                TEXTURE_WRAP_T, UNPACK_ALIGNMENT, UNSIGNED_BYTE,
            },
        };

        unsafe {
            gl.PixelStorei(UNPACK_ALIGNMENT, 1);
            gl.TexImage2D(
                target,
                0,
                ALPHA.into(),
                width.into(),
                height.into(),
                0,
                ALPHA.into(),
                UNSIGNED_BYTE,
                data.as_ptr().cast(),
            );
            gl.TexParameteri(target, TEXTURE_MIN_FILTER, LINEAR.into());
            gl.TexParameteri(target, TEXTURE_MAG_FILTER, LINEAR.into());
            gl.TexParameteri(target, TEXTURE_WRAP_S, CLAMP_TO_EDGE.into());
            gl.TexParameteri(target, TEXTURE_WRAP_T, CLAMP_TO_EDGE.into());
        }
        TextureSize {
            image_width: width.into(),
            image_height: height.into(),
            texture_width: width,
            texture_height: height,
            is_sdf: true,
        }
    }
}
