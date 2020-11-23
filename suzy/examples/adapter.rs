/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

extern crate suzy;

use suzy::adapter::{
    Adaptable,
    DownwardVecAdapter,
};
use suzy::dims::{Rect, SimplePadding2d};
use suzy::watch::Watched;
use suzy::widget::*;
use suzy::platform::opengl::{
    Text,
    TextLayoutSettings,
    TextAlignment,
};

const WORDS: &str = include_str!("words.txt");

struct Element {
    value: Watched<&'static str>,
    text: Text,
}

impl Adaptable<&'static str> for Element {
    fn adapt(&mut self, data: &&'static str) {
        *self.value = data;
    }

    fn from(data: &&'static str) -> Self {
        Element {
            value: Watched::new(data),
            text: Text::default(),
        }
    }
}

impl WidgetContent for Element {
    fn init<I: WidgetInit<Self>>(mut init: I) {
        init.watch(|this, rect| {
            let text_settings = this.text.render_settings();
            text_settings.x = rect.left();
            text_settings.y = rect.center_y();
        });
        init.watch(|this, rect| {
            let text_layout = TextLayoutSettings::default()
                .wrap_width(rect.width())
                .alignment(TextAlignment::Center)
                .y_offset(-12.0);
            this.text.set_text(&this.value, text_layout);
        });
    }

    fn children<R: WidgetChildReceiver>(&mut self, _receiver: R) {
        // no children
    }

    fn graphics<R: WidgetGraphicReceiver>(&mut self, mut receiver: R) {
        receiver.graphic(&mut self.text);
    }
}

#[derive(Default)]
struct AdapterExample {
    layout: DownwardVecAdapter<&'static str, Element>,
}

impl WidgetContent for AdapterExample {
    fn init<I: WidgetInit<Self>>(mut init: I) {
        init.watch(|this, _rect| {
            this.layout.data_mut().clear();
            this.layout.data_mut().extend(WORDS.split_whitespace());
        });
        init.watch(|this, rect| {
            this.layout.set_fill(&rect, &SimplePadding2d::zero());
        });
    }

    fn children<R: WidgetChildReceiver>(&mut self, mut receiver: R) {
        receiver.child(&mut self.layout);
    }

    fn graphics<R: WidgetGraphicReceiver>(&mut self, _receiver: R) {
        // no graphics
    }
}

fn main() {
    AdapterExample::run_as_app();
}
