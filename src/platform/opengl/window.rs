use crate::graphics::DrawContext;

use super::OpenGlRenderPlatform as Gl;
use super::bindings::types::*;
use super::bindings::{
    COLOR_BUFFER_BIT,
    BLEND,
    SRC_ALPHA,
    ONE_MINUS_SRC_ALPHA,
};
use super::drawparams::DrawParams;
use super::stdshaders::Shaders;
use super::Mat4;

/// opengl::Window provides a subset of the methods to implement the Window
/// struct. It can be embedded in another window implementation which
/// provides an opengl context.
pub struct Window {
    shaders: Shaders,
}

impl Window {
    /// Create an opengl window with a specified function pointer loader
    pub fn new() -> Self {
        Gl::global(|gl| unsafe {
            gl.ClearColor(0.176, 0.176, 0.176, 1.0);
            gl.Enable(BLEND);
            gl.BlendFunc(SRC_ALPHA, ONE_MINUS_SRC_ALPHA);
        });
        Window {
            shaders: Shaders::new().expect("Shaders failed to compile"),
        }
    }

    /// Set the viewport. Wrapping windows will probably want to do this
    /// when they detect a resize.
    pub fn viewport(&mut self, x: i32, y: i32, width: u32, height: u32) {
        Gl::global(|gl| unsafe {
            gl.Viewport(
                x as GLint,
                y as GLint,
                width as GLsizei,
                height as GLsizei,
            );
        });
    }

    pub fn prepare_draw(&mut self, screen_size: (f32, f32))
        -> DrawContext<Gl>
    {
        let mut params = DrawParams::new(self.shaders.clone());
        params.transform(
            Mat4::translate(-1.0, -1.0)
            * Mat4::scale(2.0 / screen_size.0, 2.0 / screen_size.1)
        );
        DrawContext::new(params)
    }

    /// Issue opengl call to clear the screen.
    pub fn clear(&mut self) {
        Gl::global(|gl| unsafe {
            gl.Clear(COLOR_BUFFER_BIT);
        });
    }

    /// This function does not block like window::Window requires.
    pub fn flip(&mut self) {
        Gl::global(|gl| unsafe {
            gl.Flush();  // needed?
        });
    }
}
