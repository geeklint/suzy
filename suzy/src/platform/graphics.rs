/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This describes traits which apply to a set of graphic primitives a
//! platform must implement to support Suzy's built-in widgets.

use crate::dims::{Padding2d, Rect};
use crate::selectable::{Selectable, SelectionState};
use crate::text::{RichTextCommand, TextPosition, TextSettings};

/// A platform's primitive texture type.
pub trait Texture {
    /// Load a texture from static memory (e.g. include_bytes!())
    fn load_static(
        width: u16,
        height: u16,
        alignment: u16,
        pixels: &'static [u8],
    ) -> Self;
}

/// A platform's 9-slice image graphic primitive.
pub trait SlicedImage<T>: Rect {
    /// Set the image to be drawn from a given texture and padding.
    fn set_image<P>(&mut self, texture: T, padding: P)
    where
        P: Padding2d;
}

/// A platform's selectable 9-slice image graphic primitive.
pub trait SelectableSlicedImage<T>: Rect + Selectable {
    /// Set the image to be drawn from a given texture and padding, and
    /// states present in the image.
    fn set_image<P>(
        &mut self,
        texture: T,
        padding: P,
        states: &'static [SelectionState],
    ) where
        P: Padding2d;
}

/// A platform's text graphic primitive.
pub trait Text {
    /// Set the text to be rendered with the given rich text commands,
    /// position information, and render settings.
    fn set_text<'a, T>(
        &mut self,
        text: T,
        pos: &TextPosition,
        settings: &TextSettings,
    ) where
        T: 'a + Iterator<Item = RichTextCommand<'a>>;

    /// Set the text to be rendered with the given position information,
    /// and render settings, parsing the text as rich text.
    fn set_text_rich(
        &mut self,
        text: &str,
        pos: &TextPosition,
        settings: &TextSettings,
    ) {
        use crate::text::RichTextParser;
        self.set_text(RichTextParser::new(text), pos, settings);
    }

    /// Set the text to be rendered with the given position information,
    /// and render settings, assuming the text is plain.
    fn set_text_plain(
        &mut self,
        text: &str,
        pos: &TextPosition,
        settings: &TextSettings,
    ) {
        let cmd = RichTextCommand::Text(text.into());
        self.set_text(Some(cmd).into_iter(), pos, settings);
    }
}
