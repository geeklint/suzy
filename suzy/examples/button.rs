/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use suzy::{
    dims::Rect,
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
            this.button.set_center(rect.center());
            this.button.set_width(200.0);
            this.button.set_height(50.0);
        });
        desc.child(|this| &mut this.button);
    }
}

fn main() {
    Root::run_as_app();
}
