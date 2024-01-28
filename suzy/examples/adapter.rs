/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use suzy::{
    adapter::{Adaptable, DownwardVecAdapter},
    dims::{Padding2d, Rect},
    graphics::Color,
    platform::graphics::{Text as _TextTrait, TextStyle},
    platforms::opengl::Text,
    text,
    watch::Watched,
    widget::{self, RunAsApp},
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

impl widget::Content for Element {
    fn desc(mut desc: impl widget::Desc<Self>) {
        desc.watch(|this, rect| {
            let layout = text::Layout {
                alignment: text::Alignment::Center,
                line: text::Line::BetweenBaseAndCap,
                flow: text::Flow::Out,
                origin_x: rect.center_x(),
                origin_y: rect.center_y(),
                wrap_width: rect.width(),
                vertical_limit: text::VerticalLimit::Lines(1),
                overflow_mode: text::OverflowMode::Ellipsis,
            };
            this.text.set_layout(layout);
        });
        desc.watch(|this, _rect| {
            let style = TextStyle::with_size_and_color(24.0, Color::WHITE);
            this.text.clear();
            this.text.push_span(style, &this.value);
            this.text.finish();
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
            this.layout.set_fill(rect, &Padding2d::zero());
        });
        desc.child(|this| &mut this.layout);
    }
}

fn main() {
    AdapterExample::run_as_app();
}
