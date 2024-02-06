/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

#![cfg(any(feature = "platform_osmesa", feature = "platform_sdl"))]

use suzy::{
    app::{App, AppTestingExt},
    dims::{Padding2d, Rect},
    graphics::{Color, Conditional},
    platforms::{
        opengl::{OpenGlRenderPlatform, SlicedImage},
        TestPlatform,
    },
    selectable::{Selectable, SelectionState},
    widget::{self, Widget, WidgetRect},
    widgets::{ToggleButton, ToggleButtonGroup, ToggleButtonValue},
};

mod utils;
use utils::*;

#[derive(Default)]
struct ButtonContent {
    image: Conditional<SlicedImage>,
    value: i32,
}

impl Selectable for ButtonContent {
    fn selection_changed(&mut self, state: SelectionState) {
        self.image.enable = state == SelectionState::active();
    }
}

impl ToggleButtonValue<i32> for ButtonContent {
    fn get_value(&self, _extra: &WidgetRect) -> i32 {
        self.value
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
struct GroupRoot {
    group: ToggleButtonGroup<i32>,
    value_feedback: std::rc::Rc<std::cell::Cell<Option<i32>>>,
    top: ToggleButton<ButtonContent, i32>,
    middle: ToggleButton<ButtonContent, i32>,
    bottom: ToggleButton<ButtonContent, i32>,
}

impl widget::Content<OpenGlRenderPlatform> for GroupRoot {
    fn desc(mut desc: impl widget::Desc<Self, OpenGlRenderPlatform>) {
        desc.watch(|root, _rect| {
            root.value_feedback.set(root.group.value());
        });
        desc.watch(|root, _rect| {
            root.top.content_mut().value = 1;
            root.top.add_to_group(&root.group);

            root.middle.content_mut().value = 2;
            root.middle.add_to_group(&root.group);

            root.bottom.content_mut().value = 3;
            root.bottom.add_to_group(&root.group);
        });
        desc.watch(|root, rect| {
            root.top.set_width(rect.width());
            root.top.set_center_x(rect.center_x());

            root.middle.set_width(rect.width());
            root.middle.set_center_x(rect.center_x());

            root.bottom.set_width(rect.width());
            root.bottom.set_center_x(rect.center_x());
        });
        desc.watch(|root, rect| {
            root.top.set_height(rect.height() / 3.0);
            root.top.set_top(rect.top());

            root.middle.set_height(rect.height() / 3.0);
            root.middle.set_center_y(rect.center_y());

            root.bottom.set_height(rect.height() / 3.0);
            root.bottom.set_bottom(rect.bottom());
        });
        desc.child(|this| &mut this.top);
        desc.child(|this| &mut this.middle);
        desc.child(|this| &mut this.bottom);
    }
}

#[test]
fn togglebutton_group() {
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
    let mut window = app.screenshot_tmp();
    let group_value_output = std::rc::Rc::default();
    let group_value_feedback = std::rc::Rc::clone(&group_value_output);
    let mut root = Widget::<GroupRoot>::default();
    root.value_feedback = group_value_feedback;
    app.add_root(root);
    let capture = window.draw_and_take_screenshot(&mut app);
    assert_eq!(group_value_output.get(), None);
    assert!(is_color(&capture, Color::BLACK));
    // click the bottom button
    app.mouse_click([240.0, 60.0]);
    let capture = window.draw_and_take_screenshot(&mut app);
    let index = (capture.len() / 3) & ALIGN_MASK;
    let (bottom_3rd, top) = capture.split_at(index);
    let bottom_3rd = round_back(bottom_3rd);
    let top = round_front(top);
    assert_eq!(group_value_output.get(), Some(3));
    assert!(is_color(bottom_3rd, Color::WHITE));
    assert!(is_color(top, Color::BLACK));
    // click the top button
    app.mouse_click([240.0, 300.0]);
    let capture = window.draw_and_take_screenshot(&mut app);
    let index = (capture.len() / 3) & ALIGN_MASK;
    let (bottom, top_3rd) = capture.split_at(2 * index);
    let bottom = round_back(bottom);
    let top_3rd = round_front(top_3rd);
    assert_eq!(group_value_output.get(), Some(1));
    assert!(is_color(bottom, Color::BLACK));
    assert!(is_color(top_3rd, Color::WHITE));
    // click the middle button
    app.mouse_click([240.0, 180.0]);
    let capture = window.draw_and_take_screenshot(&mut app);
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
    app.mouse_click([240.0, 180.0]);
    let capture = window.draw_and_take_screenshot(&mut app);
    assert_eq!(group_value_output.get(), None);
    assert!(is_color(&capture, Color::BLACK));
}
