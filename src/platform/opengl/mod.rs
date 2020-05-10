
mod drawparams;
pub mod graphics;
mod matrix;
mod primitive;
mod sdf;
mod shader;
mod stdshaders;
pub mod text;
mod window;

pub use matrix::Mat4;

pub(crate) use shader::{
    Shader,
    ProgramCompileError,
};

pub(crate) use window::{
    Window,
};

pub(crate) use graphics::{
    image,
};

pub(crate) use primitive::{
    Texture,
};

//pub use text::{Text, Font};

pub mod bindings {
    include!(concat!(env!("OUT_DIR"), "/opengl_bindings.rs"));
}

type Gl = bindings::Gles2;

pub struct OpenGlRenderPlatform;

impl OpenGlRenderPlatform {
    pub fn load<F>(loader: F) -> Gl
        where F: FnMut(&str) -> *const std::ffi::c_void,
    {
        let gl = Gl::load_with(loader);
        if cfg!(debug_assertions) && gl.DebugMessageCallback.is_loaded() {
            unsafe {
                gl.Enable(bindings::DEBUG_OUTPUT);
                gl.DebugMessageCallback(
                    Self::message_callback,
                    std::ptr::null(),
                );
            }
        }
        gl
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

impl super::RenderPlatform for OpenGlRenderPlatform {
    type Global = Gl;
    type DrawParams = drawparams::DrawParams;
}

impl OpenGlRenderPlatform {
    pub fn global<F, R>(func: F) -> R
        where F: FnOnce(&Gl) -> R,
    {
        <Self as super::RenderPlatform>::global(func)
    }
}
