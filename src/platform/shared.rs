use crate::window::{
    WindowEvent,
};

pub trait RenderPlatform: 'static {
    type Context: 'static;
    type DrawParams: crate::graphics::DrawParams<Self::Context>;
}

pub enum Event {
    WindowEvent(WindowEvent),
    StartFrame(std::time::Instant),
    Update,
    Draw,
}

pub trait Platform: 'static {
    type Window: crate::window::Window<Self::Renderer>;
    type Renderer: RenderPlatform;
}

