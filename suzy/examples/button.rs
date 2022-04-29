/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use suzy::dims::{Rect, SimplePadding2d};
use suzy::widget::{self, RunAsApp, WidgetDescReceiver, WidgetInit};
use suzy::widgets::Button;

#[derive(Default)]
struct Root {
    button: Button,
}

impl widget::Content for Root {
    fn init(mut init: impl WidgetInit<Self>) {
        init.watch(|this, rect| {
            this.button.set_fill(&rect, &SimplePadding2d::uniform(20.0));
        });
    }

    fn desc(mut receiver: impl WidgetDescReceiver<Self>) {
        receiver.child(|this| &mut this.button);
    }
}

fn main() {
    Root::run_as_app();
}
