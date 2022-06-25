/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

extern crate suzy;

use suzy::adapter::{Adaptable, DownwardVecAdapter};
use suzy::dims::{Rect, SimplePadding2d};
use suzy::platform::graphics::Text as _TextTrait;
use suzy::platforms::opengl::Text;
use suzy::text::{TextAlignment, TextPosition, TextSettings};
use suzy::watch::Watched;
use suzy::widget::{self, *};

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

impl widget::Content for Element {
    fn desc(mut desc: impl widget::Desc<Self>) {
        desc.watch(|this, rect| {
            let pos = TextPosition {
                left: rect.left(),
                top: rect.center_y() + 12.0,
                wrap_width: rect.width(),
            };
            let mut settings = TextSettings::default();
            settings.alignment = TextAlignment::Center;
            this.text.set_text_plain(*this.value, &pos, &settings);
        });
        desc.graphic(|this| &mut this.text);
    }
}

#[derive(Default)]
struct AdapterExample {
    layout: DownwardVecAdapter<&'static str, Element>,
}

impl widget::Content for AdapterExample {
    fn desc(mut desc: impl widget::Desc<Self>) {
        desc.watch(|this, _rect| {
            this.layout.data_mut().clear();
            this.layout.data_mut().extend(WORDS.split_whitespace());
        });
        desc.watch(|this, rect| {
            this.layout.set_fill(&rect, &SimplePadding2d::zero());
        });
        desc.child(|this| &mut this.layout);
    }
}

fn main() {
    AdapterExample::run_as_app();
}
