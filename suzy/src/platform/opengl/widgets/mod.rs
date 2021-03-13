/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::{
    OpenGlRenderPlatform, SelectableSlicedImage, Text, TextAlignment,
    TextLayoutSettings, Texture,
};
use crate::dims::{Rect, SimplePadding2d};
use crate::graphics::Color;
use crate::selectable::{Selectable, SelectionState, SelectionStateV0};
use crate::widget::{
    WidgetChildReceiver, WidgetContent, WidgetGraphicReceiver, WidgetInit,
};
use crate::widgets::TextContent;

const BUTTON_DATA: &[u8] = include_bytes!("button-all.data");
const BUTTON_STATES: &[SelectionState] = &[
    SelectionState::normal(),
    SelectionState::hover(),
    SelectionState::focus(),
    SelectionState::active(),
];

type Plat = OpenGlRenderPlatform;

pub struct DefaultOpenGlButton {
    image: SelectableSlicedImage,
    text_graphic: Text,
    text: crate::watch::Watched<String>,
}

impl Default for DefaultOpenGlButton {
    fn default() -> Self {
        DefaultOpenGlButton {
            image: SelectableSlicedImage::default(),
            text_graphic: Text::default(),
            text: crate::watch::Watched::new("Button".to_string()),
        }
    }
}

impl TextContent for DefaultOpenGlButton {
    fn set_text(&mut self, text: &str) {
        *self.text = text.to_string();
    }
}

impl WidgetContent<Plat> for DefaultOpenGlButton {
    fn init(mut init: impl WidgetInit<Self, Plat>) {
        init.watch(|this, rect| {
            this.image.set_fill(&rect, &SimplePadding2d::zero());

            let text_settings = this.text_graphic.render_settings();
            text_settings.x = rect.left();
            text_settings.y = rect.center_y();
        });
        init.watch(|this, rect| {
            let text_layout = TextLayoutSettings::default()
                .wrap_width(rect.width())
                .alignment(TextAlignment::Center)
                .y_offset(-12.0);
            this.text_graphic.set_text(&this.text, text_layout);
        });
        init.watch(|this, _rect| {
            this.image.set_image(
                Texture::from_rgba_cached(112, 37, 1, BUTTON_DATA),
                &SimplePadding2d::uniform(6.0),
                BUTTON_STATES,
            );
        });
    }

    fn children(&mut self, _receiver: impl WidgetChildReceiver<Plat>) {
        // no children
    }

    fn graphics(&mut self, mut receiver: impl WidgetGraphicReceiver<Plat>) {
        receiver.graphic(&mut self.image);
        receiver.graphic(&mut self.text_graphic);
    }
}

impl Selectable for DefaultOpenGlButton {
    fn selection_changed(&mut self, state: SelectionState) {
        self.image.selection_changed(state);
        let text_settings = self.text_graphic.render_settings();
        text_settings.text_color = match state.v0() {
            SelectionStateV0::Active => Color::BLACK,
            _ => Color::WHITE,
        };
        let (r, g, b, _) = text_settings.text_color.rgba();
        text_settings.outline_color = Color::create_rgba(r, g, b, 0.0);
    }
}
