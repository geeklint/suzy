
mod window;
mod texture_loader;

use crate::platform::opengl::OpenGlRenderPlatform as Renderer;

pub struct SDLPlatform;

type Global = <Renderer as super::RenderPlatform>::Global;

impl super::Platform for SDLPlatform {
    type Window = window::Window;
    type Renderer = Renderer;

    fn get_renderer_data(window: &mut Self::Window) -> Global {
        Renderer::load(|s| window.load_glfn(s) as *const _)
    }
}
