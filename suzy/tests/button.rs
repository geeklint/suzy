/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![cfg(feature = "platform_opengl")]

extern crate suzy;

use suzy::dims::{
    Rect,
    SimplePadding2d,
    Padding2dNew,
};
use suzy::window::WindowSettings;
use suzy::graphics::Color;
use suzy::widgets::Button;
use suzy::app::{
    App,
    AppBuilder,
};
use suzy::pointer::{
    PointerId,
    PointerAction,
    PointerEventData,
};
use suzy::selectable::{
    Selectable,
    SelectionState,
};
use suzy::widget::{
    Widget,
    WidgetContent,
};
use suzy::platform::opengl::{
    OpenGlRenderPlatform,
    SlicedImage,
};
use suzy::platform::TestPlatform;

#[derive(Default)]
struct ButtonContent {
    show_image: bool,
    image: SlicedImage,
}

impl Selectable for ButtonContent {
    fn selection_changed(&mut self, state: SelectionState) {
        self.show_image = state == SelectionState::active();
    }
}

impl WidgetContent<OpenGlRenderPlatform> for ButtonContent {
    fn init<I: suzy::widget::WidgetInit<Self, OpenGlRenderPlatform>>(mut init: I) {
        init.watch(|this, rect| {
            this.image.set_fill(&rect, &SimplePadding2d::zero());
        });
    }

    fn children<R: suzy::widget::WidgetChildReceiver<OpenGlRenderPlatform>>(&mut self, _receiver: R) {
    }

    fn graphics<R: suzy::widget::WidgetGraphicReceiver<OpenGlRenderPlatform>>(&mut self, mut receiver: R) {
        println!("ButtonContent draw: {}", self.show_image);
        if self.show_image {
            receiver.graphic(&mut self.image);
        }
    }

}

#[derive(Default)]
struct Root {
    button: Button<ButtonContent>,
}

impl WidgetContent<OpenGlRenderPlatform> for Root {
    fn init<I: suzy::widget::WidgetInit<Self, OpenGlRenderPlatform>>(mut init: I) {
        init.watch(|root, rect| {
            root.button.set_fill(rect, &SimplePadding2d::zero());
        });
    }

    fn children<R: suzy::widget::WidgetChildReceiver<OpenGlRenderPlatform>>(&mut self, mut receiver: R) {
        receiver.child(&mut self.button);
    }

    fn graphics<R: suzy::widget::WidgetGraphicReceiver<OpenGlRenderPlatform>>(&mut self, _receiver: R) {
    }

}

#[test]
fn button() {
    let mut builder = AppBuilder::default();
    builder.set_size((480.0, 360.0));
    builder.set_background_color(Color::BLACK);
    let app: App<TestPlatform> = builder.build();
    let app = app.with(|app| {
        app.add_root(Widget::<Root>::default);
    }).0;
    app.test(|mut app| {
        let capture = app.take_screenshot();
        for chunk in capture.chunks_exact(4) {
            let color = Color::create_rgba8(
                chunk[0], chunk[1], chunk[2], chunk[3]
            );
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
            let color = Color::create_rgba8(
                chunk[0], chunk[1], chunk[2], chunk[3]
            );
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
            let color = Color::create_rgba8(
                chunk[0], chunk[1], chunk[2], chunk[3]
            );
            assert_eq!(color, Color::BLACK);
        }
    });
}
