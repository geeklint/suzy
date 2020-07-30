use std::convert::TryInto;

use super::{
    Event,
    SimpleEventLoopState,
};

use crate::platform::opengl::OpenGlRenderPlatform;

mod window;
//mod texture_loader;

pub struct SDLPlatform {
    sdl: sdl2::Sdl,
}

impl crate::platform::Platform for SDLPlatform {
    type State = SimpleEventLoopState;
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
        let mut state = SimpleEventLoopState::default();
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
