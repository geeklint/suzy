/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

//! Types for dealing with formatted text.

use std::borrow::Cow;

use crate::graphics::Color;

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
    /// # use suzy::text::FontStyle;
    /// assert_eq!(FontStyle::Normal.bold(), FontStyle::Bold);
    /// assert_eq!(FontStyle::Bold.bold(), FontStyle::Bold);
    /// assert_eq!(FontStyle::Italic.bold(), FontStyle::BoldItalic);
    /// ```
    #[must_use]
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
    /// # use suzy::text::FontStyle;
    /// assert_eq!(FontStyle::Normal.italic(), FontStyle::Italic);
    /// assert_eq!(FontStyle::Italic.italic(), FontStyle::Italic);
    /// assert_eq!(FontStyle::Bold.italic(), FontStyle::BoldItalic);
    /// ```
    #[must_use]
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
    /// # use suzy::text::FontStyle;
    /// assert_eq!(FontStyle::Normal.unbold(), FontStyle::Normal);
    /// assert_eq!(FontStyle::Bold.unbold(), FontStyle::Normal);
    /// assert_eq!(FontStyle::BoldItalic.unbold(), FontStyle::Italic);
    /// ```
    #[must_use]
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
    /// # use suzy::text::FontStyle;
    /// assert_eq!(FontStyle::Normal.unitalic(), FontStyle::Normal);
    /// assert_eq!(FontStyle::Italic.unitalic(), FontStyle::Normal);
    /// assert_eq!(FontStyle::BoldItalic.unitalic(), FontStyle::Bold);
    /// ```
    #[must_use]
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

/// A enum of commands to be fed into a rich text renderer.
pub enum RichTextCommand<'a> {
    /// Text command to render text.
    Text(Cow<'a, str>),
    /// Command to increase the current font-weight.
    Bold,
    /// Command to make text more italic.
    Italic,
    /// Command to decrease the current font-weight.
    ResetBold,
    /// Command to make text less italic.
    ResetItalic,
}

/// A parser which parses a basic subset of HTML and generates TextCommands.
pub struct RichTextParser<'a> {
    text: &'a str,
}

impl<'a> RichTextParser<'a> {
    /// Create a new RichTextParser with some text
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
                std::mem::take(&mut self.text)
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

/// Text settings contains common settings for rendering text, such as
/// font size, color, and alignment.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub struct TextSettings {
    /// Primary text color.
    pub text_color: Color,

    /// Font size
    pub font_size: f32,

    /// Text Alignment
    pub alignment: TextAlignment,

    /// Base font style
    pub font_style: FontStyle,

    /// Color of outline around glyphs.
    pub outline_color: Color,

    /// Width of the outline around glyphs.
    pub outline_width: f32,

    /// Width between tab stops
    pub tab_stop: f32,
}

impl Default for TextSettings {
    fn default() -> Self {
        Self {
            text_color: Color::WHITE,
            font_size: 24.0,
            alignment: TextAlignment::Left,
            font_style: FontStyle::Normal,
            outline_color: Color::create_rgba(1.0, 1.0, 1.0, 0.0),
            outline_width: 0.0,
            tab_stop: 48.0,
        }
    }
}

/// Struct which describes basic position information for rendering text.
#[derive(Clone, Copy, Debug)]
pub struct TextPosition {
    /// Horizontal position to start rendering the text.
    pub left: f32,
    /// Vertical position to start rendering the text.
    pub top: f32,
    /// Width to wrap text at.
    pub wrap_width: f32,
}
