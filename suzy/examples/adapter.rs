/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

extern crate suzy;

use suzy::adapter::{Adaptable, DownwardVecAdapter};
use suzy::dims::{Rect, SimplePadding2d};
use suzy::platform::graphics::Text as _TextTrait;
use suzy::platforms::opengl::Text;
use suzy::text::{RichTextCommand, TextAlignment, TextPosition, TextSettings};
use suzy::watch::Watched;
use suzy::widget::*;

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
    fn init(mut init: impl WidgetInit<Self>) {
        init.watch(|this, rect| {
            let pos = TextPosition {
                left: rect.left(),
                top: rect.center_y() + 12.0,
                wrap_width: rect.width(),
            };
            let mut settings = TextSettings::default();
            settings.alignment = TextAlignment::Center;
            this.text.set_text_plain(&*this.value, &pos, &settings);
        });
    }

    fn children(&mut self, _receiver: impl WidgetChildReceiver) {
        // no children
    }

    fn graphics(&mut self, mut receiver: impl WidgetGraphicReceiver) {
        receiver.graphic(&mut self.text);
    }
}

#[derive(Default)]
struct AdapterExample {
    layout: DownwardVecAdapter<&'static str, Element>,
}

impl WidgetContent for AdapterExample {
    fn init(mut init: impl WidgetInit<Self>) {
        init.watch(|this, _rect| {
            this.layout.data_mut().clear();
            this.layout.data_mut().extend(WORDS.split_whitespace());
        });
        init.watch(|this, rect| {
            this.layout.set_fill(&rect, &SimplePadding2d::zero());
        });
    }

    fn children(&mut self, mut receiver: impl WidgetChildReceiver) {
        receiver.child(&mut self.layout);
    }

    fn graphics(&mut self, _receiver: impl WidgetGraphicReceiver) {
        // no graphics
    }
}

fn main() {
    AdapterExample::run_as_app();
}
