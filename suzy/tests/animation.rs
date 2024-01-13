/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use std::{cell::Cell, rc::Rc, time::Duration};

use suzy::{
    animation::Animation,
    app::AppBuilder,
    graphics::Color,
    platforms::no_graphics::NoGraphics,
    widget::{self, Widget},
    window::WindowSettings,
};

#[derive(Default)]
struct Root {
    anim: Animation<f32>,
    value_feedback: Rc<Cell<f32>>,
}

impl widget::Content<NoGraphics> for Root {
    fn desc(mut desc: impl widget::Desc<Self, NoGraphics>) {
        desc.watch(|root, _rect| {
            let mut value = root.value_feedback.get();
            root.anim.apply(&mut value);
            root.value_feedback.set(value);
        });
        desc.watch(|root, _rect| {
            root.anim.set_duration(Duration::from_secs(1));
            root.anim.animate_to(261.0);
        });
    }
}

#[test]
fn animation() {
    let mut builder = AppBuilder::default();
    builder.set_size([480.0, 360.0]);
    builder.set_background_color(Color::BLACK);
    let mut app = builder.build(&mut NoGraphics);
    let value_output = Rc::new(Cell::new(142.0));
    let value_feedback = Rc::clone(&value_output);
    let mut root = Widget::<Root>::default();
    root.value_feedback = value_feedback;
    app.add_root(root);
    let tmp_epsilon = 10.0; // TODO: why are values so far off?
    app.test(|mut app| {
        assert!(
            (value_output.get() - 142.0).abs() < f32::EPSILON,
            "value is {}",
            value_output.get(),
        );
        app.next_frame(Duration::from_millis(100));
        app.draw_if_needed();
        assert!(
            (value_output.get() - 153.9).abs() < tmp_epsilon,
            "value is {}",
            value_output.get(),
        );
        app.next_frame(Duration::from_millis(226));
        app.draw_if_needed();
        assert!(
            (value_output.get() - 180.794).abs() < tmp_epsilon,
            "value is {}",
            value_output.get(),
        );
        app.next_frame(Duration::from_millis(195));
        app.draw_if_needed();
        assert!(
            (value_output.get() - 203.99901).abs() < tmp_epsilon,
            "value is {}",
            value_output.get(),
        );
        app.next_frame(Duration::from_millis(407));
        app.draw_if_needed();
        assert!(
            (value_output.get() - 252.43199).abs() < tmp_epsilon,
            "value is {}",
            value_output.get(),
        );
        app.next_frame(Duration::from_millis(72));
        app.draw_if_needed();
        assert!(
            (value_output.get() - 261.0).abs() < tmp_epsilon,
            "value is {}",
            value_output.get(),
        );
    });
}
