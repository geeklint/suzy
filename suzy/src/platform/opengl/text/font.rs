/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::platform::opengl;
use opengl::context::bindings::types::*;
use opengl::texture::Texture;

use super::FontStyle;

// 0: char
// 1,2: u,v
// 3,4: uv width,height
// 5-8: relative bb
// 9: relative advance width
pub(super) type GlyphMetricsSource =
    (char, f32, f32, f32, f32, f32, f32, f32, f32, f32);
type KerningSource = (char, char, f32);

pub type FontSource<'a> = (
    u8, // texture channel
    &'a [GlyphMetricsSource],
    &'a [KerningSource],
);

/// A source for a font family.
///
/// This is normally created automatically using the crate `suzy_build_tools`.
pub type FontFamilySource = FontFamilySourceDynamic<'static>;

/// A font family which has been loaded.
pub type FontFamily = FontFamilyDynamic<'static>;

/// A source for a font family.
///
/// This is normally created automatically using the crate `suzy_build_tools`.
pub struct FontFamilySourceDynamic<'a> {
    /// The bytes of the SDF font atlas texture.
    pub atlas_image: &'static [u8],

    /// The number of channels in the atlas texture.
    pub image_channels: GLsizei,

    /// The width of the atlas texture.
    pub image_width: u16,

    /// The height of the atlas texture.
    pub image_height: u16,

    /// The alignment of the pixel rows in the atlas texture.
    pub image_row_alignment: u16,

    /// The font source for the normal font.
    pub normal: FontSource<'a>,

    /// The font source for the bold font.
    pub bold: Option<FontSource<'a>>,

    /// The font source for the italic font.
    pub italic: Option<FontSource<'a>>,

    /// The font source for the bold and italic font.
    pub bold_italic: Option<FontSource<'a>>,
}

pub(super) type ChannelMask = (u8, u8, u8, u8);

const ALPHA_MASKS: &[ChannelMask] = &[(0, 0, 0, 0xff)];
const RGB_MASKS: &[ChannelMask] =
    &[(0xff, 0, 0, 0), (0, 0xff, 0, 0), (0, 0, 0xff, 0)];
const RGBA_MASKS: &[ChannelMask] = &[
    (0xff, 0, 0, 0),
    (0, 0xff, 0, 0),
    (0, 0, 0xff, 0),
    (0, 0, 0, 0xff),
];

impl<'a> FontFamilySourceDynamic<'a> {
    /// Load a font family from this source.
    pub fn load(&self) -> FontFamilyDynamic<'a> {
        let (texture, channel_masks) = match self.image_channels {
            1 => {
                let texture = Texture::from_alpha(
                    self.image_width,
                    self.image_height,
                    self.image_row_alignment,
                    self.atlas_image,
                );
                (texture, ALPHA_MASKS)
            }
            3 => {
                let texture = Texture::from_rgb(
                    self.image_width,
                    self.image_height,
                    self.image_row_alignment,
                    self.atlas_image,
                );
                (texture, RGB_MASKS)
            }
            4 => {
                let texture = Texture::from_rgba(
                    self.image_width,
                    self.image_height,
                    self.image_row_alignment,
                    self.atlas_image,
                );
                (texture, RGBA_MASKS)
            }
            _ => panic!(
                concat!(
                    "Invalid number of channels specified ({}). Must be one",
                    " of (1, 3, or 4)"
                ),
                self.image_channels,
            ),
        };
        FontFamilyDynamic {
            texture,
            channel_masks,
            normal: self.normal,
            bold: self.bold,
            italic: self.italic,
            bold_italic: self.bold_italic,
        }
    }
}

/// A font family which has been loaded.
#[derive(Clone)]
pub struct FontFamilyDynamic<'a> {
    pub(super) texture: Texture,
    pub(super) channel_masks: &'static [ChannelMask],
    pub(super) normal: FontSource<'a>,
    pub(super) bold: Option<FontSource<'a>>,
    pub(super) italic: Option<FontSource<'a>>,
    pub(super) bold_italic: Option<FontSource<'a>>,
}

impl<'a> FontFamilyDynamic<'a> {
    pub(super) fn channel_mask(&self, style: FontStyle) -> ChannelMask {
        let index: usize = self.best_font_source(style).0.into();
        self.channel_masks
            .get(index)
            .copied()
            .unwrap_or((0, 0, 0, 0))
    }

    #[doc(hidden)]
    pub fn best_font_source(&self, style: FontStyle) -> &FontSource {
        match style {
            FontStyle::Normal => &self.normal,
            FontStyle::Bold => self.bold.as_ref().unwrap_or(&self.normal),
            FontStyle::Italic => self.italic.as_ref().unwrap_or(&self.normal),
            FontStyle::BoldItalic => self
                .bold_italic
                .as_ref()
                .or_else(|| self.bold.as_ref())
                .or_else(|| self.italic.as_ref())
                .unwrap_or(&self.normal),
        }
    }
}
