extern crate suzy;

use suzy::window::WindowSettings;
use suzy::math::Color;
use suzy::math::consts::BLACK;
use suzy::app::{
    App,
    AppBuilder,
};
use suzy::platform::sdl2::SDLPlatform;

#[test]
fn smoke() {
    let mut builder = AppBuilder::default();
    builder.set_size((480.0, 360.0));
    builder.set_background_color(BLACK);
    let app: App<SDLPlatform> = builder.build();
    app.test(|mut app| {
        let capture = app.take_screenshot();
        for chunk in capture.chunks_exact(4) {
            let color = Color::create_rgba8(
                chunk[0], chunk[1], chunk[2], chunk[3]
            );
            assert_eq!(color, BLACK);
        }
    });
}
