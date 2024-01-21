/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

#![allow(missing_docs)]

use crate::{
    platforms::opengl::OpenGlRenderPlatform, pointer::AltMouseButton,
};

mod window;

pub struct SdlPlatform {
    sdl: sdl2::Sdl,
}

impl crate::platform::Platform for SdlPlatform {
    type Window = window::Window;
    type Renderer = OpenGlRenderPlatform;

    fn new() -> Self {
        SdlPlatform {
            sdl: sdl2::init().expect("Failed to initialize SDL2"),
        }
    }

    fn create_window(
        &mut self,
        settings: crate::window::WindowBuilder,
    ) -> Result<Self::Window, String> {
        window::Window::new_window(&self.sdl, settings)
    }

    fn run(self, app: &mut crate::app::App<Self>) -> ! {
        let code: i32 = match self.run(app) {
            Ok(()) => 0,
            Err(_) => 1,
        };
        std::process::exit(code)
    }
}

impl SdlPlatform {
    pub fn run(self, app: &mut crate::app::App<Self>) -> Result<(), String> {
        let mut event_pump = self.sdl.event_pump()?;
        loop {
            use sdl2::event::{Event, WindowEvent};
            app.start_frame(std::time::Instant::now());
            while let Some(event) = event_pump.poll_event() {
                match event {
                    Event::Quit { .. }
                    | Event::Window {
                        win_event: WindowEvent::Close,
                        ..
                    } => {
                        return Ok(());
                    }
                    other => window::submit_event(app, other, || {
                        event_pump.mouse_state()
                    }),
                }
            }
            app.update_watches();
            app.draw();
            app.finish_draw();
        }
    }
}

//pub trait AppHandleSdlEvent {
//    fn handle_event(
//        &mut self,
//        event: sdl2::event::Event,
//        mouse_pos: impl FnOnce() -> (f32, f32),
//    );
//}
//
//impl AppHandleSdlEvent for crate::app::App<SdlPlatform> {
//    fn handle_event(
//        &mut self,
//        event: sdl2::event::Event,
//        mouse_pos: impl FnOnce() -> (f32, f32),
//    ) {
//        todo!()
//    }
//}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AltMouseButtonResult {
    Primary,
    Alt(AltMouseButton),
    Unknown,
}

pub trait ToSuzyMouseButton {
    fn to_suzy_mouse_button(self) -> AltMouseButtonResult;
}

impl ToSuzyMouseButton for sdl2::mouse::MouseButton {
    fn to_suzy_mouse_button(self) -> AltMouseButtonResult {
        use AltMouseButtonResult::*;
        match self {
            sdl2::mouse::MouseButton::Unknown => Unknown,
            sdl2::mouse::MouseButton::Left => Primary,
            sdl2::mouse::MouseButton::Middle => Alt(AltMouseButton::Middle),
            sdl2::mouse::MouseButton::Right => Alt(AltMouseButton::Right),
            sdl2::mouse::MouseButton::X1 => Alt(AltMouseButton::X1),
            sdl2::mouse::MouseButton::X2 => Alt(AltMouseButton::X2),
        }
    }
}
