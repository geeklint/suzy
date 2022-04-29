/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

#![cfg(feature = "platform_opengl")]

extern crate suzy;

use suzy::app::{App, AppBuilder};
use suzy::dims::{Padding, Rect, SimplePadding2d};
use suzy::graphics::Color;
use suzy::platforms::opengl::{Masker, OpenGlRenderPlatform, SlicedImage};
use suzy::platforms::TestPlatform;
use suzy::widget::{self, Widget, WidgetDescReceiver, WidgetInit};
use suzy::window::WindowSettings;

mod utils;
use utils::*;

#[derive(Default)]
struct Root {
    mask: Masker<SlicedImage>,
    image: SlicedImage,
}

impl widget::Content<OpenGlRenderPlatform> for Root {
    fn init(mut init: impl WidgetInit<Self, OpenGlRenderPlatform>) {
        init.watch(|root, rect| {
            root.mask.set_fill_width(&rect, Padding::zero());
            root.mask.set_height(rect.height() / 2.0);
            root.mask.set_top(rect.top());
        });
        init.watch(|root, rect| {
            root.image.set_fill(&rect, &SimplePadding2d::zero());
        });
    }

    fn desc(
        mut receiver: impl WidgetDescReceiver<Self, OpenGlRenderPlatform>,
    ) {
        receiver.graphic(|this| &mut this.mask);
        receiver.graphic(|this| &mut this.image);
    }
}

#[test]
fn mask_right_half() {
    let mut builder = AppBuilder::default();
    builder.set_size((480.0, 360.0));
    builder.set_background_color(Color::BLACK);
    let app: App<TestPlatform> = builder.build();
    let app = app
        .with(|app| {
            app.add_root(Widget::<Root>::default);
        })
        .0;
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
