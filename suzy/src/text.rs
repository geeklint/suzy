/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Types for dealing with formatted text.

use std::borrow::Cow;

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
