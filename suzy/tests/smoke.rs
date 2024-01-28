/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

#![cfg(any(feature = "platform_osmesa", feature = "platform_sdl"))]

use suzy::{
    app::{App, AppTestingExt},
    graphics::Color,
    platforms::TestPlatform,
};

#[test]
fn smoke() {
    let mut platform = TestPlatform::new();
    let window = suzy::platform::Platform::create_window(
        &mut platform,
        suzy::window::WindowBuilder {
            size: [480.0, 360.0],
            background_color: Color::BLACK,
            ..suzy::window::WindowBuilder::default()
        },
    )
    .expect("Failed to create window");
    let mut app = App::<TestPlatform>::from_window(window);
    let capture = app.draw_and_take_screenshot();
    for chunk in capture.chunks_exact(4) {
        let color = Color::from_rgba8(chunk[0], chunk[1], chunk[2], chunk[3]);
        assert_eq!(color, Color::BLACK);
    }
}
