/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

#![allow(missing_docs)]

use crate::{
    platform::{Event, SimpleEventLoopState},
    platforms::opengl::OpenGlRenderPlatform,
};

mod window;

pub struct SdlPlatform {
    sdl: sdl2::Sdl,
}

impl crate::platform::Platform for SdlPlatform {
    type State = SimpleEventLoopState;
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
        let mut state = SimpleEventLoopState::default();
        let mut events = window::Events {
            events: self
                .sdl
                .event_pump()
                .expect("Failed to create SDL2 event pump"),
            send_dp: false,
        };
        while state.running {
            app.handle_event(
                &mut state,
                Event::StartFrame(std::time::Instant::now()),
            );
            while let Some(event) = events.next() {
                app.handle_event(&mut state, Event::WindowEvent(event));
            }
            app.handle_event(&mut state, Event::Update);
            app.handle_event(&mut state, Event::Draw);
            app.handle_event(&mut state, Event::FinishDraw);
        }
        std::process::exit(0)
    }
}
