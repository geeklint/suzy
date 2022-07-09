/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use crate::{
    dims::Rect,
    platform::{graphics::Text, RenderPlatform},
    text::{TextPosition, TextSettings},
    watch::Watched,
    widget::{self, Widget},
};

/// A widget which displays some text
#[cfg(feature = "platform_opengl")]
pub type Label<P = crate::platforms::DefaultRenderPlatform> =
    Widget<LabelContent<P>>;

/// A widget which displays some text
#[cfg(not(feature = "platform_opengl"))]
pub type Label<P> = Widget<LabelContent<P>>;

/// The content for a widget which displays some text
pub struct LabelContent<P>
where
    P: ?Sized + RenderPlatform,
{
    text: Watched<String>,
    graphic: P::Text,
    settings: Watched<TextSettings>,
}

impl<P> Default for LabelContent<P>
where
    P: ?Sized + RenderPlatform,
{
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
    P: ?Sized + RenderPlatform,
{
    fn set_text(&mut self, text: &str) {
        *self.text = text.to_owned();
    }
}

impl<P> LabelContent<P>
where
    P: ?Sized + RenderPlatform,
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

impl<P> widget::Content<P> for LabelContent<P>
where
    P: RenderPlatform,
{
    fn desc(mut desc: impl widget::Desc<Self, P>) {
        desc.watch(|this, rect| {
            let pos = TextPosition {
                left: rect.left(),
                top: rect.top(),
                wrap_width: rect.width(),
            };
            this.graphic.set_text_rich(&this.text, &pos, &this.settings);
        });
        desc.graphic(|this| &mut this.graphic);
    }
}
