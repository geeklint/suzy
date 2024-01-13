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

    #[track_caller]
    fn set_left(&mut self, value: f32) {
        self.x.set_start(value)
    }

    #[track_caller]
    fn set_right(&mut self, value: f32) {
        self.x.set_end(value)
    }

    #[track_caller]
    fn set_bottom(&mut self, value: f32) {
        self.y.set_start(value)
    }

    #[track_caller]
    fn set_top(&mut self, value: f32) {
        self.y.set_end(value)
    }

    #[track_caller]
    fn set_center_x(&mut self, value: f32) {
        self.x.set_center(value)
    }

    #[track_caller]
    fn set_center_y(&mut self, value: f32) {
        self.y.set_center(value)
    }

    #[track_caller]
    fn set_width(&mut self, value: f32) {
        self.x.length = value;
    }

    #[track_caller]
    fn set_height(&mut self, value: f32) {
        self.y.length = value;
    }

    #[track_caller]
    fn set_pivot(&mut self, value: [f32; 2]) {
        let [px, py] = value;
        self.x.pivot = px;
        self.y.pivot = py;
    }

    #[track_caller]
    fn set_pivot_pos(&mut self, value: [f32; 2]) {
        let [px, py] = value;
        self.x.position = px;
        self.y.position = py;
    }

    #[track_caller]
    fn set_horizontal_stretch(&mut self, left: f32, right: f32) {
        self.x.set_stretch(left, right)
    }

    #[track_caller]
    fn set_vertical_stretch(&mut self, bottom: f32, top: f32) {
        self.y.set_stretch(bottom, top)
    }
}
