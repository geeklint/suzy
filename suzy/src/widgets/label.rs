/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use crate::{
    dims::Rect,
    graphics::Color,
    platform::{graphics::Text, RenderPlatform},
    text,
    watch::Watched,
    widget::{self, Widget},
};

/// A widget which displays some text
#[cfg(feature = "platform-opengl")]
pub type Label<P = crate::platforms::DefaultRenderPlatform> =
    Widget<LabelContent<P>>;

/// A widget which displays some text
#[cfg(not(feature = "platform-opengl"))]
pub type Label<P> = Widget<LabelContent<P>>;

/// The content for a widget which displays some text
pub struct LabelContent<P>
where
    P: ?Sized + RenderPlatform,
{
    pub text: Watched<String>,
    pub font_size: Watched<f32>,
    pub color: Watched<Color>,
    pub layout: Watched<text::Layout>,
    graphic: P::Text,
}

impl<P> Default for LabelContent<P>
where
    P: ?Sized + RenderPlatform,
{
    fn default() -> Self {
        Self {
            text: Watched::default(),
            font_size: Watched::new(16.0),
            color: Watched::new(Color::BLACK),
            layout: Watched::new(text::Layout {
                alignment: text::Alignment::Left,
                line: text::Line::Ascent,
                flow: text::Flow::Down,
                origin_x: 0.0,
                origin_y: 1.0,
                wrap_width: 1.0,
                vertical_limit: text::VerticalLimit::None,
                overflow_mode: text::OverflowMode::Truncate,
            }),
            graphic: P::Text::default(),
        }
    }
}

impl<P> super::TextContent for LabelContent<P>
where
    P: ?Sized + RenderPlatform,
{
    fn set_text(&mut self, text: &str) {
        text.clone_into(&mut self.text);
    }
}

impl<P> widget::Content<P> for LabelContent<P>
where
    P: RenderPlatform,
{
    fn desc(mut desc: impl widget::Desc<Self, P>) {
        desc.watch(|this, rect| {
            use crate::animation::Lerp;
            let mut layout = *this.layout;
            layout.origin_x =
                f32::lerp(&rect.left(), &rect.right(), layout.origin_x);
            layout.origin_y =
                f32::lerp(&rect.bottom(), &rect.top(), layout.origin_y);
            layout.wrap_width *= rect.width();
            this.graphic.set_layout(layout);
        });
        desc.watch(|this, _rect| {
            let style =
                crate::platform::graphics::TextStyle::with_size_and_color(
                    *this.font_size,
                    *this.color,
                );
            this.graphic.clear();
            this.graphic.push_span(style, &this.text);
            this.graphic.finish();
        });
        desc.graphic(|this| &mut this.graphic);
    }
}
