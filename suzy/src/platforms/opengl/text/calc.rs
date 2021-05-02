/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;

use super::{ChannelMask, FontFamilyDynamic, GlyphMetricsSource};
use crate::text::*;

struct GlyphMetrics {
    _ch: char,
    uv_x: f32,
    uv_y: f32,
    uv_width: f32,
    uv_height: f32,
    bb_min_x: f32,
    bb_max_x: f32,
    bb_min_y: f32,
    bb_max_y: f32,
    advance_width: f32,
}

fn conv_glyph_metrics(source: GlyphMetricsSource) -> GlyphMetrics {
    GlyphMetrics {
        _ch: source.0,
        uv_x: source.1,
        uv_y: source.2,
        uv_width: source.3,
        uv_height: source.4,
        bb_min_x: source.5,
        bb_max_x: source.6,
        bb_min_y: source.7,
        bb_max_y: source.8,
        advance_width: source.9,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CharLocationError {
    CalcError,
    Outside { clamped: usize },
}

pub trait RecordCharLocation: Default {
    fn record_char(&mut self, left: f32, right: f32, top: f32, bottom: f32);
    fn append(&mut self, other: Self);
}

impl RecordCharLocation for () {
    fn record_char(&mut self, _l: f32, _r: f32, _t: f32, _b: f32) {}
    fn append(&mut self, _other: Self) {}
}

#[derive(Clone, Copy, Debug)]
struct CharRect {
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
}

pub struct CharLocationRecorder {
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
    data: Vec<CharRect>,
}

impl CharLocationRecorder {
    pub fn char_at(&self, x: f32, y: f32) -> Result<usize, CharLocationError> {
        use std::cmp::Ordering;

        let mut err = false;
        let inside = (self.min_x..=self.max_x).contains(&x)
            && (self.min_y..=self.max_y).contains(&y);
        let x = x.clamp(self.min_x, self.max_x);
        let y = y.clamp(self.min_y, self.max_y);
        let search_res = self.data.binary_search_by(|rect| {
            if rect.top < y {
                Ordering::Greater
            } else if rect.bottom > y {
                Ordering::Less
            } else if rect.left > x {
                Ordering::Greater
            } else if rect.right < x {
                Ordering::Less
            } else if [x, y, rect.left, rect.top, rect.bottom, rect.right]
                .iter()
                .any(|v| v.is_nan())
            {
                err = true;
                std::cmp::Ordering::Equal
            } else {
                std::cmp::Ordering::Equal
            }
        });
        if err {
            return Err(CharLocationError::CalcError);
        }
        let index = match search_res {
            Ok(index) => index,
            Err(index) => index.saturating_sub(1),
        };
        if inside {
            Ok(index)
        } else {
            Err(CharLocationError::Outside { clamped: index })
        }
    }
}

impl Default for CharLocationRecorder {
    fn default() -> Self {
        Self {
            min_x: f32::INFINITY,
            min_y: f32::INFINITY,
            max_x: f32::NEG_INFINITY,
            max_y: f32::NEG_INFINITY,
            data: Vec::new(),
        }
    }
}

impl RecordCharLocation for CharLocationRecorder {
    fn record_char(&mut self, left: f32, right: f32, top: f32, bottom: f32) {
        self.min_x = self.min_x.min(left);
        self.min_y = self.min_y.min(bottom);
        self.max_x = self.max_x.max(right);
        self.max_y = self.max_y.max(top);
        self.data.push(CharRect {
            left,
            right,
            top,
            bottom,
        })
    }

    fn append(&mut self, mut other: Self) {
        self.min_x = self.min_x.min(other.min_x);
        self.min_y = self.min_y.min(other.min_y);
        self.max_x = self.max_x.max(other.max_x);
        self.max_y = self.max_y.max(other.max_y);
        self.data.append(&mut other.data);
    }
}

/// A type which contains settings which effect the vertex generation
/// of a text object.
#[derive(Clone, Copy, Debug)]
pub struct TextLayoutSettings {
    font_size: f32,
    wrap_width: f32,
    alignment: TextAlignment,
    tab_stop: f32,
    y_offset: f32,
}

impl Default for TextLayoutSettings {
    fn default() -> Self {
        Self {
            font_size: 24.0,
            wrap_width: f32::INFINITY,
            alignment: TextAlignment::Left,
            tab_stop: 48.0,
            y_offset: 0.0,
        }
    }
}

impl TextLayoutSettings {
    /// Set the font size.
    #[must_use]
    pub fn font_size(self, font_size: f32) -> Self {
        Self { font_size, ..self }
    }

    /// Set the width at which text will wrap.
    #[must_use]
    pub fn wrap_width(self, wrap_width: f32) -> Self {
        Self { wrap_width, ..self }
    }

    /// Set the alignment of the text.
    #[must_use]
    pub fn alignment(self, alignment: TextAlignment) -> Self {
        Self { alignment, ..self }
    }

    /// Set the tab-stop of the text.
    #[must_use]
    pub fn tab_stop(self, tab_stop: f32) -> Self {
        Self { tab_stop, ..self }
    }

    /// Set the y-offset of the text.
    #[must_use]
    pub fn y_offset(self, y_offset: f32) -> Self {
        Self { y_offset, ..self }
    }
}

pub(super) struct FontCharCalc<'a, T> {
    font_family: &'a FontFamilyDynamic<'a>,
    settings: TextLayoutSettings,
    current_style: FontStyle,
    x_offset: f32,
    y_offset: f32,
    bufs: HashMap<ChannelMask, Vec<f32>>,
    commited: HashMap<ChannelMask, usize>,
    char_locs: &'a mut T,
}

impl<'a, T> FontCharCalc<'a, T> {
    pub fn new(
        font_family: &'a FontFamilyDynamic,
        settings: TextLayoutSettings,
        char_locs: &'a mut T,
    ) -> Self {
        let mut bufs: HashMap<_, _> = font_family
            .channel_masks
            .iter()
            .copied()
            .map(|mask| (mask, vec![]))
            .collect();
        bufs.insert((0, 0, 0, 0), vec![]);
        let commited = bufs.keys().map(|mask| (*mask, 0)).collect();
        FontCharCalc {
            font_family,
            y_offset: settings.y_offset,
            settings,
            current_style: FontStyle::Normal,
            x_offset: 0.0,
            bufs,
            commited,
            char_locs,
        }
    }

    pub(super) fn merge_verts(
        &mut self,
        vertex_buffer: &mut Vec<f32>,
        channels: &mut HashMap<ChannelMask, std::ops::Range<usize>>,
    ) {
        self.commit_line();
        let mut vertex_offset = 0;
        for (mask, buf) in self.bufs.iter() {
            let next_vo = vertex_offset + buf.len() / 4;
            vertex_buffer.extend(buf);
            channels.insert(*mask, vertex_offset..next_vo);
            vertex_offset = next_vo;
        }
    }

    fn metrics(&self, ch: char) -> Option<GlyphMetrics> {
        let list = self.font_family.best_font_source(self.current_style).1;
        list.binary_search_by_key(&ch, |coord| coord.0)
            .ok()
            .map(|index| conv_glyph_metrics(list[index]))
    }

    fn kerning(&self, ch: char, next: char) -> f32 {
        let list = self.font_family.best_font_source(self.current_style).2;
        list.binary_search_by_key(&(ch, next), |item| (item.0, item.1))
            .ok()
            .map(|index| list[index].2)
            .unwrap_or(0.0)
    }

    fn commit_line(&mut self) {
        let remaining_in_line = self.settings.wrap_width - self.x_offset;
        let shift = match self.settings.alignment {
            TextAlignment::Left => 0.0,
            TextAlignment::Center => remaining_in_line / 2.0,
            TextAlignment::Right => remaining_in_line,
        };
        for (mask, buf) in self.bufs.iter_mut() {
            let commit = self.commited.get_mut(&mask).unwrap();
            for i in (*commit..buf.len()).step_by(4) {
                buf[i] += shift;
            }
            *commit = buf.len();
        }
    }

    pub fn push_newline(&mut self) {
        self.commit_line();
        self.x_offset = 0.0;
        self.y_offset -= self.settings.font_size;
    }

    fn populate_char(&mut self, metrics: GlyphMetrics) {
        let mask = self.font_family.channel_mask(self.current_style);
        Self::populate_vertices(
            self.settings.font_size,
            self.bufs.get_mut(&mask).unwrap(),
            self.x_offset,
            self.y_offset,
            metrics,
        );
    }

    fn populate_vertices(
        font_size: f32,
        buf: &mut Vec<f32>,
        x_offset: f32,
        y_offset: f32,
        metrics: GlyphMetrics,
    ) {
        let left_pos = x_offset + metrics.bb_min_x * font_size;
        let left_uv = metrics.uv_x;
        let right_pos = x_offset + metrics.bb_max_x * font_size;
        let right_uv = metrics.uv_x + metrics.uv_width;
        let top_pos = y_offset + metrics.bb_max_y * font_size;
        let top_uv = metrics.uv_y;
        let bottom_pos = y_offset + metrics.bb_min_y * font_size;
        let bottom_uv = metrics.uv_y + metrics.uv_height;
        #[rustfmt::skip]
        buf.extend(&[
            left_pos,
            bottom_pos,
            left_uv,
            bottom_uv,

            right_pos,
            top_pos,
            right_uv,
            top_uv,

            left_pos,
            top_pos,
            left_uv,
            top_uv,

            left_pos,
            bottom_pos,
            left_uv,
            bottom_uv,

            right_pos,
            bottom_pos,
            right_uv,
            bottom_uv,

            right_pos,
            top_pos,
            right_uv,
            top_uv,
        ]);
    }
}

impl<'a, T: RecordCharLocation> FontCharCalc<'a, T> {
    fn record_char_loc(&mut self, advance: f32) {
        self.char_locs.record_char(
            self.x_offset,
            self.x_offset + advance,
            self.y_offset + self.settings.font_size,
            self.y_offset,
        );
    }

    fn push_word_splitwrap(&mut self, word: &str) {
        let mut iter = word.chars().peekable();
        while let Some(ch) = iter.next() {
            if let Some(metrics) = self.metrics(ch) {
                let kerning = iter
                    .peek()
                    .copied()
                    .map_or(0.0, |nch| self.kerning(ch, nch));
                let advance = metrics.advance_width + kerning;
                let advance = advance * self.settings.font_size;
                if self.x_offset + advance > self.settings.wrap_width {
                    self.push_newline();
                }
                self.record_char_loc(advance);
                self.populate_char(metrics);
                self.x_offset += advance;
            }
        }
    }

    pub fn push_word(&mut self, word: &str) {
        if self.x_offset == 0.0 {
            self.push_word_splitwrap(word);
            return;
        }
        let mut x_offset = self.x_offset;
        let mut verts = Vec::new();
        let mut char_locs = T::default();
        let mut iter = word.chars().peekable();
        while let Some(ch) = iter.next() {
            if let Some(metrics) = self.metrics(ch) {
                let kerning = iter
                    .peek()
                    .copied()
                    .map_or(0.0, |nch| self.kerning(ch, nch));
                let advance = metrics.advance_width + kerning;
                let advance = advance * self.settings.font_size;
                if x_offset + advance > self.settings.wrap_width {
                    self.push_newline();
                    self.push_word_splitwrap(word);
                    return;
                }
                char_locs.record_char(
                    x_offset,
                    x_offset + advance,
                    self.y_offset + self.settings.font_size,
                    self.y_offset,
                );
                Self::populate_vertices(
                    self.settings.font_size,
                    &mut verts,
                    x_offset,
                    self.y_offset,
                    metrics,
                );
                x_offset += advance;
            }
        }
        let mask = self.font_family.channel_mask(self.current_style);
        self.bufs.get_mut(&mask).unwrap().append(&mut verts);
        self.char_locs.append(char_locs);
        self.x_offset = x_offset;
    }

    pub fn push_whitespace(&mut self, white_char: char) {
        self.record_char_loc(f32::EPSILON);
        if let Some(metrics) = self.metrics(white_char) {
            let advance = metrics.advance_width;
            let advance = advance * self.settings.font_size;
            if self.x_offset + advance > self.settings.wrap_width {
                self.push_newline();
            }
            self.populate_char(metrics);
            self.x_offset += advance;
            return;
        }
        match white_char {
            '\t' => {
                let ntabs = self.x_offset.div_euclid(self.settings.tab_stop);
                self.x_offset = (ntabs + 1.0) * self.settings.tab_stop;
            }
            '\n' => {
                self.push_newline();
            }
            ' ' => {
                self.x_offset += self.settings.font_size * 0.25;
            }
            _ => (),
        }
    }

    pub fn push_str(&mut self, text: &str) {
        let mut remaining = text;
        while !remaining.is_empty() {
            let word_end = remaining.find(char::is_whitespace);
            match word_end {
                None => {
                    self.push_word(remaining);
                    remaining = "";
                }
                Some(0) => {
                    let mut iter = remaining.chars();
                    self.push_whitespace(iter.next().expect(concat!(
                        "remaining text was not empty,",
                        "but str::chars returned no items"
                    )));
                    remaining = iter.as_str();
                }
                Some(index) => {
                    let (word, next) = remaining.split_at(index);
                    self.push_word(word);
                    remaining = next;
                }
            }
        }
    }

    pub fn push(&mut self, cmd: RichTextCommand) {
        match cmd {
            RichTextCommand::Text(text) => self.push_str(&text),
            RichTextCommand::Bold => {
                self.current_style = self.current_style.bold();
            }
            RichTextCommand::Italic => {
                self.current_style = self.current_style.italic();
            }
            RichTextCommand::ResetBold => {
                self.current_style = self.current_style.unbold();
            }
            RichTextCommand::ResetItalic => {
                self.current_style = self.current_style.unitalic();
            }
        }
    }
}
