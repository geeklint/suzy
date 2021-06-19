/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use suzy::dims::{Rect, SimplePadding2d};
use suzy::widget::{
    WidgetChildReceiver, WidgetContent, WidgetGraphicReceiver, WidgetInit,
};
use suzy::widgets::Button;

#[derive(Default)]
struct Root {
    button: Button,
}

impl WidgetContent for Root {
    fn init(mut init: impl WidgetInit<Self>) {
        init.watch(|this, rect| {
            this.button.set_fill(&rect, &SimplePadding2d::uniform(20.0));
        });
    }

    fn children(&mut self, mut receiver: impl WidgetChildReceiver) {
        receiver.child(&mut self.button);
    }

    fn graphics(&mut self, _receiver: impl WidgetGraphicReceiver) {
        // no graphics
    }
}

fn main() {
    Root::run_as_app();
}
