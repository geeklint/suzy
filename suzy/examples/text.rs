/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2023 Violet Leonard */

use suzy::{
    dims::Rect,
    graphics::Color,
    platform::graphics::{Text as _TextTrait, TextStyle},
    platforms::opengl::Text,
    text,
    widget::{self, *},
};

#[derive(Default)]
struct TextExample {
    text: Text,
}

impl widget::Content for TextExample {
    fn desc(mut desc: impl widget::Desc<Self>) {
        desc.watch(|this, rect| {
            let layout = text::Layout {
                alignment: text::Alignment::Left,
                line: text::Line::Ascent,
                flow: text::Flow::Down,
                origin_x: rect.left(),
                origin_y: rect.top(),
                wrap_width: rect.width(),
                vertical_limit: text::VerticalLimit::None,
                overflow_mode: text::OverflowMode::Truncate,
            };
            this.text.set_layout(layout);
        });
        desc.watch(|this, _rect| {
            let style = TextStyle::with_size_and_color(50.0, Color::WHITE);
            this.text.clear();
            this.text.push_span(
                style,
                "The quick brown fox jumps over the lazy dog.",
            );
            this.text.finish();
        });
        desc.graphic(|this| &mut this.text);
    }
}

fn main() {
    TextExample::run_as_app();
}
