/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This describes traits which apply to a set of graphic primitives a
//! platform must implement to support Suzy's built-in widgets.

use crate::dims::Padding2d;
use crate::selectable::{Selectable, SelectionState};
use crate::text::{RichTextCommand, TextPosition, TextSettings};

pub trait Texture {
    fn load_static(
        width: u16,
        height: u16,
        alignment: u16,
        pixels: &'static [u8],
    ) -> Self;
}

pub trait SlicedImage<T> {
    fn set_image<P>(&mut self, texture: T, padding: P)
    where
        P: Padding2d;
}

pub trait SelectableSlicedImage<T>: Selectable {
    fn set_image<P>(
        &mut self,
        texture: T,
        padding: P,
        states: &'static [SelectionState],
    ) where
        P: Padding2d;
}

pub trait Text {
    fn set_text<'a, T>(
        &mut self,
        text: T,
        pos: &TextPosition,
        settings: &TextSettings,
    ) where
        T: 'a + Iterator<Item = RichTextCommand<'a>>;
}
