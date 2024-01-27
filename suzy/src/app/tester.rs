/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use std::time;

use drying_paint::WatchedValueCore;

use crate::{platform::Platform, pointer::PointerEventData, window::Window};

use super::App;

pub trait AppTestingExt {
    /// Issue an update to ensure all events have fully resolved
    fn draw_if_needed(&mut self);

    /// Short-hand to simulate a mouse click
    ///
    /// This is equivalent to:
    /// 1) sending a mouse-down pointer event
    /// 2) advancing the frame with the default frame time
    /// 3) sending a mouse-up pointer event
    fn mouse_click(&mut self, pos: [f32; 2]);

    /// Take a screenshot.
    ///
    /// Data returned by this function may be dependent on the suzy platform
    /// in use.
    fn draw_and_take_screenshot(&mut self) -> Box<[u8]>;

    /// Update and draw the current frame, then start a new one, acting as
    /// though `frame_time` has passed (e.g. for the purposes of App::time).
    fn next_frame(&mut self, frame_time: time::Duration);

    /// Start the next frame with a default frame time.
    fn next_frame_60fps(&mut self) {
        let frame_time = time::Duration::from_nanos(16666667);
        self.next_frame(frame_time);
    }
}

impl<P: Platform> AppTestingExt for App<P> {
    fn draw_if_needed(&mut self) {
        if self.needs_draw {
            self.update_watches();
            self.loop_draw();
        }
    }

    fn next_frame(&mut self, frame_time: time::Duration) {
        self.update_watches();
        let frame_time = self.state().time().get_unwatched() + frame_time;
        self.start_frame(frame_time);
    }

    fn draw_and_take_screenshot(&mut self) -> Box<[u8]> {
        self.draw_if_needed();
        self.window.take_screenshot()
    }

    fn mouse_click(&mut self, pos: [f32; 2]) {
        let [px, py] = pos;
        self.pointer_event(PointerEventData {
            id: crate::pointer::PointerId::Mouse,
            action: crate::pointer::PointerAction::Down,
            x: px,
            y: py,
        });
        self.next_frame_60fps();
        self.pointer_event(PointerEventData {
            id: crate::pointer::PointerId::Mouse,
            action: crate::pointer::PointerAction::Up,
            x: px,
            y: py,
        });
    }
}
