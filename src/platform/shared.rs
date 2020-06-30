
pub trait RenderPlatform: 'static {
    type Context: 'static;
    type DrawParams: crate::graphics::DrawParams<Self::Context>;
}

pub trait Platform: 'static {
    type Window: crate::window::Window<Self::Renderer>;
    type Renderer: RenderPlatform;

    fn get_renderer_data(window: &mut Self::Window)
        -> <Self::Renderer as RenderPlatform>::Context;
}

