/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::time;

use crate::platform::{Event, Platform, SimpleEventLoopState};
use crate::pointer::PointerEventData;
use crate::window::WindowEvent;

use super::{App, CurrentApp};

/// An interface to enable some automated testing of an app.
///
/// Retrieve with [`App::test`](struct.App.html#method.test)
pub struct AppTesterInterface<'a, P: Platform> {
    start_time: time::Instant,
    state: SimpleEventLoopState,
    app: &'a mut CurrentApp<P>,
    needs_draw: bool,
}

impl<'a, P: Platform> AppTesterInterface<'a, P> {
    /// Create a tester interface from a CurrentApp.
    pub fn new(app: &'a mut CurrentApp<P>) -> Self {
        let start_time = App::<P>::time();
        let mut state = SimpleEventLoopState::default();
        app.handle_event(&mut state, Event::StartFrame(start_time));
        Self {
            app,
            state,
            start_time,
            needs_draw: true,
        }
    }
}

impl<P: Platform> AppTesterInterface<'_, P> {
    /// Panic if the tested app is still running.
    pub fn assert_exited(self) {
        if self.state.running {
            panic!("assert_exited called but app has not exited");
        }
    }

    /// Panic if the tested app has quit.
    fn assert_running(&self) {
        if !self.state.running {
            panic!("app exited unexpectedly during test");
        }
    }

    /// Issue an update to ensure all events have fully resolved
    pub fn draw_if_needed(&mut self) {
        if self.needs_draw {
            self.app.handle_event(&mut self.state, Event::Update);
            self.app.handle_event(&mut self.state, Event::Draw);
            self.needs_draw = false;
        }
    }

    /// Start the next frame with a default frame time.
    pub fn next_frame_60fps(&mut self) {
        self.next_frame(time::Duration::from_nanos(16666667));
    }

    /// Update and draw the current frame, then start a new one, acting as
    /// though `frame_time` has passed (e.g. for the purposes of App::time).
    pub fn next_frame(&mut self, frame_time: time::Duration) {
        self.assert_running();
        self.draw_if_needed();
        self.app.handle_event(&mut self.state, Event::FinishDraw);
        self.start_time += frame_time;
        self.app
            .handle_event(&mut self.state, Event::StartFrame(self.start_time));
        self.needs_draw = true;
    }

    /// Simulate a pointer event.
    ///
    /// If the passed in event is already in the suzy coordinate system,
    /// remember to set `normalized` to true.
    pub fn pointer(&mut self, pointer: PointerEventData) {
        self.assert_running();
        self.app.handle_event(
            &mut self.state,
            Event::WindowEvent(WindowEvent::Pointer(pointer)),
        );
        self.needs_draw = true;
    }

    /// Take a screenshot.
    ///
    /// Data returned by this function may be dependent on the suzy platform
    /// in use.
    pub fn take_screenshot(&mut self) -> Box<[u8]> {
        self.assert_running();
        self.draw_if_needed();
        let mut data: Box<[u8]> = Box::new([0u8; 0]);
        self.app
            .handle_event(&mut self.state, Event::TakeScreenshot(&mut data));
        data
    }

    /// Short-hand to simulate a mouse click
    ///
    /// This is equivalent to:
    /// 1) sending a mouse-down pointer event
    /// 2) advancing the frame with the default frame time
    /// 3) sending a mouse-up pointer event
    pub fn mouse_click(&mut self, pos: (f32, f32)) {
        self.pointer(PointerEventData {
            id: crate::pointer::PointerId::Mouse,
            action: crate::pointer::PointerAction::Down,
            x: pos.0,
            y: pos.1,
            normalized: true,
        });
        self.next_frame_60fps();
        self.pointer(PointerEventData {
            id: crate::pointer::PointerId::Mouse,
            action: crate::pointer::PointerAction::Up,
            x: pos.0,
            y: pos.1,
            normalized: true,
        });
    }
}
