use std::collections::HashMap;
use std::borrow::Cow;

use super::{
    FontFamilyDynamic,
    ChannelMask,
    GlyphMetricsSource,
};


struct GlyphMetrics {
    ch: char,
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
        ch: source.0,
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

#[derive(Clone, Copy, Debug)]
pub enum FontStyle {
    Normal, Bold, Italic, BoldItalic
}

impl Default for FontStyle {
    fn default() -> Self {
        Self::Normal
    }
}

impl FontStyle {
    pub fn bold(self) -> Self {
        match self {
            Self::Normal => Self::Bold,
            Self::Italic => Self::BoldItalic,
            _ => self,
        }
    }

    pub fn italic(self) -> Self {
        match self {
            Self::Normal => Self::Italic,
            Self::Bold => Self::BoldItalic,
            _ => self,
        }
    }

    pub fn unbold(self) -> Self {
        match self {
            Self::Bold => Self::Normal,
            Self::BoldItalic => Self::Italic,
            _ => self,
        }
    }

    pub fn unitalic(self) -> Self {
        match self {
            Self::Italic => Self::Normal,
            Self::BoldItalic => Self::Bold,
            _ => self,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum TextAlignment {
    Left, Center, Right
}

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
    pub fn font_size(self, font_size: f32) -> Self {
        Self { font_size, ..self }
    }

    pub fn wrap_width(self, wrap_width: f32) -> Self {
        Self { wrap_width, ..self }
    }

    pub fn alignment(self, alignment: TextAlignment) -> Self {
        Self { alignment, ..self }
    }

    pub fn tab_stop(self, tab_stop: f32) -> Self {
        Self { tab_stop, ..self }
    }

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
        if self.text.len() == 0 {
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
            let next_cmd = ["<b>", "<i>", "</b>", "</i>"].iter()
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
    pub fn new(font_family: &'a FontFamilyDynamic, settings: TextLayoutSettings)
        -> Self
    {
        let mut bufs: HashMap<_,_> = font_family.channel_masks.iter()
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
        &self,
        vertex_buffer: &mut Vec<f32>,
        channels: &mut HashMap<ChannelMask, std::ops::Range<usize>>,
    ) {
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

    pub fn push_newline(&mut self) {
        let remaining_in_line = self.settings.wrap_width - self.y_offset;
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
        let left = (
            x_offset + metrics.bb_min_x * font_size,
            metrics.uv_x,
        );
        let right = (
            x_offset + metrics.bb_max_x * font_size,
            metrics.uv_x + metrics.uv_width,
        );
        let top = (
            y_offset + metrics.bb_max_y * font_size,
            metrics.uv_y,
        );
        let bottom = (
            y_offset + metrics.bb_min_y * font_size,
            metrics.uv_y + metrics.uv_height,
        );
        buf.reserve(4 * 6);

        buf.push(left.0);
        buf.push(bottom.0);
        buf.push(left.1);
        buf.push(bottom.1);

        buf.push(right.0);
        buf.push(top.0);
        buf.push(right.1);
        buf.push(top.1);

        buf.push(left.0);
        buf.push(top.0);
        buf.push(left.1);
        buf.push(top.1);

        buf.push(left.0);
        buf.push(bottom.0);
        buf.push(left.1);
        buf.push(bottom.1);

        buf.push(right.0);
        buf.push(bottom.0);
        buf.push(right.1);
        buf.push(bottom.1);

        buf.push(right.0);
        buf.push(top.0);
        buf.push(right.1);
        buf.push(top.1);
    }

    fn push_word_splitwrap(&mut self, word: &str) {
        let mut iter = word.chars().peekable();
        while let Some(ch) = iter.next() {
            if let Some(metrics) = self.metrics(ch) {
                let kerning = iter.peek()
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
                let kerning = iter.peek()
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
            },
            '\n' => {
                self.push_newline();
            },
            ' ' => {
                self.x_offset += self.settings.font_size * 0.25;
            },
            _ => (),
        }
    }

    pub fn push_str(&mut self, text: &str) {
        let mut remaining = text;
        while remaining.len() > 0 {
            let word_end = remaining.find(char::is_whitespace);
            match word_end {
                None => {
                    self.push_word(remaining);
                    remaining = "";
                },
                Some(0) => {
                    let mut iter = remaining.chars();
                    self.push_whitespace(iter.next().unwrap());
                    remaining = iter.as_str();
                },
                Some(index) => {
                    let (word, next) = remaining.split_at(index);
                    self.push_word(word);
                    remaining = next;
                },
            }
        }
    }

    pub fn push(&mut self, cmd: RichTextCommand) {
        match cmd {
            RichTextCommand::Text(text) => self.push_str(&text),
            RichTextCommand::Bold => {
                self.current_style = self.current_style.bold();
            },
            RichTextCommand::Italic => {
                self.current_style = self.current_style.italic();
            },
            RichTextCommand::ResetBold => {
                self.current_style = self.current_style.unbold();
            },
            RichTextCommand::ResetItalic => {
                self.current_style = self.current_style.unitalic();
            },
        }
    }
}
