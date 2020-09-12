/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use drying_paint::{
    Watched,
    WatchedCell,
};

use crate::pointer::{
    PointerEvent,
    PointerAction,
};
use crate::platform::{
    RenderPlatform,
    DefaultRenderPlatform,
};
use crate::widget::{
    Widget,
    WidgetId,
    WidgetContent,
    WidgetInit,
    WidgetChildReceiver,
    WidgetGraphicReceiver,
    WidgetExtra,
};
use crate::selectable::{
    Selectable,
    SelectionState,
    SelectionStateV1,
};

pub struct ToggleButtonGroup<V = ()> {
    ptr: Rc<WatchedCell<Option<V>>>,
}

impl<V> ToggleButtonGroup<V> {
    pub fn new() -> Self {
        Self {
            ptr: Rc::new(WatchedCell::new(None)),
        }
    }

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
    pub fn value(&self) -> Option<V> {
        self.ptr.get()
    }
}

impl<V> Default for ToggleButtonGroup<V> {
    fn default() -> Self { Self::new() }
}

pub trait ToggleButtonValue<V> {
    fn get_value(&self, extra: &WidgetExtra) -> V;
}

impl<T> ToggleButtonValue<()> for T {
    fn get_value(&self, _extra: &WidgetExtra) { }
}

impl<T> ToggleButtonValue<WidgetId> for T {
    fn get_value(&self, extra: &WidgetExtra) -> WidgetId {
        extra.id()
    }
}

pub struct ToggleButtonContent<T, V = ()> {
    state: Watched<SelectionState>,
    group: Watched<Option<ToggleButtonGroup<V>>>,
    allow_unselect: bool,
    interactable: Watched<bool>,
    pointers_down: usize,
    just_clicked: bool,
    currently_selected: bool,
    content: T,
}

impl<T, V> ToggleButtonContent<T, V> {
    pub fn content(&self) -> &T { &self.content }

    pub fn content_mut(&mut self) -> &mut T { &mut self.content }

    pub fn state(&self) -> SelectionState { *self.state }

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

impl<T, V, P> WidgetContent<P> for ToggleButtonContent<T, V>
where
    P: RenderPlatform,
    T: Selectable + WidgetContent<P> + ToggleButtonValue<V>,
    V: 'static + std::fmt::Debug + Copy,
{
    fn init<I: WidgetInit<Self, P>>(mut init: I) {
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
    }

    fn children<R: WidgetChildReceiver<P>>(&mut self, receiver: R) {
        self.content.children(receiver);
    }

    fn graphics<R: WidgetGraphicReceiver<P>>(&mut self, receiver: R) {
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
                let grabbed = self.hittest(extra, event.pos())
                    && event.try_grab(extra.id());
                if grabbed {
                    eprintln!("down: {:?}", self.content.get_value(&extra));
                    self.pointers_down += 1;
                    if *self.interactable {
                        *self.state = SelectionState::pressed();
                    }
                }
                grabbed
            },
            PointerAction::Move(_, _) => {
                let ungrabbed = !self.hittest(extra, event.pos())
                    && event.try_ungrab(extra);
                if ungrabbed {
                    self.pointers_down -= 1;
                    if self.pointers_down == 0 {
                        *self.state = self.base_state();
                    }
                }
                ungrabbed
            },
            PointerAction::GrabStolen => {
                self.pointers_down -= 1;
                if self.pointers_down == 0 {
                    *self.state = self.base_state();
                }
                true
            },
            PointerAction::Up => {
                let ungrabbed = event.try_ungrab(extra.id());
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
            },
            PointerAction::Hover => {
                match (self.state.v1(), self.hittest(extra, event.pos())) {
                    (SelectionStateV1::Normal, true) => {
                        let grabbed = event.try_grab(extra);
                        if grabbed && *self.interactable {
                            *self.state = SelectionState::hover();
                        }
                        grabbed
                    }
                    (SelectionStateV1::Hover, false) => {
                        let ungrabbed = event.try_ungrab(extra);
                        if ungrabbed {
                            *self.state = self.base_state();
                        }
                        ungrabbed
                    }
                    _ => false,
                }
            },
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
            content: T::default(),
        }
    }
}

pub type ToggleButton<T, V = (), P = DefaultRenderPlatform>
    = Widget<ToggleButtonContent<T, V>, P>;

