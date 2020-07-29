use std::convert::TryInto;

use super::{
    Event,
};

use crate::platform::opengl::OpenGlRenderPlatform;

mod window;
//mod texture_loader;

pub struct SDLPlatform {
    sdl: sdl2::Sdl,
}

pub struct EventLoopState {
    running: bool,
}

impl super::EventLoopState for EventLoopState {
    fn request_shutdown(&mut self) {
        self.running = false;
    }
}

impl crate::platform::Platform for SDLPlatform {
    type State = EventLoopState;
    type Window = window::Window;
    type Renderer = OpenGlRenderPlatform;

    fn new() -> Self {
        SDLPlatform {
            sdl: sdl2::init().unwrap(),
        }
    }

    fn create_window(&mut self, settings: crate::window::WindowBuilder)
        -> Result<Self::Window, String>
    {
        window::Window::new_window(&self.sdl, settings)
    }

    fn run<F>(self, mut event_handler: F) -> !
    where
        F: 'static + FnMut(&mut Self::State, Event)
    {
        let mut state = EventLoopState { running: true };
        let mut events = window::Events {
            events: self.sdl.event_pump().unwrap(),
            send_dp: false,
        };
        while state.running {
            event_handler(
                &mut state,
                Event::StartFrame(std::time::Instant::now()),
            );
            while let Some(event) = events.next() {
                event_handler(
                    &mut state,
                    Event::WindowEvent(event),
                );
            }
            event_handler(
                &mut state,
                Event::Update,
            );
            event_handler(
                &mut state,
                Event::Draw,
            );
        }
        std::process::exit(0)
    }
}
