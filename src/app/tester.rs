use std::time;

use crate::platform::{
    Event,
    Platform,
    SimpleEventLoopState,
};
use crate::pointer::PointerEventData;
use crate::window::WindowEvent;

use super::{
    App,
    CurrentApp,
};

pub struct AppTesterInterface<'a, P: Platform> {
    start_time: time::Instant,
    state: SimpleEventLoopState,
    app: &'a mut CurrentApp<P>,
    needs_draw: bool,
}

impl<'a, P: Platform> AppTesterInterface<'a, P> {
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
    pub fn assert_exited(self) {
        if self.state.running {
            panic!("assert_exited called but app has not exited");
        }
    }

    fn assert_running(&self) {
        if !self.state.running {
            panic!("app exited unexpectedly during test");
        }
    }

    fn draw_if_needed(&mut self) {
        if self.needs_draw {
            self.app.handle_event(
                &mut self.state,
                Event::Update,
            );
            self.app.handle_event(
                &mut self.state,
                Event::Draw,
            );
            self.needs_draw = false;
        }
    }

    pub fn next_frame_60fps(&mut self) {
        self.next_frame(time::Duration::from_nanos(16666667));
    }

    pub fn next_frame(&mut self, frame_time: time::Duration) {
        self.assert_running();
        self.draw_if_needed();
        self.app.handle_event(
            &mut self.state,
            Event::FinishDraw,
        );
        self.start_time += frame_time;
        self.app.handle_event(
            &mut self.state,
            Event::StartFrame(self.start_time),
        );
        self.needs_draw = true;
    }

    pub fn pointer(&mut self, pointer: PointerEventData) {
        self.assert_running();
        self.app.handle_event(
            &mut self.state,
            Event::WindowEvent(WindowEvent::Pointer(pointer)),
        );
        self.needs_draw = true;
    }

    pub fn take_screenshot(&mut self) -> Box<[u8]> {
        self.assert_running();
        self.draw_if_needed();
        let mut data: Box<[u8]> = Box::new([0u8; 0]);
        self.app.handle_event(
            &mut self.state,
            Event::TakeScreenshot(&mut data),
        );
        data
    }

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
