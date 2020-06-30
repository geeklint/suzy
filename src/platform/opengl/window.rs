use crate::math::Color;
use crate::graphics::DrawContext;

use super::{
    OpenGlRenderPlatform,
    OpenGlContext,
    Mat4,
};
use super::bindings::types::*;
use super::bindings::{
    COLOR_BUFFER_BIT,
    BLEND,
    SRC_ALPHA,
    ONE_MINUS_SRC_ALPHA,
};
use super::drawparams::DrawParams;
use super::stdshaders::Shaders;

/// opengl::Window provides a subset of the methods to implement the Window
/// trait. It can be embedded in another window implementation which
/// provides an opengl context.
pub struct Window {
    ctx: OpenGlContext,
}

impl Window {
    /// Create an opengl window
    pub fn new(ctx: OpenGlContext) -> Self {
        Window { ctx }
    }

    pub fn clear_color(&mut self, color: Color) {
        let (r, g, b, a) = color.rgba();
        unsafe {
            self.ctx.bindings.ClearColor(r, g, b, a);
        }
    }

    /// Set the viewport. Wrapping windows will probably want to do this
    /// when they detect a resize.
    pub fn viewport(&mut self, x: i32, y: i32, width: u32, height: u32) {
        unsafe {
            self.ctx.bindings.Viewport(
                x as GLint,
                y as GLint,
                width as GLsizei,
                height as GLsizei,
            );
        }
    }

    pub fn prepare_draw<'a>(&'a mut self, screen_size: (f32, f32))
        -> DrawContext<'a, OpenGlRenderPlatform>
    {
        unsafe {
            self.ctx.bindings.Enable(BLEND);
            self.ctx.bindings.BlendFunc(SRC_ALPHA, ONE_MINUS_SRC_ALPHA);
        }
        let mut params = DrawParams::new();
        params.transform(
            Mat4::translate(-1.0, -1.0)
            * Mat4::scale(2.0 / screen_size.0, 2.0 / screen_size.1)
        );
        DrawContext::new(&mut self.ctx, params)
    }

    /// Issue opengl call to clear the screen.
    pub fn clear(&mut self) {
        unsafe {
            self.ctx.bindings.Clear(COLOR_BUFFER_BIT);
        }
    }

    /// This function does not block like window::Window requires.
    pub fn flip(&mut self) {
    }
}
