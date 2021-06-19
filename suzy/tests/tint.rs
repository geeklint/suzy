/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

#![cfg(feature = "platform_opengl")]

extern crate suzy;

use suzy::app::{App, AppBuilder};
use suzy::dims::{Rect, SimplePadding2d};
use suzy::graphics::Color;
use suzy::platforms::opengl::{OpenGlRenderPlatform, SlicedImage, Tint};
use suzy::platforms::TestPlatform;
use suzy::widget::{
    Widget, WidgetChildReceiver, WidgetContent, WidgetGraphicReceiver,
    WidgetInit,
};
use suzy::window::WindowSettings;

mod utils;
use utils::*;

#[derive(Default)]
struct Root {
    tint: Tint,
    image: SlicedImage,
}

impl WidgetContent<OpenGlRenderPlatform> for Root {
    fn init(mut init: impl WidgetInit<Self, OpenGlRenderPlatform>) {
        init.watch(|root, _rect| {
            root.tint.set_tint_color(Color::RED);
        });
        init.watch(|root, rect| {
            root.image.set_fill(&rect, &SimplePadding2d::zero());
        });
    }

    fn children(
        &mut self,
        _receiver: impl WidgetChildReceiver<OpenGlRenderPlatform>,
    ) {
    }

    fn graphics(
        &mut self,
        mut receiver: impl WidgetGraphicReceiver<OpenGlRenderPlatform>,
    ) {
        receiver.graphic(&mut self.tint);
        receiver.graphic(&mut self.image);
    }
}

#[test]
fn tint_red() {
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
        assert!(is_color(&capture, Color::RED));
    });
}
