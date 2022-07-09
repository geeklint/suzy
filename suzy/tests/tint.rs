/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

#![cfg(feature = "platform_opengl")]

use suzy::app::{App, AppBuilder};
use suzy::dims::{Rect, SimplePadding2d};
use suzy::graphics::Color;
use suzy::platforms::opengl::{OpenGlRenderPlatform, SlicedImage, Tint};
use suzy::platforms::TestPlatform;
use suzy::widget::{self, Widget};
use suzy::window::WindowSettings;

mod utils;
use utils::*;

#[derive(Default)]
struct Root {
    tint: Tint,
    image: SlicedImage,
}

impl widget::Content<OpenGlRenderPlatform> for Root {
    fn desc(mut desc: impl widget::Desc<Self, OpenGlRenderPlatform>) {
        desc.watch(|root, _rect| {
            root.tint.set_tint_color(Color::RED);
        });
        desc.watch(|root, rect| {
            root.image.set_fill(&rect, &SimplePadding2d::zero());
        });
        desc.graphic(|this| &mut this.tint);
        desc.graphic(|this| &mut this.image);
    }
}

#[test]
fn tint_red() {
    let mut builder = AppBuilder::default();
    builder.set_size((480.0, 360.0));
    builder.set_background_color(Color::BLACK);
    let mut app: App<TestPlatform> = builder.build();
    app.add_root(Widget::<Root>::default());
    app.test(|mut app| {
        let capture = app.take_screenshot();
        assert!(is_color(&capture, Color::RED));
    });
}
