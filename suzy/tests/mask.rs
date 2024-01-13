/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

#![cfg(any(feature = "platform_osmesa", feature = "platform_sdl"))]

use suzy::{
    app::AppBuilder,
    dims::{Padding, Padding2d, Rect},
    graphics::Color,
    platforms::{
        opengl::{Mask, OpenGlRenderPlatform, SlicedImage},
        TestPlatform,
    },
    widget::{self, Widget},
    window::WindowSettings,
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
    let mut builder = AppBuilder::default();
    builder.set_size([480.0, 360.0]);
    builder.set_background_color(Color::BLACK);
    let mut platform = <TestPlatform as suzy::platform::Platform>::new();
    let mut app = builder.build(&mut platform);
    app.add_root(Widget::<Root>::default());
    app.test(|mut app| {
        let capture = app.take_screenshot();
        let index = (capture.len() / 2) & ALIGN_MASK;
        let (bottom, top) = capture.split_at(index);
        let bottom = round_back(bottom);
        let top = round_front(top);
        assert!(is_color(bottom, Color::BLACK));
        assert!(is_color(top, Color::WHITE));
    });
}
