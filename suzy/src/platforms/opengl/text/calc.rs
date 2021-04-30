/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;
use std::collections::HashMap;

use super::{ChannelMask, FontFamilyDynamic, GlyphMetricsSource};

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

/// A font style for a block of rich text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FontStyle {
    /// Normal font style.
    Normal,
    /// Bold font style.
    Bold,
    /// Italic font style.
    Italic,
    /// Bold and italic font style.
    BoldItalic,
}

impl Default for FontStyle {
    fn default() -> Self {
        Self::Normal
    }
}

impl FontStyle {
    /// Convert the font style, applying bold (if not already present).
    ///
    /// ```
    /// # use suzy::platform::opengl::FontStyle;
    /// assert_eq!(FontStyle::Normal.bold(), FontStyle::Bold);
    /// assert_eq!(FontStyle::Bold.bold(), FontStyle::Bold);
    /// assert_eq!(FontStyle::Italic.bold(), FontStyle::BoldItalic);
    /// ```
    pub fn bold(self) -> Self {
        match self {
            Self::Normal => Self::Bold,
            Self::Italic => Self::BoldItalic,
            _ => self,
        }
    }

    /// Convert the font style, applying italic (if not already present).
    ///
    /// ```
    /// # use suzy::platform::opengl::FontStyle;
    /// assert_eq!(FontStyle::Normal.italic(), FontStyle::Italic);
    /// assert_eq!(FontStyle::Italic.italic(), FontStyle::Italic);
    /// assert_eq!(FontStyle::Bold.italic(), FontStyle::BoldItalic);
    /// ```
    pub fn italic(self) -> Self {
        match self {
            Self::Normal => Self::Italic,
            Self::Bold => Self::BoldItalic,
            _ => self,
        }
    }

    /// Convert the font style, removing bold (if present).
    ///
    /// ```
    /// # use suzy::platform::opengl::FontStyle;
    /// assert_eq!(FontStyle::Normal.unbold(), FontStyle::Normal);
    /// assert_eq!(FontStyle::Bold.unbold(), FontStyle::Normal);
    /// assert_eq!(FontStyle::BoldItalic.unbold(), FontStyle::Italic);
    /// ```
    pub fn unbold(self) -> Self {
        match self {
            Self::Bold => Self::Normal,
            Self::BoldItalic => Self::Italic,
            _ => self,
        }
    }

    /// Convert the font style, removing italic (if present).
    ///
    /// ```
    /// # use suzy::platform::opengl::FontStyle;
    /// assert_eq!(FontStyle::Normal.unitalic(), FontStyle::Normal);
    /// assert_eq!(FontStyle::Italic.unitalic(), FontStyle::Normal);
    /// assert_eq!(FontStyle::BoldItalic.unitalic(), FontStyle::Bold);
    /// ```
    pub fn unitalic(self) -> Self {
        match self {
            Self::Italic => Self::Normal,
            Self::BoldItalic => Self::Bold,
            _ => self,
        }
    }
}

/// An enum describing horizontal text alignment settings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextAlignment {
    /// Left-aligned
    Left,
    /// Center-aligned
    Center,
    /// Right-aligned
    Right,
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

pub enum RichTextCommand<'a> {
    Text(Cow<'a, str>),
    Bold,
    Italic,
    ResetBold,
    ResetItalic,
}

pub struct RichTextParser<'a> {
    text: &'a str,
}

impl<'a> RichTextParser<'a> {
    pub fn new(text: &'a str) -> Self {
        Self { text }
    }
}

impl<'a> Iterator for RichTextParser<'a> {
    type Item = RichTextCommand<'a>;
    fn next(&mut self) -> Option<RichTextCommand<'a>> {
        // TODO: can these be changed to `if let Some = text.strip_prefix` ?
        if self.text.is_empty() {
            None
        } else if self.text.starts_with("<b>") {
            self.text = &self.text[3..];
            Some(RichTextCommand::Bold)
        } else if self.text.starts_with("<i>") {
            self.text = &self.text[3..];
            Some(RichTextCommand::Italic)
        } else if self.text.starts_with("</b>") {
            self.text = &self.text[4..];
            Some(RichTextCommand::ResetBold)
        } else if self.text.starts_with("</i>") {
            self.text = &self.text[4..];
            Some(RichTextCommand::ResetItalic)
        } else {
            let next_cmd = ["<b>", "<i>", "</b>", "</i>"]
                .iter()
                .filter_map(|cmd| self.text.find(cmd))
                .min();
            let text = if let Some(index) = next_cmd {
                let (text, next) = self.text.split_at(index);
                self.text = next;
                text
            } else {
                std::mem::replace(&mut self.text, "")
            };
            let cow = if text.contains('&') {
                Cow::Owned(text.replace("&lt;", "<").replace("&amp;", "&"))
            } else {
                Cow::Borrowed(text)
            };
            Some(RichTextCommand::Text(cow))
        }
    }
}

pub(super) struct FontCharCalc<'a> {
    font_family: &'a FontFamilyDynamic<'a>,
    settings: TextLayoutSettings,
    current_style: FontStyle,
    x_offset: f32,
    y_offset: f32,
    bufs: HashMap<ChannelMask, Vec<f32>>,
    commited: HashMap<ChannelMask, usize>,
}

impl<'a> FontCharCalc<'a> {
    pub fn new(
        font_family: &'a FontFamilyDynamic,
        settings: TextLayoutSettings,
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
        // TODO: use extend or something, rather than all these pushes.
        buf.reserve(4 * 6);

        buf.push(left_pos);
        buf.push(bottom_pos);
        buf.push(left_uv);
        buf.push(bottom_uv);

        buf.push(right_pos);
        buf.push(top_pos);
        buf.push(right_uv);
        buf.push(top_uv);

        buf.push(left_pos);
        buf.push(top_pos);
        buf.push(left_uv);
        buf.push(top_uv);

        buf.push(left_pos);
        buf.push(bottom_pos);
        buf.push(left_uv);
        buf.push(bottom_uv);

        buf.push(right_pos);
        buf.push(bottom_pos);
        buf.push(right_uv);
        buf.push(bottom_uv);

        buf.push(right_pos);
        buf.push(top_pos);
        buf.push(right_uv);
        buf.push(top_uv);
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
        self.x_offset = x_offset;
    }

    pub fn push_whitespace(&mut self, white_char: char) {
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
