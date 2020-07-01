#[macro_use] mod primitive;
mod buffer;
mod texture;

mod context;
mod drawparams;
pub mod graphics;
mod matrix;
mod shader;
mod stdshaders;
pub mod text;
mod window;

pub use context::{OpenGlContext, OpenGlBindings};

pub use matrix::Mat4;

pub(crate) use window::{
    Window,
};

pub struct OpenGlRenderPlatform;

impl super::RenderPlatform for OpenGlRenderPlatform {
    type Context = OpenGlContext;
    type DrawParams = drawparams::DrawParams;
}
