/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use suzy::dims::Rect;
use suzy::widget::{
    WidgetChildReceiver, WidgetContent, WidgetGraphicReceiver, WidgetInit,
};
use suzy::widgets::{Button, TextContent};

#[derive(Default)]
struct Root {
    one: Button,
    two: Button,
    three: Button,
}

impl WidgetContent for Root {
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

    fn children(&mut self, mut receiver: impl WidgetChildReceiver) {
        receiver.child(&mut self.one);
        receiver.child(&mut self.two);
        receiver.child(&mut self.three);
    }

    fn graphics(&mut self, _receiver: impl WidgetGraphicReceiver) {
        // no graphics
    }
}

fn main() {
    Root::run_as_app();
}
