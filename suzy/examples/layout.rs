/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use suzy::{
    dims::Rect,
    widget::{self, RunAsApp},
    widgets::{Button, TextContent},
};

#[derive(Default)]
struct Root {
    one: Button,
    two: Button,
    three: Button,
}

impl widget::Content for Root {
    fn desc(mut desc: impl widget::Desc<Self>) {
        desc.watch(|this, _rect| {
            this.one.content_mut().set_text("One");
            this.two.content_mut().set_text("Two");
            this.three.content_mut().set_text("Three");
        });
        desc.child(|this| &mut this.one);
        desc.child(|this| &mut this.two);
        desc.child(|this| &mut this.three);
        desc.create_layout_group()
            .stack_right()
            .start_at(|this| this.left())
            .spacing(|_| 10.0)
            .push(|this| &mut this.one)
            .push(|this| &mut this.two)
            .push(|this| &mut this.three);
    }
}

fn main() {
    Root::run_as_app();
}
