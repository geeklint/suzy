/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use crate::text;

use super::font;

#[derive(Debug, Default)]
pub(super) struct FontCharCalc {
    pub layout: text::Layout,
    pub cursor: Cursor,
    pub first_line_line: f32,
    pub last_line_line: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Cursor {
    pub font_size: f32,
    pub x: f32,
    pub y: f32,
    pub current_line_height: f32,
}

pub(super) struct CalcParams<'a, F> {
    pub font: &'a font::FontData,
    pub handle_glyph: F,
}

impl FontCharCalc {
    pub fn reset(&mut self) {
        self.cursor = Cursor {
            font_size: 0.0,
            x: 0.0,
            y: 0.0,
            current_line_height: f32::NAN,
        };
        self.first_line_line = f32::NAN;
        self.last_line_line = f32::NAN;
    }

    pub fn align_line<'a>(
        &mut self,
        verts: impl Iterator<Item = &'a mut [f32; 2]>,
    ) {
        let horiz_shift = match self.layout.alignment {
            text::Alignment::Left => 0.0,
            text::Alignment::Center => -self.cursor.x / 2.0,
            text::Alignment::Right => -self.cursor.x,
        };
        let vert_shift = self.cursor.current_line_height.max(0.0);
        for [x, y] in verts {
            *x += horiz_shift;
            *y -= vert_shift;
        }
    }

    pub fn reset_line(&mut self) {
        if self.cursor.current_line_height.is_nan() {
            self.cursor.current_line_height = 0.0;
            self.first_line_line = self.last_line_line;
        }
        self.cursor.x = 0.0;
        self.cursor.y -= self.cursor.current_line_height;
        self.last_line_line -= self.cursor.current_line_height;
        self.cursor.current_line_height = 0.0;
    }

    pub fn align_block<'a>(
        &mut self,
        verts: impl Iterator<Item = &'a mut [f32; 2]>,
    ) {
        let origin_line = match self.layout.flow {
            text::Flow::Down => self.first_line_line,
            text::Flow::Up => self.last_line_line,
            text::Flow::Out => {
                (self.first_line_line + self.last_line_line) / 2.0
            }
        };
        let vert_shift = self.layout.origin_y - origin_line;
        let horiz_shift = self.layout.origin_x;
        for [x, y] in verts {
            *x += horiz_shift;
            *y += vert_shift;
        }
    }

    pub fn push_span(
        &mut self,
        mut params: CalcParams<'_, impl FnMut(GlyphMetrics)>,
        text: &str,
    ) -> (usize, bool) {
        let mut remaining = text;
        let mut consumed = 0;
        let mut last_ch = None;
        let mut line_break = false;
        let line = match self.layout.line {
            text::Line::Ascent => params.font.ascent,
            text::Line::Descent => params.font.descent,
            text::Line::Baseline => 0.0,
            text::Line::BetweenBaseAndCap => params.font.capline / 2.0,
        };
        let line = line * self.cursor.font_size + self.cursor.y;
        if self.cursor.x == 0.0 {
            self.last_line_line = f32::NAN;
        }
        self.last_line_line = if self.layout.line == text::Line::Descent {
            self.last_line_line.min(line)
        } else {
            self.last_line_line.max(line)
        };
        let line_height = params.font.line_spacing * self.cursor.font_size;
        // intentionally propgate NaNs
        if line_height > self.cursor.current_line_height {
            self.cursor.current_line_height = line_height;
        }
        while !remaining.is_empty() {
            if self.cursor.x > self.layout.wrap_width {
                break;
            }
            let word_end = remaining.find(is_breaking_space);
            match word_end {
                None => {
                    let (_, cons) = self.push_word(&mut params, remaining);
                    consumed += cons;
                    break;
                }
                Some(0) => {
                    let mut iter = remaining.char_indices();
                    let (_, ch) = iter.next().expect(concat!(
                        "remaining text was not empty,",
                        "but str::chars returned no items"
                    ));
                    consumed +=
                        iter.next().map_or(remaining.len(), |(i, _)| i);
                    if is_line_break(ch) {
                        line_break = true;
                        break;
                    }
                    Self::push_whitespace(
                        &mut self.cursor,
                        &params,
                        last_ch,
                        ch,
                    );
                    last_ch = Some(ch);
                }
                Some(index) => {
                    let word = &remaining[..index];
                    let (lch, cons) = self.push_word(&mut params, word);
                    last_ch = lch;
                    consumed += cons;
                    if cons < word.len() {
                        break;
                    }
                }
            }
            remaining = &text[consumed..];
        }
        (consumed, line_break || consumed < remaining.len())
    }

    pub fn push_word(
        &mut self,
        params: &mut CalcParams<'_, impl FnMut(GlyphMetrics)>,
        word: &str,
    ) -> (Option<char>, usize) {
        if self.cursor.x == 0.0 {
            return self.push_word_splitwrap(params, word);
        }
        let mut glyphs = Vec::new();
        let mut last_ch = None;
        let mut cursor = self.cursor;
        for ch in word.chars() {
            if let Some(glyph) = params.font.glyph(ch) {
                let kerning = last_ch
                    .and_then(|left| params.font.kerning(left, ch))
                    .unwrap_or(0.0);
                let kerning = kerning * cursor.font_size;
                let advance = glyph.advance * cursor.font_size;
                if cursor.x + kerning + advance > self.layout.wrap_width {
                    return (None, 0);
                }
                cursor.x += kerning;
                glyphs.push(Self::position_char(&mut cursor, *glyph));
                cursor.x += advance;
            } else if ch.is_whitespace() {
                Self::push_whitespace(&mut cursor, params, last_ch, ch);
            }
            last_ch = Some(ch);
        }
        for glyph in glyphs {
            (params.handle_glyph)(glyph);
        }
        self.cursor = cursor;
        (last_ch, word.len())
    }

    pub fn push_whitespace(
        cursor: &mut Cursor,
        params: &CalcParams<'_, impl FnMut(GlyphMetrics)>,
        last_ch: Option<char>,
        white_char: char,
    ) {
        let advance = if let Some(glyph) = params.font.glyph(white_char) {
            glyph.advance
        } else {
            match white_char {
                ' ' => 0.25,
                '\u{2002}' => 0.5,
                '\u{2003}' => 1.0,
                '\u{2004}' => 1.0 / 3.0,
                '\u{2005}' => 0.5,
                '\u{2006}' => 1.0 / 6.0,
                '\u{2009}' => 1.0 / 5.0,
                _ => 0.0,
            }
        };
        let kerning = last_ch
            .and_then(|left| params.font.kerning(left, white_char))
            .unwrap_or(0.0);
        cursor.x += cursor.font_size * (kerning + advance);
    }

    fn push_word_splitwrap(
        &mut self,
        params: &mut CalcParams<'_, impl FnMut(GlyphMetrics)>,
        word: &str,
    ) -> (Option<char>, usize) {
        let mut last_ch = None;
        for (i, ch) in word.char_indices() {
            if let Some(glyph) = params.font.glyph(ch) {
                let kerning = last_ch
                    .and_then(|left| params.font.kerning(left, ch))
                    .unwrap_or(0.0);
                let kerning = kerning * self.cursor.font_size;
                let advance = glyph.advance * self.cursor.font_size;
                if self.cursor.x + kerning + advance > self.layout.wrap_width {
                    return (None, i);
                }
                self.cursor.x += kerning;
                (params.handle_glyph)(Self::position_char(
                    &mut self.cursor,
                    *glyph,
                ));
                self.cursor.x += advance;
            } else if ch.is_whitespace() {
                Self::push_whitespace(&mut self.cursor, params, last_ch, ch);
            }
            last_ch = Some(ch);
        }
        (last_ch, word.len())
    }

    #[must_use]
    fn position_char(cursor: &mut Cursor, glyph: font::Glyph) -> GlyphMetrics {
        let font::Glyph {
            bb_left,
            bb_right,
            bb_bottom,
            bb_top,
            tex_left,
            tex_right,
            tex_bottom,
            tex_top,
            ..
        } = glyph;
        GlyphMetrics {
            bb_left: bb_left * cursor.font_size + cursor.x,
            bb_right: bb_right * cursor.font_size + cursor.x,
            bb_bottom: bb_bottom * cursor.font_size + cursor.y,
            bb_top: bb_top * cursor.font_size + cursor.y,
            tex_left,
            tex_right,
            tex_bottom,
            tex_top,
        }
    }
}

#[must_use]
fn is_line_break(ch: char) -> bool {
    matches!(ch, '\n')
}

#[must_use]
fn is_breaking_space(ch: char) -> bool {
    match ch {
        '\u{00a0}' | '\u{2007}' | '\u{202f}' => false,
        _ if ch.is_whitespace() => true,
        _ => false,
    }
}

#[derive(Clone, Copy, Debug)]
pub struct GlyphMetrics {
    pub bb_left: f32,
    pub bb_right: f32,
    pub bb_bottom: f32,
    pub bb_top: f32,
    pub tex_left: u16,
    pub tex_right: u16,
    pub tex_bottom: u16,
    pub tex_top: u16,
}
