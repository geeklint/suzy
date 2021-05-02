/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::{
    OpenGlRenderPlatform, SelectableSlicedImage, Text, TextLayoutSettings,
    Texture,
};
use crate::dims::{Rect, SimplePadding2d};
use crate::graphics::Color;
use crate::platform::graphics::Text as _TextTrait;
use crate::selectable::{Selectable, SelectionState, SelectionStateV0};
use crate::text::{RichTextParser, TextAlignment, TextPosition, TextSettings};
use crate::watch::Watched;
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
    text_color: Watched<Color>,
    text: Watched<String>,
}

impl Default for DefaultOpenGlButton {
    fn default() -> Self {
        DefaultOpenGlButton {
            image: SelectableSlicedImage::default(),
            text_graphic: Text::default(),
            text_color: Watched::new(Color::WHITE),
            text: Watched::new("Button".to_string()),
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
        });
        init.watch(|this, rect| {
            let pos = TextPosition {
                left: rect.left(),
                top: rect.center_y() + 12.0,
                wrap_width: rect.width(),
            };
            let (r, g, b, _) = this.text_color.rgba();
            let settings = TextSettings {
                text_color: *this.text_color,
                alignment: TextAlignment::Center,
                outline_color: Color::create_rgba(r, g, b, 0.0),
                ..TextSettings::default()
            };
            this.text_graphic.set_text_rich(&this.text, &pos, &settings);
        });
        init.watch(|this, _rect| {
            this.image.set_image(
                Texture::from_rgba_cached(112, 37, 1, BUTTON_DATA),
                SimplePadding2d::uniform(6.0),
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
        *self.text_color = match state.v0() {
            SelectionStateV0::Active => Color::BLACK,
            _ => Color::WHITE,
        };
    }
}
