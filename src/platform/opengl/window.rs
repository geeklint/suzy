use std::rc::Rc;
use std::cell::RefCell;

use gl::types::*;

use crate::graphics::DrawContext;

use super::Shader;
use super::DrawParams;
use super::graphics::layout::StandardLayout;

extern "system" fn message_callback(
    source: GLenum,
    gltype: GLenum,
    id: GLuint,
    severity: GLenum,
    length: GLsizei,
    message: *const GLchar,
    user_param: *mut std::ffi::c_void,
) {
    let data = unsafe {
        std::slice::from_raw_parts(message as *const u8, length as usize)
    };
    if let Ok(string) = std::str::from_utf8(data) {
        println!("{}", string);
    } else {
        println!("OpenGL message not valid utf8");
    }
}

/// opengl::Window provides a subset of the methods to implement the Window
/// struct. It can be embedded in another window implementation which
/// provides an opengl context.
pub struct Window {
    layout: StandardLayout,
}

impl Window {
    /// Create an opengl window. This assumes there is an active opengl
    /// context.
    pub fn new() -> Self {
        unsafe {
            gl::Enable(gl::DEBUG_OUTPUT);
            gl::DebugMessageCallback(
                Some(message_callback),
                std::ptr::null(),
            );
        }
        Window {
            layout: StandardLayout::new(),
        }
    }

    /// Set the viewport. Wrapping windows will probably want to do this
    /// when they detect a resize.
    pub fn viewport(&mut self, x: i32, y: i32, width: u32, height: u32) {
        unsafe {
            gl::Viewport(
                x as GLint,
                y as GLint,
                width as GLsizei,
                height as GLsizei,
            );
        }
    }

    pub fn prepare_draw(&mut self, screen_size: (f32, f32)) -> DrawContext {
        self.layout.make_current();
        self.layout.set_screen_size(screen_size);
        self.layout.set_tint_color(crate::math::consts::WHITE);
        let params = DrawParams::new(self.layout.clone());
        DrawContext::new(params)
    }

    /// Issue opengl call to clear the screen.
    pub fn clear(&mut self) {
		unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
    }

    /// This function does not block like window::Window requires.
    pub fn flip(&mut self) {
        unsafe {
            gl::Flush();  // needed?
        }
    }
}
