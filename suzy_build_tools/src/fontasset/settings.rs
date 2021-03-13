/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::path::Path;

#[derive(Clone, Copy, PartialEq)]
pub enum AssetSize {
    FontSize(f64),
    TextureSize(usize),
}

pub struct Settings {
    pub(super) chars: Vec<char>,
    pub(super) padding_ratio: f64,
    pub(super) size: AssetSize,
    pub(super) progressbar: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            chars: Vec::new(),
            padding_ratio: 0.5,
            size: AssetSize::TextureSize(64),
            progressbar: false,
        }
    }
}

impl Settings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn padding_ratio(mut self, ratio: f64) -> Self {
        self.padding_ratio = ratio;
        self
    }

    pub fn size(mut self, size: AssetSize) -> Self {
        self.size = size;
        self
    }

    pub fn add_chars<I>(mut self, chars: I) -> Settings
    where
        I: IntoIterator<Item = char>,
    {
        self.chars.extend(chars);
        self
    }

    pub fn ascii(self) -> Settings {
        self.add_chars((b'!'..=b'~').map(char::from))
    }

    pub fn latin1(self) -> Settings {
        self.ascii().add_chars((0xa1..=0xff).map(char::from))
    }

    pub fn show_progress(self) -> Settings {
        Settings {
            progressbar: true,
            ..self
        }
    }
}

use std::borrow::Cow;

enum FontSource<'a> {
    Path(&'a Path),
    Bytes(Cow<'a, [u8]>),
}

fn parse_font(source: FontSource) -> Result<rusttype::Font, String> {
    Ok(match source {
        FontSource::Path(path) => {
            let bytes = std::fs::read(path)
                .map_err(|_| format!("Failed to read font file {:?}", path))?;
            rusttype::Font::try_from_vec(bytes).ok_or_else(|| {
                format!("Failed to parse font file {:?}", path)
            })?
        }
        FontSource::Bytes(cow) => match cow {
            Cow::Borrowed(bytes) => rusttype::Font::try_from_bytes(bytes)
                .ok_or_else(|| "Failed to parse font data".to_string())?,
            Cow::Owned(bytes) => rusttype::Font::try_from_vec(bytes)
                .ok_or_else(|| "Failed to parse font data".to_string())?,
        },
    })
}

pub struct FontFamily<'n, 'b, 'i, 'bi> {
    normal: FontSource<'n>,
    bold: Option<FontSource<'b>>,
    italic: Option<FontSource<'i>>,
    bold_italic: Option<FontSource<'bi>>,
}

pub(super) struct ParsedFontFamily<'n, 'b, 'i, 'bi> {
    pub(super) normal: Result<rusttype::Font<'n>, String>,
    pub(super) bold: Option<Result<rusttype::Font<'b>, String>>,
    pub(super) italic: Option<Result<rusttype::Font<'i>, String>>,
    pub(super) bold_italic: Option<Result<rusttype::Font<'bi>, String>>,
}

impl<'n> FontFamily<'n, 'static, 'static, 'static> {
    pub fn font_path<P: AsRef<Path> + ?Sized>(font_path: &'n P) -> Self {
        Self {
            normal: FontSource::Path(font_path.as_ref()),
            bold: None,
            italic: None,
            bold_italic: None,
        }
    }

    pub fn font_bytes<B: Into<Cow<'n, [u8]>>>(font_bytes: B) -> Self {
        Self {
            normal: FontSource::Bytes(font_bytes.into()),
            bold: None,
            italic: None,
            bold_italic: None,
        }
    }
}

impl<'n, 'b, 'i, 'bi> FontFamily<'n, 'b, 'i, 'bi> {
    pub(super) fn parse(self) -> ParsedFontFamily<'n, 'b, 'i, 'bi> {
        ParsedFontFamily {
            normal: parse_font(self.normal),
            bold: self.bold.map(parse_font),
            italic: self.italic.map(parse_font),
            bold_italic: self.bold_italic.map(parse_font),
        }
    }

    pub fn bold_path<'bnew, P: AsRef<Path> + ?Sized>(
        self,
        font_path: &'bnew P,
    ) -> FontFamily<'n, 'bnew, 'i, 'bi> {
        FontFamily {
            normal: self.normal,
            bold: Some(FontSource::Path(font_path.as_ref())),
            italic: self.italic,
            bold_italic: self.bold_italic,
        }
    }

    pub fn italic_path<'inew, P: AsRef<Path> + ?Sized>(
        self,
        font_path: &'inew P,
    ) -> FontFamily<'n, 'b, 'inew, 'bi> {
        FontFamily {
            normal: self.normal,
            bold: self.bold,
            italic: Some(FontSource::Path(font_path.as_ref())),
            bold_italic: self.bold_italic,
        }
    }

    pub fn bold_italic_path<'binew, P: AsRef<Path> + ?Sized>(
        self,
        font_path: &'binew P,
    ) -> FontFamily<'n, 'b, 'i, 'binew> {
        FontFamily {
            normal: self.normal,
            bold: self.bold,
            italic: self.italic,
            bold_italic: Some(FontSource::Path(font_path.as_ref())),
        }
    }

    pub fn bold_bytes<'bnew, B: Into<Cow<'bnew, [u8]>>>(
        self,
        font_bytes: B,
    ) -> FontFamily<'n, 'bnew, 'i, 'bi> {
        FontFamily {
            normal: self.normal,
            bold: Some(FontSource::Bytes(font_bytes.into())),
            italic: self.italic,
            bold_italic: self.bold_italic,
        }
    }

    pub fn italic_bytes<'inew, B: Into<Cow<'inew, [u8]>>>(
        self,
        font_bytes: B,
    ) -> FontFamily<'n, 'b, 'inew, 'bi> {
        FontFamily {
            normal: self.normal,
            bold: self.bold,
            italic: Some(FontSource::Bytes(font_bytes.into())),
            bold_italic: self.bold_italic,
        }
    }

    pub fn bold_italic_bytes<'binew, B: Into<Cow<'binew, [u8]>>>(
        self,
        font_bytes: B,
    ) -> FontFamily<'n, 'b, 'i, 'binew> {
        FontFamily {
            normal: self.normal,
            bold: self.bold,
            italic: self.italic,
            bold_italic: Some(FontSource::Bytes(font_bytes.into())),
        }
    }
}
