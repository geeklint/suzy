/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use suzy::{
    dims::{Padding2d, Rect},
    widget::{self, Coroutine, RunAsApp},
    widgets::Button,
};

#[derive(Default)]
struct Root {
    button: Button,
    coroutine: Coroutine<()>,
}

impl widget::Content for Root {
    fn desc(mut desc: impl widget::Desc<Self>) {
        desc.watch(|this, rect| {
            this.button.set_fill(&rect, &Padding2d::uniform(20.0));
        });
        desc.watch(|this, _rect| {
            let Self { button, coroutine } = this;
            button.on_click(|| {
                coroutine.start(());
            });
        });
        desc.register_coroutine(
            |this| &mut this.coroutine,
            |()| async {
                Coroutine::delay_secs(5.0).await;
                println!("Button clicked after delay");
            },
        );
        desc.child(|this| &mut this.button);
    }
}

fn main() {
    Root::run_as_app();
}
