/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![cfg(feature = "platform_opengl")]

extern crate suzy;

use suzy::app::{App, AppBuilder};
use suzy::dims::{Rect, SimplePadding2d};
use suzy::graphics::Color;
use suzy::platform::opengl::{OpenGlRenderPlatform, SlicedImage};
use suzy::platform::TestPlatform;
use suzy::selectable::{Selectable, SelectionState};
use suzy::widget::{
    Widget, WidgetChildReceiver, WidgetContent, WidgetExtra,
    WidgetGraphicReceiver, WidgetInit,
};
use suzy::widgets::{ToggleButton, ToggleButtonGroup, ToggleButtonValue};
use suzy::window::WindowSettings;

mod utils;
use utils::*;

#[derive(Default)]
struct ButtonContent {
    show_image: bool,
    image: SlicedImage,
    value: i32,
}

impl Selectable for ButtonContent {
    fn selection_changed(&mut self, state: SelectionState) {
        self.show_image = state == SelectionState::active();
    }
}

impl ToggleButtonValue<i32> for ButtonContent {
    fn get_value(&self, _extra: &WidgetExtra) -> i32 {
        self.value
    }
}

impl WidgetContent<OpenGlRenderPlatform> for ButtonContent {
    fn init(mut init: impl WidgetInit<Self, OpenGlRenderPlatform>) {
        init.watch(|this, rect| {
            this.image.set_fill(&rect, &SimplePadding2d::zero());
        });
    }

    fn children(
        &mut self,
        _receiver: impl WidgetChildReceiver<OpenGlRenderPlatform>,
    ) {
    }

    fn graphics(
        &mut self,
        mut receiver: impl WidgetGraphicReceiver<OpenGlRenderPlatform>,
    ) {
        if self.show_image {
            receiver.graphic(&mut self.image);
        }
    }
}

#[derive(Default)]
struct GroupRoot {
    group: ToggleButtonGroup<i32>,
    value_feedback: std::rc::Rc<std::cell::Cell<Option<i32>>>,
    top: ToggleButton<ButtonContent, i32>,
    middle: ToggleButton<ButtonContent, i32>,
    bottom: ToggleButton<ButtonContent, i32>,
}

impl WidgetContent<OpenGlRenderPlatform> for GroupRoot {
    fn init(mut init: impl WidgetInit<Self, OpenGlRenderPlatform>) {
        init.watch(|root, _rect| {
            root.value_feedback.set(root.group.value());
        });
        init.watch(|root, _rect| {
            root.top.content_mut().value = 1;
            root.top.add_to_group(&root.group);

            root.middle.content_mut().value = 2;
            root.middle.add_to_group(&root.group);

            root.bottom.content_mut().value = 3;
            root.bottom.add_to_group(&root.group);
        });
        init.watch(|root, rect| {
            root.top.set_width(rect.width());
            root.top.set_center_x(rect.center_x());

            root.middle.set_width(rect.width());
            root.middle.set_center_x(rect.center_x());

            root.bottom.set_width(rect.width());
            root.bottom.set_center_x(rect.center_x());
        });
        init.watch(|root, rect| {
            root.top.set_height(rect.height() / 3.0);
            root.top.set_top(rect.top());

            root.middle.set_height(rect.height() / 3.0);
            root.middle.set_center_y(rect.center_y());

            root.bottom.set_height(rect.height() / 3.0);
            root.bottom.set_bottom(rect.bottom());
        });
    }

    fn children(
        &mut self,
        mut receiver: impl WidgetChildReceiver<OpenGlRenderPlatform>,
    ) {
        receiver.child(&mut self.top);
        receiver.child(&mut self.middle);
        receiver.child(&mut self.bottom);
    }

    fn graphics(
        &mut self,
        _receiver: impl WidgetGraphicReceiver<OpenGlRenderPlatform>,
    ) {
    }
}

#[test]
fn togglebutton_group() {
    let mut builder = AppBuilder::default();
    builder.set_size((480.0, 360.0));
    builder.set_background_color(Color::BLACK);
    let app: App<TestPlatform> = builder.build();
    let group_value_output = std::rc::Rc::default();
    let group_value_feedback = std::rc::Rc::clone(&group_value_output);
    let app = app
        .with(|app| {
            app.add_root(move || {
                let mut root = Widget::<GroupRoot>::default();
                root.value_feedback = group_value_feedback;
                root
            });
        })
        .0;
    app.test(|mut app| {
        let capture = app.take_screenshot();
        assert_eq!(group_value_output.get(), None);
        assert!(is_color(&capture, Color::BLACK));
        // click the bottom button
        app.mouse_click((240.0, 60.0));
        let capture = app.take_screenshot();
        let index = (capture.len() / 3) & ALIGN_MASK;
        let (bottom_3rd, top) = capture.split_at(index);
        let bottom_3rd = round_back(bottom_3rd);
        let top = round_front(top);
        assert_eq!(group_value_output.get(), Some(3));
        assert!(is_color(bottom_3rd, Color::WHITE));
        assert!(is_color(top, Color::BLACK));
        // click the top button
        app.mouse_click((240.0, 300.0));
        let capture = app.take_screenshot();
        let index = (capture.len() / 3) & ALIGN_MASK;
        let (bottom, top_3rd) = capture.split_at(2 * index);
        let bottom = round_back(bottom);
        let top_3rd = round_front(top_3rd);
        assert_eq!(group_value_output.get(), Some(1));
        assert!(is_color(bottom, Color::BLACK));
        assert!(is_color(top_3rd, Color::WHITE));
        // click the middle button
        app.mouse_click((240.0, 180.0));
        let capture = app.take_screenshot();
        let index = (capture.len() / 3) & ALIGN_MASK;
        let (bottom_3rd, top) = capture.split_at(index);
        let index = (top.len() / 2) & ALIGN_MASK;
        let (middle_3rd, top_3rd) = top.split_at(index);
        let bottom_3rd = round_back(bottom_3rd);
        let middle_3rd = round_both(middle_3rd);
        let top_3rd = round_front(top_3rd);
        assert_eq!(group_value_output.get(), Some(2));
        assert!(is_color(bottom_3rd, Color::BLACK));
        assert!(is_color(middle_3rd, Color::WHITE));
        assert!(is_color(top_3rd, Color::BLACK));
        // click the middle button again
        app.mouse_click((240.0, 180.0));
        let capture = app.take_screenshot();
        assert_eq!(group_value_output.get(), None);
        assert!(is_color(&capture, Color::BLACK));
    });
}
