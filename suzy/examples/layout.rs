/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use suzy::dims::Rect;
use suzy::widget::{
    self, RunAsApp, WidgetChildReceiver, WidgetGraphicReceiver, WidgetInit,
};
use suzy::widgets::{Button, TextContent};

#[derive(Default)]
struct Root {
    one: Button,
    two: Button,
    three: Button,
}

impl widget::Content for Root {
    fn init(mut init: impl WidgetInit<Self>) {
        init.watch(|this, _rect| {
            this.one.content_mut().set_text("One");
            this.two.content_mut().set_text("Two");
            this.three.content_mut().set_text("Three");
        });
        init.create_layout_group()
            .stack_right()
            .start_at(|this| this.left())
            .spacing(|_| 10.0)
            .push(|this| &mut this.one)
            .push(|this| &mut this.two)
            .push(|this| &mut this.three);
    }

    fn children(mut receiver: impl WidgetChildReceiver<Self>) {
        receiver.child(|this| &mut this.one);
        receiver.child(|this| &mut this.two);
        receiver.child(|this| &mut this.three);
    }

    fn graphics(&mut self, _receiver: impl WidgetGraphicReceiver) {
        // no graphics
    }
}

fn main() {
    Root::run_as_app();
}
