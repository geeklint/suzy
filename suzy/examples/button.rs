/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use suzy::{
    dims::{Padding2d, Rect},
    widget::{self, RunAsApp},
    widgets::Button,
};

#[derive(Default)]
struct Root {
    button: Button,
}

impl widget::Content for Root {
    fn desc(mut desc: impl widget::Desc<Self>) {
        desc.watch(|this, rect| {
            this.button.set_fill(rect, &Padding2d::uniform(20.0));
        });
        desc.child(|this| &mut this.button);
    }
}

fn main() {
    Root::run_as_app();
}
