/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use std::{cell::OnceCell, ops::Deref, rc::Rc};

use crate::platforms::opengl;
use opengl::texture::Texture;

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
pub struct Kerning {
    pub left: char,
    pub right: char,
    pub kerning: f32,
}

#[derive(Debug)]
pub struct FontData {
    pub texture: Texture,
    pub padding_ratio: f32,
    pub glyphs: Box<[Glyph]>,
    pub kerning: Box<[Kerning]>,
    pub line_spacing: f32,
    pub ascent: f32,
    pub capline: f32,
    pub descent: f32,
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

#[derive(Debug)]
pub struct StaticFont {
    data: FontData,
    bold: Option<&'static StaticFont>,
    italic: Option<&'static StaticFont>,
}

#[derive(Debug)]
pub struct RcFont {
    data: FontData,
    bold: OnceCell<Rc<RcFont>>,
    italic: OnceCell<Rc<RcFont>>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum FontRef<'a> {
    Static(&'static StaticFont),
    Rc(&'a RcFont),
}

#[derive(Clone, Debug)]
pub enum Font {
    Static(&'static StaticFont),
    Rc(Rc<RcFont>),
}

impl Font {
    pub(crate) fn as_ref(&self) -> FontRef<'_> {
        match self {
            Font::Static(font) => FontRef::Static(font),
            Font::Rc(font) => FontRef::Rc(&font),
        }
    }
}

impl<'a> FontRef<'a> {
    pub fn data(&self) -> &FontData {
        match self {
            Self::Static(font) => &font.data,
            Self::Rc(font) => &font.data,
        }
    }

    pub fn bold(&self) -> FontRef<'_> {
        match self {
            FontRef::Static(font) => Self::Static(font.bold.unwrap_or(font)),
            FontRef::Rc(font) => {
                Self::Rc(font.bold.get().map(Rc::deref).unwrap_or(font))
            }
        }
    }

    pub fn italic(&self) -> FontRef<'_> {
        match self {
            FontRef::Static(font) => Self::Static(font.italic.unwrap_or(font)),
            FontRef::Rc(font) => {
                Self::Rc(font.italic.get().map(Rc::deref).unwrap_or(font))
            }
        }
    }
}
