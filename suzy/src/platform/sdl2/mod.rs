/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(missing_docs)]

use super::{Event, SimpleEventLoopState};

use crate::platform::opengl::OpenGlRenderPlatform;

mod window;
//mod texture_loader;

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

    fn run<F>(self, mut event_handler: F) -> !
    where
        F: 'static + FnMut(&mut Self::State, Event),
    {
        let mut state = SimpleEventLoopState::default();
        let mut events = window::Events {
            events: self
                .sdl
                .event_pump()
                .expect("Failed to create SDL2 event pump"),
            send_dp: false,
        };
        while state.running {
            event_handler(
                &mut state,
                Event::StartFrame(std::time::Instant::now()),
            );
            while let Some(event) = events.next() {
                event_handler(&mut state, Event::WindowEvent(event));
            }
            event_handler(&mut state, Event::Update);
            event_handler(&mut state, Event::Draw);
            event_handler(&mut state, Event::FinishDraw);
        }
        std::process::exit(0)
    }
}
