/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

#![cfg(any(feature = "platform_osmesa", feature = "platform_sdl"))]

use suzy::{
    app::{App, AppTestingExt},
    dims::{Padding2d, Rect},
    graphics::{Color, Conditional},
    platform,
    platforms::{
        opengl::{OpenGlRenderPlatform, SlicedImage},
        TestPlatform,
    },
    pointer::{PointerAction, PointerEventData, PointerId},
    selectable::{Selectable, SelectionState},
    widget::{self, Widget},
    widgets::Button,
};

#[derive(Default)]
struct ButtonContent {
    image: Conditional<SlicedImage>,
}

impl Selectable for ButtonContent {
    fn selection_changed(&mut self, state: SelectionState) {
        self.image.enable = state == SelectionState::active();
    }
}

impl widget::Content<OpenGlRenderPlatform> for ButtonContent {
    fn desc(mut desc: impl widget::Desc<Self, OpenGlRenderPlatform>) {
        desc.watch(|this, rect| {
            this.image.graphic.set_fill(rect, &Padding2d::zero());
        });
        desc.graphic(|this| &mut this.image);
    }
}

#[derive(Default)]
struct Root {
    button: Button<ButtonContent>,
}

impl widget::Content<OpenGlRenderPlatform> for Root {
    fn desc(mut desc: impl widget::Desc<Self, OpenGlRenderPlatform>) {
        desc.watch(|root, rect| {
            root.button.set_fill(rect, &Padding2d::zero());
        });
        desc.child(|this| &mut this.button);
    }
}

#[test]
fn button() {
    let mut platform = <TestPlatform as platform::Platform>::new();
    let window = platform::Platform::create_window(
        &mut platform,
        suzy::window::WindowBuilder {
            size: [480.0, 360.0],
            background_color: Color::BLACK,
            ..suzy::window::WindowBuilder::default()
        },
    )
    .expect("Failed to create window");
    let mut app = App::<TestPlatform>::from_window(window);
    app.add_root(Widget::<Root>::default());
    let capture = app.draw_and_take_screenshot();
    for chunk in capture.chunks_exact(4) {
        let color = Color::from_rgba8(chunk[0], chunk[1], chunk[2], chunk[3]);
        assert_eq!(color, Color::BLACK);
    }
    app.pointer_event(PointerEventData {
        id: PointerId::Other(1),
        action: PointerAction::Down,
        x: 240.0,
        y: 180.0,
    });
    let capture = app.draw_and_take_screenshot();
    for chunk in capture.chunks_exact(4) {
        let color = Color::from_rgba8(chunk[0], chunk[1], chunk[2], chunk[3]);
        assert_eq!(color, Color::WHITE);
    }
    app.pointer_event(PointerEventData {
        id: PointerId::Other(1),
        action: PointerAction::Up,
        x: 240.0,
        y: 180.0,
    });
    let capture = app.draw_and_take_screenshot();
    for chunk in capture.chunks_exact(4) {
        let color = Color::from_rgba8(chunk[0], chunk[1], chunk[2], chunk[3]);
        assert_eq!(color, Color::BLACK);
    }
}
