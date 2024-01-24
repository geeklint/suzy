/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use std::time;

use drying_paint::WatchedValueCore;

use crate::{platform::Platform, pointer::PointerEventData, window::Window};

use super::App;

pub struct AppTesterInterface<'a, P: Platform> {
    pub(super) app: &'a mut App<P>,
}

impl<P: Platform> AppTesterInterface<'_, P> {
    /// Issue an update to ensure all events have fully resolved
    pub fn draw_if_needed(&mut self) {
        if self.app.needs_draw {
            self.app.update_watches();
            self.app.loop_draw();
        }
    }

    /// Start the next frame with a default frame time.
    pub fn next_frame_60fps(&mut self) {
        self.next_frame(time::Duration::from_nanos(16666667));
    }

    /// Update and draw the current frame, then start a new one, acting as
    /// though `frame_time` has passed (e.g. for the purposes of App::time).
    pub fn next_frame(&mut self, frame_time: time::Duration) {
        self.draw_if_needed();
        self.app.finish_draw();
        let frame_time = self.app.state().time().get_unwatched() + frame_time;
        self.app.start_frame(frame_time);
    }

    /// Simulate a pointer event.
    ///
    /// If the passed in event is already in the suzy coordinate system,
    /// remember to set `normalized` to true.
    pub fn pointer(&mut self, pointer: PointerEventData) {
        self.app.pointer_event(pointer);
    }

    /// Take a screenshot.
    ///
    /// Data returned by this function may be dependent on the suzy platform
    /// in use.
    pub fn take_screenshot(&mut self) -> Box<[u8]> {
        self.draw_if_needed();
        self.app.take_screenshot()
    }

    /// Short-hand to simulate a mouse click
    ///
    /// This is equivalent to:
    /// 1) sending a mouse-down pointer event
    /// 2) advancing the frame with the default frame time
    /// 3) sending a mouse-up pointer event
    pub fn mouse_click(&mut self, pos: [f32; 2]) {
        let [px, py] = pos;
        self.pointer(PointerEventData {
            id: crate::pointer::PointerId::Mouse,
            action: crate::pointer::PointerAction::Down,
            x: px,
            y: py,
        });
        self.next_frame_60fps();
        self.pointer(PointerEventData {
            id: crate::pointer::PointerId::Mouse,
            action: crate::pointer::PointerAction::Up,
            x: px,
            y: py,
        });
    }
}
