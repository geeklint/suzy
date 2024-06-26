/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use std::time;

use drying_paint::WatchedValueCore;

use crate::{platform::RenderPlatform, pointer::PointerEventData};

use super::App;

pub trait TestingExt {
    /// Short-hand to simulate a mouse click
    ///
    /// This is equivalent to:
    /// 1) sending a mouse-down pointer event
    /// 2) advancing the frame with the default frame time
    /// 3) sending a mouse-up pointer event
    fn mouse_click(&mut self, pos: [f32; 2]);

    /// Update and draw the current frame, then start a new one, acting as
    /// though `frame_time` has passed (e.g. for the purposes of [`crate::app::time()`]).
    fn next_frame(&mut self, frame_time: time::Duration);

    /// Start the next frame with a default frame time.
    fn next_frame_60fps(&mut self) {
        let frame_time = time::Duration::from_nanos(16_666_667);
        self.next_frame(frame_time);
    }
}

impl<P: RenderPlatform> TestingExt for App<P> {
    fn next_frame(&mut self, frame_time: time::Duration) {
        self.update_watches();
        let frame_time = self.state().time().get_unwatched() + frame_time;
        self.start_frame(frame_time);
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
