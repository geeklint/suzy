/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![cfg(feature = "platform_opengl")]

extern crate suzy;

use suzy::app::{App, AppBuilder};
use suzy::graphics::Color;
use suzy::platform::TestPlatform;
use suzy::window::WindowSettings;

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
                Color::create_rgba8(chunk[0], chunk[1], chunk[2], chunk[3]);
            assert_eq!(color, Color::BLACK);
        }
    });
}
