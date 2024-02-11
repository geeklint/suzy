/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

#![cfg(any(feature = "platform_osmesa", feature = "platform_sdl"))]

use suzy::{
    app::App,
    dims::{Padding, Padding2d, Rect},
    graphics::Color,
    platforms::{
        opengl::{Mask, OpenGlRenderPlatform, SlicedImage},
        TEST_ENV,
    },
    widget::{self, Widget},
};

mod utils;
use utils::*;

#[derive(Default)]
struct Root {
    mask: Mask<SlicedImage>,
    image: SlicedImage,
}

impl widget::Content<OpenGlRenderPlatform> for Root {
    fn desc(mut desc: impl widget::Desc<Self, OpenGlRenderPlatform>) {
        desc.watch(|root, rect| {
            root.mask.graphic.set_fill_width(rect, Padding::zero());
            root.mask.graphic.set_height(rect.height() / 2.0);
            root.mask.graphic.set_top(rect.top());
        });
        desc.watch(|root, rect| {
            root.image.set_fill(rect, &Padding2d::zero());
        });
        desc.graphic(|this| &mut this.mask);
        desc.graphic(|this| &mut this.image);
    }
}

#[test]
fn mask_right_half() {
    let mut window = unsafe { TEST_ENV.initialize(480, 360) };
    let mut app = App::<OpenGlRenderPlatform>::new(480.0, 360.0);
    app.add_root(Widget::<Root>::default());
    let capture = window.draw_and_take_screenshot(&mut app);
    let index = (capture.len() / 2) & ALIGN_MASK;
    let (bottom, top) = capture.split_at(index);
    let bottom = round_back(bottom);
    let top = round_front(top);
    assert!(is_color(bottom, Color::BLACK));
    assert!(is_color(top, Color::WHITE));
}
