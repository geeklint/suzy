/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use std::{
    io::{self, Write},
    path::Path,
};

pub use blurry::{ascii, hexdigits, latin1, latin1_french};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u16)]
#[non_exhaustive]
pub enum TextureDim {
    V16 = 16,
    V32 = 32,
    V64 = 64,
    V128 = 128,
    V256 = 256,
    V512 = 512,
    V1024 = 1024,
    V2048 = 2048,
    V4096 = 4096,
    V8192 = 8192,
}

#[derive(Debug)]
struct FontData {
    name: String,
    data: Vec<u8>,
}

#[derive(Debug)]
pub struct FontAtlas<I> {
    builder: blurry::FontAssetBuilder,
    font_data: Vec<FontData>,
    spec: I,
    padding_ratio: f32,
}

impl FontAtlas<std::iter::Empty<FontSpec>> {
    pub fn with_texture_size(width: TextureDim, height: TextureDim) -> Self {
        let initial_padding = 0.1;
        Self {
            builder: blurry::FontAssetBuilder::with_texture_size(
                width as u16,
                height as u16,
            )
            .with_padding_ratio(initial_padding),
            font_data: Vec::new(),
            spec: std::iter::empty(),
            padding_ratio: initial_padding,
        }
    }
}

impl<I> FontAtlas<I> {
    pub fn with_padding_ratio(self, padding: f32) -> Self {
        Self {
            builder: self.builder.with_padding_ratio(padding),
            font_data: self.font_data,
            spec: self.spec,
            padding_ratio: padding,
        }
    }

    pub fn add_font(
        self,
        out_name: String,
        font: impl AsRef<Path>,
        chars: impl Clone + Iterator<Item = char>,
    ) -> Result<FontAtlas<impl Clone + Iterator<Item = FontSpec>>, io::Error>
    where
        I: Clone + Iterator<Item = FontSpec>,
    {
        let Self {
            builder,
            mut font_data,
            spec,
            padding_ratio,
        } = self;
        let font_index = font_data.len();
        let data = std::fs::read(font)?;
        font_data.push(FontData {
            name: out_name,
            data,
        });
        Ok(FontAtlas {
            builder,
            font_data,
            spec: spec.chain(chars.map(move |ch| FontSpec { font_index, ch })),
            padding_ratio,
        })
    }

    pub fn write_module(self, path: impl AsRef<Path>) -> Result<(), Error>
    where
        I: Clone + Iterator<Item = FontSpec>,
    {
        let Self {
            builder,
            font_data,
            spec,
            padding_ratio,
        } = self;
        let faces = font_data
            .iter()
            .map(|data| blurry::ttf_parser::Face::parse(&data.data, 0))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| Error::FontParseError)?;
        let asset = builder
            .build(spec.map(|spec| blurry::GlyphRequest {
                user_data: spec.font_index,
                face: &faces[spec.font_index],
                codepoint: spec.ch,
            }))
            .map_err(|err| match err {
                blurry::Error::MissingGlyph(ch) => Error::MissingGlyph(ch),
                blurry::Error::PackingAtlasFailed => Error::PackingAtlasFailed,
                _ => Error::FontParseError,
            })?;
        let mut texture_path = path.as_ref().as_os_str().to_os_string();
        texture_path.push(".texture");
        std::fs::write(texture_path, asset.data)?;
        let mut mod_file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(path)?;
        write!(mod_file, "pub const TEXTURE_WIDTH: u16 = {};", asset.width)?;
        write!(
            mod_file,
            "pub const TEXTURE_HEIGHT: u16 = {};",
            asset.height
        )?;
        for (index, font) in font_data.iter().enumerate() {
            let font_face = &faces[index];
            let height = f32::from(font_face.height());
            let rel_value = |val| f32::from(val) / height;
            let line_spacing = rel_value(font_face.line_gap());
            let ascent = rel_value(font_face.ascender());
            let mut capline = font_face.capital_height().map(rel_value);
            let mut x_height = font_face.x_height().map(rel_value);
            let descent = rel_value(font_face.descender());
            write!(mod_file, "pub mod {} {{", font.name)?;
            write!(
                mod_file,
                "
                use super::Glyph;
                pub const PADDING_RATIO: f32 = {padding_ratio}f32;
                pub const LINE_SPACING: f32 = {line_spacing}f32;
                pub const ASCENT: f32 = {ascent}f32;
                pub const DESCENT: f32 = {descent}f32;
                ",
            )?;
            let mut glyphs: Vec<_> = asset
                .metadata
                .iter()
                .filter(|glyph| glyph.user_data == index)
                .collect();
            glyphs.sort_by_key(|glyph| glyph.codepoint);
            write!(mod_file, "pub static GLYPHS: &[Glyph] = &[")?;
            for glyph in glyphs {
                let blurry::Glyph {
                    codepoint,
                    left,
                    right,
                    bottom,
                    top,
                    tex_left,
                    tex_right,
                    tex_top,
                    tex_bottom,
                    ..
                } = *glyph;
                if x_height.is_none() && codepoint == 'x' {
                    x_height = Some(top);
                }
                if capline.is_none() && codepoint == 'I' {
                    capline = Some(top);
                }
                let ch = codepoint.escape_unicode();
                let advance = font_face
                    .glyph_index(codepoint)
                    .and_then(|id| font_face.glyph_hor_advance(id))
                    .ok_or(Error::MissingGlyph(codepoint))?;
                let advance = f32::from(advance) / height;
                let tex_left =
                    (tex_left * f32::from(asset.width)).round() as u16;
                let tex_right =
                    (tex_right * f32::from(asset.width)).round() as u16;
                let tex_bottom =
                    (tex_bottom * f32::from(asset.height)).round() as u16;
                let tex_top =
                    (tex_top * f32::from(asset.height)).round() as u16;
                write!(
                    mod_file,
                    "
                    Glyph {{
                        ch:'{ch}',
                        advance:{advance}f32,
                        bb_left:{left}f32,
                        bb_right:{right}f32,
                        bb_bottom:{bottom}f32,
                        bb_top:{top}f32,
                        tex_left:{tex_left},
                        tex_right:{tex_right},
                        tex_bottom:{tex_bottom},
                        tex_top:{tex_top},
                    }},"
                )?;
            }
            write!(mod_file, "];")?;
            let capline = capline.unwrap_or(0.7);
            let x_height = x_height.unwrap_or(0.5);
            write!(mod_file, "pub const CAPLINE: f32 = {capline}f32;")?;
            write!(mod_file, "pub const X_HEIGHT: f32 = {x_height}f32;")?;
            write!(mod_file, "}}")?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    FontParseError,
    MissingGlyph(char),
    PackingAtlasFailed,
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

pub struct FontSpec {
    font_index: usize,
    ch: char,
}
