use std::convert::TryInto;

mod window;
//mod texture_loader;

use crate::platform::opengl::OpenGlRenderPlatform;

pub struct SDLPlatform;

impl crate::platform::Platform for SDLPlatform {
    type Window = window::Window;
    type Renderer = OpenGlRenderPlatform;

    fn new() -> Self {
        SDLPlatform
    }

    fn create_window(&mut self, settings: crate::window::WindowBuilder)
        -> Result<Self::Window, String>
    {
        settings.try_into()
    }
}
