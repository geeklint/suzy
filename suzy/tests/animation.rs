/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

extern crate suzy;

use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use suzy::animation::Animation;
use suzy::app::{App, AppBuilder};
use suzy::graphics::Color;
use suzy::platform::{TestPlatform, TestRenderPlatform};
use suzy::widget::{
    self, Widget, WidgetChildReceiver, WidgetGraphicReceiver, WidgetInit,
};
use suzy::window::WindowSettings;

#[derive(Default)]
struct Root {
    anim: Animation<f32>,
    value_feedback: Rc<Cell<f32>>,
}

impl widget::Content<TestRenderPlatform> for Root {
    fn init(mut init: impl WidgetInit<Self, TestRenderPlatform>) {
        init.watch(|root, _rect| {
            let mut value = root.value_feedback.get();
            root.anim.apply(&mut value);
            root.value_feedback.set(value);
        });
        init.watch(|root, _rect| {
            root.anim.set_duration(Duration::from_secs(1));
            root.anim.animate_to(261.0);
        });
    }

    fn children(
        _receiver: impl WidgetChildReceiver<Self, TestRenderPlatform>,
    ) {
    }

    fn graphics(
        &mut self,
        _receiver: impl WidgetGraphicReceiver<TestRenderPlatform>,
    ) {
    }
}

#[test]
fn animation() {
    let mut builder = AppBuilder::default();
    builder.set_size((480.0, 360.0));
    builder.set_background_color(Color::BLACK);
    let app: App<TestPlatform> = builder.build();
    let value_output = Rc::new(Cell::new(142.0));
    let value_feedback = Rc::clone(&value_output);
    let app = app
        .with(|app| {
            app.add_root(move || {
                let mut root = Widget::<Root>::default();
                root.value_feedback = value_feedback;
                root
            });
        })
        .0;
    app.test(|mut app| {
        assert!(
            (value_output.get() - 142.0).abs() < f32::EPSILON,
            "value is {}",
            value_output.get(),
        );
        app.next_frame(Duration::from_millis(100));
        app.draw_if_needed();
        assert!(
            (value_output.get() - 153.9).abs() < f32::EPSILON,
            "value is {}",
            value_output.get(),
        );
        app.next_frame(Duration::from_millis(226));
        app.draw_if_needed();
        assert!(
            (value_output.get() - 180.794).abs() < f32::EPSILON,
            "value is {}",
            value_output.get(),
        );
        app.next_frame(Duration::from_millis(195));
        app.draw_if_needed();
        assert!(
            (value_output.get() - 203.99901).abs() < f32::EPSILON,
            "value is {}",
            value_output.get(),
        );
        app.next_frame(Duration::from_millis(407));
        app.draw_if_needed();
        assert!(
            (value_output.get() - 252.43199).abs() < f32::EPSILON,
            "value is {}",
            value_output.get(),
        );
        app.next_frame(Duration::from_millis(72));
        app.draw_if_needed();
        assert!(
            (value_output.get() - 261.0).abs() < f32::EPSILON,
            "value is {}",
            value_output.get(),
        );
    });
}
