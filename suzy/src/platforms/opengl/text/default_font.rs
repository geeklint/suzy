/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2023 Violet Leonard */

use std::{cell::OnceCell, rc::Rc};

use crate::platforms::opengl::{self, PopulateTexture, Texture};

use super::font;

thread_local! {
    static DEFAULT_FONT: Rc<font::Font> = build();
}

pub fn default_font() -> Rc<font::Font> {
    DEFAULT_FONT.with(Rc::clone)
}

fn build() -> Rc<font::Font> {
    let texture = Texture::new(Rc::new(DefaultFontAtlasPopulator));
    let regular_data = font::FontData {
        texture,
        padding_ratio: suzy_default_font::regular::PADDING_RATIO,
        glyphs: suzy_default_font::regular::GLYPHS
            .iter()
            .map(|glyph| font::Glyph {
                ch: glyph.ch,
                advance: glyph.advance,
                bb_left: glyph.bb_left,
                bb_right: glyph.bb_right,
                bb_bottom: glyph.bb_bottom,
                bb_top: glyph.bb_top,
                tex_left: glyph.tex_left,
                tex_right: glyph.tex_right,
                tex_bottom: glyph.tex_bottom,
                tex_top: glyph.tex_top,
            })
            .collect(),
        kerning: Box::new([]),
        line_spacing: suzy_default_font::regular::LINE_SPACING,
        ascent: suzy_default_font::regular::ASCENT,
        capline: suzy_default_font::regular::CAPLINE,
        descent: suzy_default_font::regular::DESCENT,
    };
    Rc::new(font::Font {
        data: regular_data,
        bold: OnceCell::new(),
        italic: OnceCell::new(),
    })
}

struct DefaultFontAtlasPopulator;

impl PopulateTexture for DefaultFontAtlasPopulator {
    fn populate(
        &self,
        gl: &opengl::OpenGlBindings,
        target: opengl::opengl_bindings::types::GLenum,
    ) -> Result<opengl::TextureSize, String> {
        Ok(font::Font::populate_font_atlas(
            gl,
            target,
            suzy_default_font::TEXTURE_WIDTH,
            suzy_default_font::TEXTURE_HEIGHT,
            suzy_default_font::TEXTURE_DATA,
        ))
    }

    fn texture_key(&self) -> &[u8] {
        // random, very unlikely to collide with anything else
        b"\xf6>\xed=\x9d\x80\xc4N\xb9\x14\n\x03Q\x8f&\xea"
    }
}
