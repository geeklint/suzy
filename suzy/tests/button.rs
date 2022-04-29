/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

#![cfg(feature = "platform_opengl")]

extern crate suzy;

use suzy::app::{App, AppBuilder};
use suzy::dims::{Rect, SimplePadding2d};
use suzy::graphics::{Color, Conditional};
use suzy::platforms::opengl::{OpenGlRenderPlatform, SlicedImage};
use suzy::platforms::TestPlatform;
use suzy::pointer::{PointerAction, PointerEventData, PointerId};
use suzy::selectable::{Selectable, SelectionState};
use suzy::widget::{self, Widget, WidgetDescReceiver, WidgetInit};
use suzy::widgets::Button;
use suzy::window::WindowSettings;

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
    fn init(mut init: impl WidgetInit<Self, OpenGlRenderPlatform>) {
        init.watch(|this, rect| {
            this.image.graphic.set_fill(&rect, &SimplePadding2d::zero());
        });
    }

    fn desc(
        mut receiver: impl WidgetDescReceiver<Self, OpenGlRenderPlatform>,
    ) {
        receiver.graphic(|this| &mut this.image);
    }
}

#[derive(Default)]
struct Root {
    button: Button<ButtonContent>,
}

impl widget::Content<OpenGlRenderPlatform> for Root {
    fn init(mut init: impl WidgetInit<Self, OpenGlRenderPlatform>) {
        init.watch(|root, rect| {
            root.button.set_fill(rect, &SimplePadding2d::zero());
        });
    }

    fn desc(
        mut receiver: impl WidgetDescReceiver<Self, OpenGlRenderPlatform>,
    ) {
        receiver.child(|this| &mut this.button);
    }
}

#[test]
fn button() {
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
        for chunk in capture.chunks_exact(4) {
            let color =
                Color::create_rgba8(chunk[0], chunk[1], chunk[2], chunk[3]);
            assert_eq!(color, Color::BLACK);
        }
        app.pointer(PointerEventData {
            id: PointerId::Other(1),
            action: PointerAction::Down,
            x: 240.0,
            y: 180.0,
            normalized: true,
        });
        let capture = app.take_screenshot();
        for chunk in capture.chunks_exact(4) {
            let color =
                Color::create_rgba8(chunk[0], chunk[1], chunk[2], chunk[3]);
            assert_eq!(color, Color::WHITE);
        }
        app.pointer(PointerEventData {
            id: PointerId::Other(1),
            action: PointerAction::Up,
            x: 240.0,
            y: 180.0,
            normalized: true,
        });
        let capture = app.take_screenshot();
        for chunk in capture.chunks_exact(4) {
            let color =
                Color::create_rgba8(chunk[0], chunk[1], chunk[2], chunk[3]);
            assert_eq!(color, Color::BLACK);
        }
    });
}
