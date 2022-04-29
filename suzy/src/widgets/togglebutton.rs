/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use std::rc::Rc;

use drying_paint::{Watched, WatchedCell};

use crate::platform::{DefaultRenderPlatform, RenderPlatform};
use crate::pointer::{PointerAction, PointerEvent};
use crate::selectable::{Selectable, SelectionState, SelectionStateV1};
use crate::widget::{
    self, UniqueHandle, Widget, WidgetChildReceiver, WidgetExtra,
    WidgetGraphicReceiver, WidgetInit,
};

/// A group of toggle buttons make members of the group mutually exclusive.
///
/// Toggle buttons may also relate to a value, in which case the group
/// reference can be used to retrieve the value of the currently-selected
/// ToggleButton.
pub struct ToggleButtonGroup<V = ()> {
    ptr: Rc<WatchedCell<Option<V>>>,
}

impl<V> ToggleButtonGroup<V> {
    /// Create a new toggle button group
    pub fn new() -> Self {
        Self {
            ptr: Rc::new(WatchedCell::new(None)),
        }
    }

    /// Fetch the value from the toggle group, and reset the group so that
    /// nothing is selected.
    ///
    /// This will bind a current watch function to the value of the group.
    pub fn take(&self) -> Option<V> {
        self.ptr.take()
    }

    fn set(&self, value: V) {
        self.ptr.set(Some(value));
    }

    fn unset(&self) {
        self.ptr.set(None);
    }

    fn private_clone(&self) -> Self {
        Self {
            ptr: Rc::clone(&self.ptr),
        }
    }
}

impl<V: Copy> ToggleButtonGroup<V> {
    /// Copy the value of the currently selected toggle button which belongs
    /// to this group.
    ///
    /// This will bind a current watch function to the value of the group.
    pub fn value(&self) -> Option<V> {
        self.ptr.get()
    }
}

impl<V> Default for ToggleButtonGroup<V> {
    fn default() -> Self {
        Self::new()
    }
}

/// Custom toggle button content can implement this trait to describe
/// what value is associated with which content.
pub trait ToggleButtonValue<V> {
    /// Get the value associated with this toggle button content.
    fn get_value(&self, extra: &WidgetExtra) -> V;
}

impl<T> ToggleButtonValue<()> for T {
    fn get_value(&self, _extra: &WidgetExtra) {}
}

pub struct ToggleButtonContent<T, V = ()> {
    state: Watched<SelectionState>,
    group: Watched<Option<ToggleButtonGroup<V>>>,
    allow_unselect: bool,
    interactable: Watched<bool>,
    pointers_down: usize,
    just_clicked: bool,
    currently_selected: bool,
    handle: UniqueHandle,
    content: T,
}

impl<T, V> ToggleButtonContent<T, V> {
    pub fn content(&self) -> &T {
        &self.content
    }

    pub fn content_mut(&mut self) -> &mut T {
        &mut self.content
    }

    pub fn state(&self) -> SelectionState {
        *self.state
    }

    pub fn add_to_group(&mut self, group: &ToggleButtonGroup<V>) {
        if let Some(existing) = &*self.group {
            if Rc::ptr_eq(&existing.ptr, &group.ptr) {
                return;
            }
        }
        *self.group = Some(group.private_clone());
        group.unset();
    }

    fn base_state(&self) -> SelectionState {
        if self.currently_selected {
            SelectionState::active()
        } else {
            SelectionState::normal()
        }
    }
}

impl<T, V, P> widget::Content<P> for ToggleButtonContent<T, V>
where
    P: RenderPlatform,
    T: Selectable + widget::Content<P> + ToggleButtonValue<V>,
    V: 'static + std::fmt::Debug + Copy,
{
    fn init(mut init: impl WidgetInit<Self, P>) {
        init.init_child_inline(|button| &mut button.content);
        init.watch(|button, _rect| {
            button.content.selection_changed(*button.state);
        });
        init.watch(|button, _rect| {
            if !*button.interactable {
                *button.state = button.base_state();
            }
        });
        init.watch(|button, _rect| {
            if let Some(group) = &*button.group {
                group.ptr.watched();
                // when the group changes, reset ourselves
                // unless we initiated the change
                if button.just_clicked {
                    button.just_clicked = false;
                } else if button.currently_selected {
                    let at_base = button.state == button.base_state();
                    button.currently_selected = false;
                    if at_base {
                        *button.state = button.base_state();
                    }
                }
            }
        });
        init.watch(|button, _rect| {
            let base_state = button.base_state();
            let Self {
                pointers_down,
                state,
                handle,
                ..
            } = button;
            handle.handle_pointer_grab_stolen(|_pointer_id| {
                *pointers_down -= 1;
                if *pointers_down == 0 {
                    **state = base_state;
                }
            });
        });
    }

    fn children(mut receiver: impl WidgetChildReceiver<Self, P>) {
        receiver.bare_child(|this| &mut this.content);
    }

    fn graphics(mut receiver: impl WidgetGraphicReceiver<Self, P>) {
        receiver.bare_child(|this| &mut this.content);
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
                    eprintln!("down: {:?}", self.content.get_value(extra));
                    self.pointers_down += 1;
                    if *self.interactable {
                        *self.state = SelectionState::pressed();
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
                        *self.state = self.base_state();
                    }
                }
                ungrabbed
            }
            PointerAction::Up => {
                let ungrabbed = event.try_ungrab(self.handle.id());
                if ungrabbed {
                    self.pointers_down -= 1;
                    if self.pointers_down == 0 {
                        if !self.currently_selected {
                            self.just_clicked = true;
                            if let Some(group) = &*self.group {
                                group.set(self.content.get_value(extra));
                            }
                            self.currently_selected = true;
                        } else if self.allow_unselect {
                            if let Some(group) = &*self.group {
                                group.unset();
                            }
                            self.currently_selected = false;
                        }
                        *self.state = self.base_state();
                    }
                }
                ungrabbed
            }
            PointerAction::Hover(_, _) => {
                match (self.state.v1(), self.hittest(extra, event.pos())) {
                    (SelectionStateV1::Normal, true) => {
                        let grabbed = event.try_grab(self.handle.id());
                        if grabbed && *self.interactable {
                            *self.state = SelectionState::hover();
                        }
                        grabbed
                    }
                    (SelectionStateV1::Hover, false) => {
                        let ungrabbed = event.try_ungrab(self.handle.id());
                        if ungrabbed {
                            *self.state = self.base_state();
                        }
                        ungrabbed
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }
}

impl<T: Default, V> Default for ToggleButtonContent<T, V> {
    fn default() -> Self {
        Self {
            state: Watched::default(),
            group: Watched::new(None),
            allow_unselect: true,
            interactable: Watched::new(true),
            pointers_down: 0,
            just_clicked: false,
            currently_selected: false,
            handle: UniqueHandle::default(),
            content: T::default(),
        }
    }
}

/// A button which remains in an active state after being selected.
pub type ToggleButton<T, V = (), P = DefaultRenderPlatform> =
    Widget<ToggleButtonContent<T, V>, P>;
