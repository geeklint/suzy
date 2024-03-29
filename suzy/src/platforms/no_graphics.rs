/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use crate::{
    dims::Rect,
    graphics::Color,
    platform::{graphics, RenderPlatform},
};

#[derive(Clone, Copy, Debug, Default)]
pub struct NoGraphics;

pub enum TextStyle {}
pub enum Graphic {}

impl RenderPlatform for NoGraphics {
    type DrawContextBuilder = fn(&mut ()) -> Self;

    type SlicedImage = Graphic;

    type TextStyle = TextStyle;

    type Text = Graphic;
}

impl graphics::TextStyle for TextStyle {
    fn with_size_and_color(_size: f32, _color: Color) -> Self {
        unimplemented!("Can't construct TextStyle for NoGraphics")
    }

    fn push_tag(
        &self,
        _tag: &mut &str,
    ) -> Result<Self, crate::text::RichTextTagParseError> {
        unreachable!()
    }
}

impl Default for Graphic {
    fn default() -> Self {
        unimplemented!("Can't construct a graphic for the NoGraphics platform")
    }
}

impl crate::graphics::Graphic<NoGraphics> for Graphic {
    fn draw(
        &mut self,
        _ctx: &mut crate::graphics::DrawContext<'_, NoGraphics>,
    ) {
        unreachable!()
    }
}

impl Rect for Graphic {
    fn x(&self) -> crate::dims::Dim {
        unreachable!()
    }

    fn y(&self) -> crate::dims::Dim {
        unreachable!()
    }

    fn set_left(&mut self, _value: f32) {
        unreachable!()
    }

    fn set_right(&mut self, _value: f32) {
        unreachable!()
    }

    fn set_bottom(&mut self, _value: f32) {
        unreachable!()
    }

    fn set_top(&mut self, _value: f32) {
        unreachable!()
    }

    fn set_center_x(&mut self, _value: f32) {
        unreachable!()
    }

    fn set_center_y(&mut self, _value: f32) {
        unreachable!()
    }

    fn set_width(&mut self, _value: f32) {
        unreachable!()
    }

    fn set_height(&mut self, _value: f32) {
        unreachable!()
    }

    fn set_pivot(&mut self, _value: [f32; 2]) {
        unreachable!()
    }

    fn set_pivot_pos(&mut self, _value: [f32; 2]) {
        unreachable!()
    }

    fn set_horizontal_stretch(&mut self, _left: f32, _right: f32) {
        unreachable!()
    }

    fn set_vertical_stretch(&mut self, _bottom: f32, _top: f32) {
        unreachable!()
    }
}

impl graphics::SlicedImage for Graphic {
    fn set_color(&mut self, _color: Color) {
        unreachable!()
    }

    fn set_slice_padding(&mut self, _padding: crate::dims::Padding2d) {
        unreachable!()
    }

    fn set_corners(&mut self, _style: crate::graphics::CornerStyle) {
        unreachable!()
    }
}

impl graphics::Text<TextStyle> for Graphic {
    fn set_layout(&mut self, _layout: crate::text::Layout) {
        unreachable!()
    }

    fn clear(&mut self) {
        unreachable!()
    }

    fn push_span(&mut self, _style: TextStyle, _text: &str) {
        unreachable!()
    }

    fn finish(&mut self) {
        unreachable!()
    }
}
