use crate::window::{
    Window,
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

pub trait EventLoopState {
    fn request_shutdown(&mut self);
}

pub trait Platform: 'static {
    type Renderer: RenderPlatform;
    type Window: Window<Self::Renderer>;
}

