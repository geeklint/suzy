/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

//! Pointer events are considered the primary input mechanism.
//!
//! Pointer events may originate from a mouse or touchscreen.
//!
//! Each unique pointer has a concept of being "grabbed" by a widget, in which
//! case other widgets should generally ignore it.

use std::collections::HashMap;

use crate::widget::UniqueHandleId;

/// A unique id for a particular pointer
#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub enum PointerId {
    /// Mouse device
    Mouse,

    /// A specific touch on a touchscreen
    Touch(i64),

    /// Other; for instance a simulated event not corosponding to a real
    /// device.
    Other(i64),
}

/// An enum for possible mouse buttons used besides the primary (left).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AltMouseButton {
    /// Middle mouse button.
    Middle,

    /// Right mouse button.
    Right,

    /// Mouse button X1.
    X1,

    /// Mouse button X2.
    X2,
}

/// An enum describing the activity that generated a particular pointer event
#[derive(Debug, Copy, Clone)]
pub enum PointerAction {
    /// The pointer was pressed.
    Down,

    /// The pointer moved while it was held down.
    ///
    /// The parameters indicate the relative change in position.
    Move(f32, f32),

    /// The pointer was released.
    Up,

    /// For mice, the scroll wheel was moved.
    ///
    /// The parameters describe the amount scrolled.
    Wheel(f32, f32),

    /// For mice, an alterntive button (besides the left) was pressed.
    AltDown(AltMouseButton),

    /// For mice, an alterntive button (besides the left) was released.
    AltUp(AltMouseButton),

    /// The pointer moved while not considered held down.
    ///
    /// The parameters indicate the relative change in position.
    Hover(f32, f32),
}

mod internal {
    /// The data associated with a particular pointer event.
    #[derive(Copy, Clone, Debug)]
    pub struct PointerEvent {
        /// The pointer involved in this event
        pub id: super::PointerId,
        /// The activity that caused this event
        pub action: super::PointerAction,
        /// The horizontal position of the pointer
        pub x: f32,
        /// The vertical position of the pointer
        pub y: f32,
    }
}

pub use internal::PointerEvent as PointerEventData;

/// This struct will get passed to [`crate::widget::Content::pointer_event`] method.
pub struct PointerEvent<'a> {
    data: PointerEventData,
    grab_map: &'a mut HashMap<PointerId, UniqueHandleId>,
}

impl std::fmt::Debug for PointerEvent<'_> {
    fn fmt(&self, fmtter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.data, fmtter)
    }
}

impl<'a> PointerEvent<'a> {
    pub(crate) fn new(
        data: PointerEventData,
        grab_map: &'a mut HashMap<PointerId, UniqueHandleId>,
    ) -> Self {
        PointerEvent { data, grab_map }
    }
}

impl PointerEvent<'_> {
    /// Get the pointer involved in this event
    #[must_use]
    pub fn id(&self) -> PointerId {
        self.data.id
    }

    /// Get the activity which triggered this event
    #[must_use]
    pub fn action(&self) -> &PointerAction {
        &self.data.action
    }

    /// Get the horizontal position of the pointer during this event
    #[must_use]
    pub fn x(&self) -> f32 {
        self.data.x
    }

    /// Get the vertical position of the pointer during this event
    #[must_use]
    pub fn y(&self) -> f32 {
        self.data.y
    }

    /// Get the position of the pointer during this event
    #[must_use]
    pub fn pos(&self) -> [f32; 2] {
        [self.data.x, self.data.y]
    }

    /// Try to "grab" the pointer, indicating that the identified handle
    /// should be the primary handler of this pointer.
    ///
    /// Returns false if a different handle has already grabbed the pointer.
    pub fn try_grab<I>(&mut self, holder: I) -> bool
    where
        I: Into<UniqueHandleId>,
    {
        use std::collections::hash_map::Entry::{Occupied, Vacant};

        let wid = holder.into();
        match self.grab_map.entry(self.id()) {
            Occupied(entry) => wid == *entry.get(),
            Vacant(entry) => {
                entry.insert(wid);
                true
            }
        }
    }

    /// Focibly "grab" the pointer, indicating that the identified handle
    /// should be the primary handler of this pointer.
    ///
    /// The handle previously grabbing the pointer will be notified with
    /// [`crate::widget::UniqueHandle::handle_pointer_grab_stolen`].
    pub fn force_grab<I>(&mut self, holder: I)
    where
        I: Into<UniqueHandleId>,
    {
        let handle_id = holder.into();
        let prev = self.grab_map.insert(self.id(), handle_id);
        if let Some(prev_handle_id) = prev {
            prev_handle_id.notify_grab_stolen(self.id());
        }
    }

    /// Check if this event is grabbed by the identified handle.
    pub fn is_grabbed_by<I>(&self, holder: I) -> bool
    where
        I: Into<UniqueHandleId>,
    {
        self.grab_map
            .get(&self.id())
            .is_some_and(|v| v == &holder.into())
    }

    /// Try to stop grabbing this pointer, indicating that the identified
    /// widget should no longer be considered the primary handler.
    ///
    /// Returns false if the grab was not previously held by the
    /// identified handle.
    pub fn try_ungrab<I>(&mut self, holder: I) -> bool
    where
        I: Into<UniqueHandleId>,
    {
        use std::collections::hash_map::Entry::{Occupied, Vacant};

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
            Vacant(_entry) => false,
        }
    }
}
