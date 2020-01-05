use std::rc::Rc;
use std::cell::RefCell;

use gl::types::*;

use super::Shader;

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
    vao: GLuint,
    std_shader: Rc<RefCell<Shader>>,
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
        let mut vao = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut vao as *mut _);
            gl::BindVertexArray(vao);
            gl::EnableVertexAttribArray(0);
            gl::EnableVertexAttribArray(1);
        }
        let std_shader = Rc::new(RefCell::new(Shader::standard()));
        Window { vao, std_shader }
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

    pub fn prepare_draw(&mut self) -> super::drawparams::DrawParams;

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
