/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use drying_paint::{Watched, WatchedQueue};

use crate::graphics::Color;
use crate::platform::{DefaultRenderPlatform, RenderPlatform};
use crate::pointer::{PointerAction, PointerEvent, PointerId};
use crate::selectable::{Selectable, SelectionState, SelectionStateV1};
use crate::widget::{self, UniqueHandle, Widget, WidgetExtra};

const IMAGE_DATA: &[u8] = include_bytes!("button-all.data");
const IMAGE_WIDTH: u16 = 112;
const IMAGE_HEIGHT: u16 = 37;
const IMAGE_ALIGNMENT: u16 = 1;
const IMAGE_STATES: &[SelectionState] = &[
    SelectionState::normal(),
    SelectionState::hover(),
    SelectionState::focus(),
    SelectionState::active(),
];

/// A Widget providing the behavior of a button.
pub struct ButtonBehavior<T> {
    on_click: WatchedQueue<'static, ()>,
    state: Watched<SelectionState>,
    interactable: Watched<bool>,
    pointers_down: usize,
    handle: UniqueHandle,
    content: T,
}

impl<T> ButtonBehavior<T> {
    /// Get a reference to the content of this button.
    pub fn content(&self) -> &T {
        &self.content
    }

    /// Get a mutable reference to the content of this button.
    pub fn content_mut(&mut self) -> &mut T {
        &mut self.content
    }

    /// Get the current button selection state.
    pub fn state(&self) -> SelectionState {
        *self.state
    }

    pub fn on_click<F: FnOnce()>(&self, f: F) {
        crate::watch::WatchArg::try_with_current(|arg| {
            self.on_click.handle_item(arg, |()| f());
        });
    }
}

impl<T, P> widget::Content<P> for ButtonBehavior<T>
where
    T: Selectable + widget::Content<P>,
{
    fn desc(mut desc: impl widget::Desc<Self, P>) {
        desc.bare_child(|button| &mut button.content);
        desc.watch(|button, _rect| {
            button.content.selection_changed(*button.state);
        });
        desc.watch(|button, _rect| {
            if !*button.interactable {
                *button.state = SelectionState::normal();
            }
        });
        desc.watch(|button, _rect| {
            let Self {
                pointers_down,
                state,
                handle,
                ..
            } = button;
            handle.handle_pointer_grab_stolen(|_pointer_id| {
                *pointers_down -= 1;
                if *pointers_down == 0 {
                    **state = SelectionState::normal();
                }
            });
        });
    }

    fn hittest(&self, extra: &mut WidgetExtra<'_>, point: (f32, f32)) -> bool {
        self.content.hittest(extra, point)
    }

    fn pointer_event(
        &mut self,
        extra: &mut WidgetExtra<'_>,
        event: &mut PointerEvent,
    ) -> bool {
        match event.action() {
            PointerAction::Down => {
                let grabbed = self.hittest(extra, event.pos())
                    && event.try_grab(self.handle.id());
                if grabbed {
                    self.pointers_down += 1;
                    if *self.interactable {
                        *self.state = SelectionState::active();
                    }
                }
                grabbed
            }
            PointerAction::Move(_, _) => {
                let ungrabbed = !self.hittest(extra, event.pos())
                    && event.try_ungrab(self.handle.id());
                if ungrabbed {
                    self.pointers_down -= 1;
                    if self.pointers_down == 0 {
                        *self.state = SelectionState::normal();
                    }
                }
                ungrabbed
            }
            PointerAction::Up => {
                let ungrabbed = event.try_ungrab(self.handle.id());
                if ungrabbed {
                    self.pointers_down -= 1;
                    if self.pointers_down == 0 {
                        *self.state = if event.id() == PointerId::Mouse {
                            SelectionState::hover()
                        } else {
                            SelectionState::normal()
                        };
                        self.on_click.push_external(());
                    }
                }
                ungrabbed
            }
            PointerAction::Hover(_, _) => {
                match (self.state.v1(), self.hittest(extra, event.pos())) {
                    (SelectionStateV1::Normal, true) => {
                        if *self.interactable {
                            *self.state = SelectionState::hover();
                        }
                        true
                    }
                    (SelectionStateV1::Hover, false) => {
                        *self.state = SelectionState::normal();
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }
}

impl<T: Default> Default for ButtonBehavior<T> {
    fn default() -> Self {
        Self {
            on_click: WatchedQueue::default(),
            state: Watched::default(),
            interactable: Watched::new(true),
            pointers_down: 0,
            handle: UniqueHandle::default(),
            content: T::default(),
        }
    }
}

/// A simple button.
///
/// Use `Button::on_click` like a WatchedEvent to handle button clicks
pub type Button<T = DefaultButtonContent<DefaultRenderPlatform>> =
    Widget<ButtonBehavior<T>>;

pub struct DefaultButtonContent<P>
where
    P: ?Sized + RenderPlatform,
{
    image: P::SelectableSlicedImage,
    text_graphic: P::Text,
    text_color: Watched<Color>,
    text: Watched<String>,
}

impl<P> Default for DefaultButtonContent<P>
where
    P: ?Sized + RenderPlatform,
{
    fn default() -> Self {
        Self {
            image: P::SelectableSlicedImage::default(),
            text_graphic: P::Text::default(),
            text_color: Watched::new(Color::WHITE),
            text: Watched::new("Button".to_string()),
        }
    }
}

impl<P> super::TextContent for DefaultButtonContent<P>
where
    P: ?Sized + RenderPlatform,
{
    fn set_text(&mut self, text: &str) {
        *self.text = text.to_string();
    }
}

impl<P> Selectable for DefaultButtonContent<P>
where
    P: ?Sized + RenderPlatform,
{
    fn selection_changed(&mut self, state: SelectionState) {
        use crate::selectable::SelectionStateV0;
        self.image.selection_changed(state);
        *self.text_color = match state.v0() {
            SelectionStateV0::Active => Color::BLACK,
            _ => Color::WHITE,
        };
    }
}

impl<P> widget::Content<P> for DefaultButtonContent<P>
where
    P: RenderPlatform,
{
    fn desc(mut desc: impl widget::Desc<Self, P>) {
        use crate::dims::{Rect, SimplePadding2d};
        use crate::platform::graphics::{
            SelectableSlicedImage, Text, Texture,
        };
        use crate::text::{TextAlignment, TextPosition, TextSettings};

        desc.watch(|this, rect| {
            this.image.set_fill(&rect, &SimplePadding2d::zero());
        });
        desc.watch(|this, rect| {
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
        desc.watch(|this, _rect| {
            let texture = P::Texture::load_static(
                IMAGE_WIDTH,
                IMAGE_HEIGHT,
                IMAGE_ALIGNMENT,
                IMAGE_DATA,
            );
            this.image.set_image(
                texture,
                SimplePadding2d::uniform(6.0),
                IMAGE_STATES,
            );
        });
        desc.graphic(|this| &mut this.image);
        desc.graphic(|this| &mut this.text_graphic);
    }
}
