/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

#![cfg(any(feature = "platform_osmesa", feature = "platform_sdl"))]

use suzy::{
    app::{App, AppBuilder},
    graphics::Color,
    platforms::TestPlatform,
    window::WindowSettings,
};

#[test]
fn smoke() {
    let mut builder = AppBuilder::default();
    builder.set_size((480.0, 360.0));
    builder.set_background_color(Color::BLACK);
    let app: App<TestPlatform> = builder.build();
    app.test(|mut app| {
        let capture = app.take_screenshot();
        for chunk in capture.chunks_exact(4) {
            let color =
                Color::from_rgba8(chunk[0], chunk[1], chunk[2], chunk[3]);
            assert_eq!(color, Color::BLACK);
        }
    });
}
