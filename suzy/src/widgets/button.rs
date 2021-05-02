/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use drying_paint::{Watched, WatchedEvent};

use crate::platform::{DefaultRenderPlatform, RenderPlatform};
use crate::pointer::{PointerAction, PointerEvent, PointerId};
use crate::selectable::{Selectable, SelectionState, SelectionStateV1};
use crate::widget::{
    Widget, WidgetChildReceiver, WidgetContent, WidgetExtra,
    WidgetGraphicReceiver, WidgetInit,
};

/// A Widget providing the behavior of a button.
pub struct ButtonBehavior<T> {
    on_click: WatchedEvent<()>,
    state: Watched<SelectionState>,
    interactable: Watched<bool>,
    pointers_down: usize,
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

    /// Returns Some(()) in a watch closure when the button is clicked.
    pub fn on_click(&self) -> Option<()> {
        self.on_click.bind().copied()
    }
}

impl<T, P> WidgetContent<P> for ButtonBehavior<T>
where
    P: RenderPlatform,
    T: Selectable + WidgetContent<P>,
{
    fn init(mut init: impl WidgetInit<Self, P>) {
        init.init_child_inline(|button| &mut button.content);
        init.watch(|button, _rect| {
            button.content.selection_changed(*button.state);
        });
        init.watch(|button, _rect| {
            if !*button.interactable {
                *button.state = SelectionState::normal();
            }
        });
    }

    fn children(&mut self, receiver: impl WidgetChildReceiver<P>) {
        self.content.children(receiver);
    }

    fn graphics(&mut self, receiver: impl WidgetGraphicReceiver<P>) {
        self.content.graphics(receiver);
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
                let grabbed =
                    self.hittest(extra, event.pos()) && event.try_grab(extra);
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
                    && event.try_ungrab(extra);
                if ungrabbed {
                    self.pointers_down -= 1;
                    if self.pointers_down == 0 {
                        *self.state = SelectionState::normal();
                    }
                }
                ungrabbed
            }
            PointerAction::GrabStolen => {
                self.pointers_down -= 1;
                if self.pointers_down == 0 {
                    *self.state = SelectionState::normal();
                }
                true
            }
            PointerAction::Up => {
                let ungrabbed = event.try_ungrab(extra.id());
                if ungrabbed {
                    self.pointers_down -= 1;
                    if self.pointers_down == 0 {
                        *self.state = if event.id() == PointerId::Mouse {
                            SelectionState::hover()
                        } else {
                            SelectionState::normal()
                        };
                        self.on_click.dispatch(());
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
            on_click: WatchedEvent::default(),
            state: Watched::default(),
            interactable: Watched::new(true),
            pointers_down: 0,
            content: T::default(),
        }
    }
}

/// A simple button.
///
/// Use `Button::on_click` like a WatchedEvent to handle button clicks
pub type Button<
    T = <DefaultRenderPlatform as RenderPlatform>::DefaultButtonContent,
    P = DefaultRenderPlatform,
> = Widget<ButtonBehavior<T>, P>;
