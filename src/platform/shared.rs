use crate::window::{
    Window,
    WindowEvent,
};

pub trait RenderPlatform: 'static {
    type Context: 'static;
    type DrawParams: crate::graphics::DrawParams<Self::Context>;
}

pub enum Event<'a, W> {
    WindowEvent(WindowEvent),
    StartFrame(std::time::Instant),
    Update,
    Draw(&'a mut W),
}

pub trait Platform: 'static {
    type Renderer: RenderPlatform;
    type Window: Window<Self::Renderer>;
}

