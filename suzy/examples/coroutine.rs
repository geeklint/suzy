/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use suzy::dims::{Rect, SimplePadding2d};
use suzy::widget::{
    WidgetChildReceiver, WidgetContent, WidgetGraphicReceiver, WidgetInit,
    Coroutine,
};
use suzy::widgets::Button;

#[derive(Default)]
struct Root {
    button: Button,
    coroutine: Coroutine<()>,
}

impl WidgetContent for Root {
    fn init(mut init: impl WidgetInit<Self>) {
        init.watch(|this, rect| {
            this.button.set_fill(&rect, &SimplePadding2d::uniform(20.0));
        });
        init.watch(|this, _rect| {
            if let Some(()) = this.button.on_click() {
                this.coroutine.start(());
            }
        });
        init.register_coroutine(|this| &mut this.coroutine, |()| async {
            Coroutine::delay_secs(5.0).await;
            println!("Button clicked after delay");
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
