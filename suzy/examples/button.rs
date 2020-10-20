/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use suzy::dims::{Rect, SimplePadding2d};
use suzy::widget::{
    WidgetContent,
    WidgetInit,
    WidgetChildReceiver,
    WidgetGraphicReceiver,
};
use suzy::widgets::Button;

#[derive(Default)]
struct Root {
    button: Button,
}

impl WidgetContent for Root {
    fn init<I: WidgetInit<Self>>(mut init: I) {
        init.watch(|this, rect| {
            this.button.set_fill(&rect, &SimplePadding2d::uniform(20.0));
        });
    }

    fn children<R: WidgetChildReceiver>(&mut self, mut receiver: R) {
        receiver.child(&mut self.button);
    }

    fn graphics<R: WidgetGraphicReceiver>(&mut self, _receiver: R) {
        // no graphics
    }
}

fn main() {
    Root::run_as_app();
}