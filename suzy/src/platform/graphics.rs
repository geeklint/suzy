/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

//! This describes traits which apply to a set of graphic primitives a
//! platform must implement to support Suzy's built-in widgets.

use crate::{
    dims::Padding2d,
    graphics::{Color, CornerStyle},
    text,
};

/// A platform's 9-slice image graphic primitive.
pub trait SlicedImage {
    fn set_color(&mut self, color: Color);
    fn set_slice_padding(&mut self, padding: impl Padding2d);
    fn set_corners(&mut self, style: CornerStyle);
}

pub trait TextStyle: Sized {
    fn with_size_and_color(size: f32, color: Color) -> Self;
    fn push_tag(&self, tag: &mut &str) -> Result<Self, ()>;
}

/// A platform's text graphic primitive.
pub trait Text<Style> {
    fn set_layout(&mut self, layout: text::Layout);
    fn clear(&mut self);
    fn push_span(&mut self, style: Style, text: &str);
    fn finish(&mut self);
}
