use std::ffi::c_void;

use crate::graphics::DrawContext;

use super::OpenGlRenderPlatform as Gl;
use super::bindings::types::*;
use super::bindings::{
    COLOR_BUFFER_BIT,
};
use super::drawparams::DrawParams;
use super::layout::StandardLayout;

/// opengl::Window provides a subset of the methods to implement the Window
/// struct. It can be embedded in another window implementation which
/// provides an opengl context.
pub struct Window {
    layout: StandardLayout,
}

impl Window {
    /// Create an opengl window with a specified function pointer loader
    pub fn new<F>(loader: F) -> Self
        where F: FnMut(&str) -> *const c_void,
    {
        Window {
            layout: StandardLayout::new(),
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
        self.layout.make_current();
        self.layout.set_screen_size(screen_size);
        self.layout.set_tint_color(crate::math::consts::WHITE);
        let params = DrawParams::new(self.layout.clone());
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
