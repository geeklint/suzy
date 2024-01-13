/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

#![allow(missing_docs)]

use crate::{
    platform::SimpleEventLoopState, platforms::opengl::OpenGlRenderPlatform,
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
        let mut event_pump = self
            .sdl
            .event_pump()
            .expect("Failed to create SDL2 event pump");
        while state.running {
            app.start_frame(std::time::Instant::now());
            window::pump(&mut event_pump, &mut state, app);
            app.update_watches();
            app.draw();
            app.finish_draw();
        }
        std::process::exit(0)
    }
}
