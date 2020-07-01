use std::rc::Rc;
use std::borrow::Cow;
use std::collections::HashMap;

use super::stdshaders::Shaders;
use super::texture::TextureCache;

pub mod bindings {
    include!(concat!(env!("OUT_DIR"), "/opengl_bindings.rs"));
}

pub type OpenGlBindings = bindings::Gles2;

pub struct OpenGlContext {
    pub(super) bindings: Rc<OpenGlBindings>,
    pub(super) shaders: Shaders,
    pub(super) texture_cache: TextureCache,
}

impl OpenGlContext {
    pub fn new<F>(loader: F) -> Self
    where
        F: FnMut(&str) -> *const std::ffi::c_void,
    {
        let ptr = Rc::new(OpenGlBindings::load_with(loader));
        if cfg!(debug_assertions) && ptr.DebugMessageCallback.is_loaded() {
            unsafe {
                ptr.Enable(bindings::DEBUG_OUTPUT);
                ptr.DebugMessageCallback(
                    Self::message_callback,
                    std::ptr::null(),
                );
            }
        }
        let shaders = Shaders::new(&ptr).expect("Failed to compile shaders");
        Self {
            bindings: ptr,
            shaders,
            texture_cache: HashMap::new(),
        }
    }

    extern "system" fn message_callback(
        _source: bindings::types::GLenum,
        _gltype: bindings::types::GLenum,
        _id: bindings::types::GLuint,
        _severity: bindings::types::GLenum,
        length: bindings::types::GLsizei,
        message: *const bindings::types::GLchar,
        _user_param: *mut std::ffi::c_void,
    ) {
        let data = unsafe {
            std::slice::from_raw_parts(message as *const u8, length as usize)
        };
        println!("{}", String::from_utf8_lossy(data));
    }
}
