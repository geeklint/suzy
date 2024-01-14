/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use drying_paint::{Watched, WatchedQueue};

use crate::{
    graphics::Color,
    platform::RenderPlatform,
    pointer::{PointerAction, PointerEvent, PointerId},
    selectable::{Selectable, SelectionState, SelectionStateV1},
    widget::{self, UniqueHandle, Widget, WidgetRect},
};

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

    fn hittest(&self, rect: &WidgetRect, point: [f32; 2]) -> bool {
        self.content.hittest(rect, point)
    }

    fn pointer_event(
        &mut self,
        rect: &WidgetRect,
        event: &mut PointerEvent<'_>,
    ) -> bool {
        match event.action() {
            PointerAction::Down => {
                let grabbed = self.hittest(rect, event.pos())
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
                let ungrabbed = !self.hittest(rect, event.pos())
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
                match (self.state.v1(), self.hittest(rect, event.pos())) {
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
#[cfg(feature = "platform_opengl")]
pub type Button<
    T = DefaultButtonContent<crate::platforms::DefaultRenderPlatform>,
> = Widget<ButtonBehavior<T>>;

/// A simple button.
///
/// Use `Button::on_click` like a WatchedEvent to handle button clicks
#[cfg(not(feature = "platform_opengl"))]
pub type Button<T> = Widget<ButtonBehavior<T>>;

pub struct DefaultButtonContent<P>
where
    P: ?Sized + RenderPlatform,
{
    pub text: Watched<String>,
    image: P::SlicedImage,
    text_graphic: P::Text,
    text_color: Watched<Color>,
}

impl<P> Default for DefaultButtonContent<P>
where
    P: ?Sized + RenderPlatform,
{
    fn default() -> Self {
        Self {
            text: Watched::new("Button".to_string()),
            image: P::SlicedImage::default(),
            text_graphic: P::Text::default(),
            text_color: Watched::new(Color::WHITE),
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
        use crate::{
            platform::graphics::SlicedImage, selectable::SelectionStateV0,
        };
        match state.v0() {
            SelectionStateV0::Active => {
                *self.text_color = Color::BLACK;
                self.image.set_color(Color::LAVENDER);
            }
            _ => {
                *self.text_color = Color::LAVENDER;
                self.image.set_color(Color::SLATE_BLUE);
            }
        };
    }
}

impl<P> widget::Content<P> for DefaultButtonContent<P>
where
    P: RenderPlatform,
{
    fn desc(mut desc: impl widget::Desc<Self, P>) {
        use crate::{
            dims::{Padding2d, Rect},
            graphics::CornerStyle,
            platform::graphics::{SlicedImage, Text},
            text,
        };

        desc.watch(|this, rect| {
            this.image.set_fill(rect, &Padding2d::zero());
        });
        desc.watch(|this, rect| {
            this.text_graphic.set_layout(text::Layout {
                alignment: text::Alignment::Center,
                line: text::Line::BetweenBaseAndCap,
                flow: text::Flow::Out,
                origin_x: rect.center_x(),
                origin_y: rect.center_y(),
                wrap_width: f32::INFINITY,
                vertical_limit: text::VerticalLimit::Lines(1),
                overflow_mode: text::OverflowMode::Truncate,
            });
        });
        desc.watch(|this, _rect| {
            let style =
                crate::platform::graphics::TextStyle::with_size_and_color(
                    24.0,
                    *this.text_color,
                );
            this.text_graphic.clear();
            this.text_graphic.push_span(style, &this.text);
            this.text_graphic.finish();
        });
        desc.watch(|this, _rect| this.image.set_color(Color::ALICE_BLUE));
        desc.watch(|this, _rect| {
            this.image.set_slice_padding(Padding2d::uniform(6.0));
            this.image.set_corners(CornerStyle::Rounded);
        });
        desc.graphic(|this| &mut this.image);
        desc.graphic(|this| &mut this.text_graphic);
    }
}
