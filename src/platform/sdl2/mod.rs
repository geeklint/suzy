
mod window;
//mod texture_loader;

use crate::platform::opengl::OpenGlRenderPlatform;

pub struct SDLPlatform;

impl crate::platform::Platform for SDLPlatform {
    type Window = window::Window;
    type Renderer = OpenGlRenderPlatform;
}
