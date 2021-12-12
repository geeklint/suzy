/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use suzy::dims::{Rect, SimplePadding2d};
use suzy::widget::{
    Coroutine, WidgetChildReceiver, WidgetContent, WidgetGraphicReceiver,
    WidgetInit,
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
        init.register_coroutine(
            |this| &mut this.coroutine,
            |()| async {
                Coroutine::delay_secs(5.0).await;
                println!("Button clicked after delay");
            },
        );
    }

    fn children(mut receiver: impl WidgetChildReceiver<Self>) {
        receiver.child(|this| &mut this.button);
    }

    fn graphics(&mut self, _receiver: impl WidgetGraphicReceiver) {
        // no graphics
    }
}

fn main() {
    Root::run_as_app();
}
