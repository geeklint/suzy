/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This module contains types associated with pointer events, which are
//! considered the primary input mechanism.
//!
//! Pointer events may originate from a mouse or touchscreen.
//!
//! Each unique pointer has a concept of being "grabbed" by a widget, in which
//! case other widgets should generally ignore it.

use std::collections::HashMap;

use crate::widget::WidgetId;

#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub enum PointerId {
    Mouse,
    Touch(i64),
    Other(i64),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AltMouseButton {
    Middle,
    Right,
    X1,
    X2,
}

#[derive(Debug, Copy, Clone)]
pub enum PointerAction {
    Down,
    Move(f32, f32),
    Up,
    Wheel(f32, f32),
    AltDown(AltMouseButton),
    AltUp(AltMouseButton),
    Hover,
    GrabStolen,
}

mod internal {
    #[derive(Copy, Clone, Debug)]
    pub struct PointerEvent {
        pub id: super::PointerId,
        pub action: super::PointerAction,
        pub x: f32,
        pub y: f32,
        pub normalized: bool,
    }

    impl PointerEvent {
        pub fn new(
            id: super::PointerId,
            action: super::PointerAction,
            x: f32, y: f32,
        ) -> Self {
            Self {
                id, action, x, y, normalized: false,
            }
        }
    }
}

pub use internal::PointerEvent as PointerEventData;

pub struct PointerEvent<'a> {
    data: PointerEventData,
    grab_map: &'a mut HashMap<PointerId, WidgetId>,
    pub(crate) grab_stolen_from: Option<WidgetId>,
}

impl std::fmt::Debug for PointerEvent<'_> {
    fn fmt(&self, fmtter: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.data, fmtter)
    }
}

impl<'a> PointerEvent<'a> {
    pub(crate) fn new(
        data: PointerEventData,
        grab_map: &'a mut HashMap<PointerId, WidgetId>,
    ) -> Self {
        PointerEvent {
            data,
            grab_map,
            grab_stolen_from: None,
        }
    }
}

impl PointerEvent<'_> {

    pub fn id(&self) -> PointerId { self.data.id }

    pub fn action(&self) -> &PointerAction { &self.data.action }

    pub fn x(&self) -> f32 { self.data.x }

    pub fn y(&self) -> f32 { self.data.y }

    pub fn pos(&self) -> (f32, f32) { (self.data.x, self.data.y) }

    pub fn try_grab<I>(&mut self, holder: I) -> bool
        where I: Into<WidgetId>
    {
        use std::collections::hash_map::Entry::*;

        let wid = holder.into();
        match self.grab_map.entry(self.id()) {
            Occupied(entry) => (wid == *entry.get()),
            Vacant(entry) => {
                entry.insert(wid);
                true
            }
        }
    }

    pub fn is_grabbed_by<I>(&self, holder: I) -> bool
        where I: Into<WidgetId>
    {
        self.grab_map.get(&self.id()).map_or(false, |v| v == &holder.into())
    }

    pub fn try_ungrab<I>(&mut self, holder: I) -> bool
        where I: Into<WidgetId>
    {
        use std::collections::hash_map::Entry::*;

        let wid = holder.into();
        match self.grab_map.entry(self.id()) {
            Occupied(entry) => {
                if wid == *entry.get() {
                    entry.remove();
                    true
                } else {
                    false
                }
            }
            Vacant(_entry) => false
        }
    }
}
