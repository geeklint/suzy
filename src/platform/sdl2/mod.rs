
mod window;
//mod texture_loader;

use crate::platform::opengl::OpenGlRenderPlatform;

pub struct SDLPlatform;

impl super::Platform for SDLPlatform {
    type Window = window::Window;
    type Renderer = OpenGlRenderPlatform;
}
