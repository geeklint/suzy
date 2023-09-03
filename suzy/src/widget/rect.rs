/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use drying_paint::Watched;

use crate::dims::{Dim, Rect};

/// A version of Rect where each dimension will trigger watching functions
#[derive(Default)]
pub struct WidgetRect {
    x: Watched<Dim>,
    y: Watched<Dim>,
}

impl WidgetRect {
    pub(super) fn from_rect<R>(rect: &R) -> Self
    where
        R: Rect + ?Sized,
    {
        let x = Watched::new(rect.x());
        let y = Watched::new(rect.y());
        Self { x, y }
    }
}

impl Rect for WidgetRect {
    fn x(&self) -> Dim {
        *self.x
    }

    fn y(&self) -> Dim {
        *self.y
    }

    fn set_left(&mut self, value: f32) {
        self.x.set_start(value)
    }

    fn set_right(&mut self, value: f32) {
        self.x.set_end(value)
    }

    fn set_bottom(&mut self, value: f32) {
        self.y.set_start(value)
    }

    fn set_top(&mut self, value: f32) {
        self.y.set_end(value)
    }

    fn set_center_x(&mut self, value: f32) {
        self.x.set_center(value)
    }

    fn set_center_y(&mut self, value: f32) {
        self.y.set_center(value)
    }

    fn set_width(&mut self, value: f32) {
        self.x.length = value;
    }

    fn set_height(&mut self, value: f32) {
        self.y.length = value;
    }

    fn set_pivot(&mut self, value: (f32, f32)) {
        self.x.pivot = value.0;
        self.y.pivot = value.0;
    }

    fn set_pivot_pos(&mut self, value: (f32, f32)) {
        self.x.position = value.0;
        self.y.position = value.1;
    }

    fn set_horizontal_stretch(&mut self, left: f32, right: f32) {
        self.x.set_stretch(left, right)
    }

    fn set_vertical_stretch(&mut self, bottom: f32, top: f32) {
        self.y.set_stretch(bottom, top)
    }
}
