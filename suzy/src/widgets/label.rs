/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use crate::dims::Rect;
use crate::platform::graphics::Text;
use crate::platform::{DefaultRenderPlatform, RenderPlatform};
use crate::text::{TextPosition, TextSettings};
use crate::watch::Watched;
use crate::widget::{
    Widget, WidgetChildReceiver, WidgetContent, WidgetGraphicReceiver,
    WidgetInit,
};

/// A widget which displays some text
pub type Label<P = DefaultRenderPlatform> = Widget<LabelContent<P>>;

/// The content for a widget which displays some text
pub struct LabelContent<P: RenderPlatform> {
    text: Watched<String>,
    graphic: P::Text,
    settings: Watched<TextSettings>,
}

impl<P: RenderPlatform> Default for LabelContent<P> {
    fn default() -> Self {
        Self {
            text: Watched::default(),
            graphic: P::Text::default(),
            settings: Watched::default(),
        }
    }
}

impl<P> super::TextContent for LabelContent<P>
where
    P: RenderPlatform,
{
    fn set_text(&mut self, text: &str) {
        *self.text = text.to_owned();
    }
}

impl<P> LabelContent<P>
where
    P: RenderPlatform,
{
    /// Get a reference to the current text settings
    pub fn settings(&self) -> &TextSettings {
        &*self.settings
    }

    /// Get a mutable reference to the current text settings
    pub fn settings_mut(&mut self) -> &mut TextSettings {
        &mut *self.settings
    }
}

impl<P> WidgetContent<P> for LabelContent<P>
where
    P: RenderPlatform,
{
    fn init(mut init: impl WidgetInit<Self, P>) {
        init.watch(|this, rect| {
            let pos = TextPosition {
                left: rect.left(),
                top: rect.top(),
                wrap_width: rect.width(),
            };
            this.graphic.set_text_rich(&this.text, &pos, &this.settings);
        });
    }

    fn children(_receiver: impl WidgetChildReceiver<Self, P>) {
        // no children
    }

    fn graphics(&mut self, mut receiver: impl WidgetGraphicReceiver<P>) {
        receiver.graphic(&mut self.graphic);
    }
}
