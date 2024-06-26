/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

#![cfg(any(feature = "platform-osmesa", feature = "platform-sdl"))]

use suzy::{
    app::App,
    graphics::Color,
    platforms::{opengl::OpenGlRenderPlatform, TEST_ENV},
};

#[test]
fn smoke() {
    let mut window = unsafe { TEST_ENV.initialize(480, 360) };
    let mut app = App::<OpenGlRenderPlatform>::new(480.0, 360.0);
    let capture = window.draw_and_take_screenshot(&mut app);
    for chunk in capture.chunks_exact(4) {
        let color = Color::from_rgba8(chunk[0], chunk[1], chunk[2], chunk[3]);
        assert_eq!(color, Color::BLACK);
    }
}
