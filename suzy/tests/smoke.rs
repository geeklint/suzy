/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

#![cfg(any(feature = "platform_osmesa", feature = "platform_sdl"))]

use suzy::{
    app::App,
    graphics::Color,
    platforms::{TestEnvWindow, TestPlatform},
};

#[test]
fn smoke() {
    let mut window = unsafe { TestEnvWindow::new(480, 360) };
    let mut app = App::<TestPlatform>::from_window(&window);
    let capture = window.draw_and_take_screenshot(&mut app);
    for chunk in capture.chunks_exact(4) {
        let color = Color::from_rgba8(chunk[0], chunk[1], chunk[2], chunk[3]);
        assert_eq!(color, Color::BLACK);
    }
}
