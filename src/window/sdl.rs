
use std::rc::Rc;
use std::cell::RefCell;
use std::io;

use gl::types::*;

use sdl2::event::{Event, WindowEvent};
use sdl2::video::WindowBuildError;

use crate::platform::opengl::{Shader, ProgramCompileError};
use crate::graphics::{DrawContext, DrawParams};

fn s2io(msg: String) -> io::Error {
    io::Error::new(io::ErrorKind::Other, msg)
}

pub struct Window {
    sdl: sdl2::Sdl,
    video: sdl2::VideoSubsystem,
    window: sdl2::video::Window,
    context: sdl2::video::GLContext,
    vao: GLuint,
    std_shader: Rc<RefCell<Shader>>,
}

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
        println!("OpenGL message not falid utf8");
    }
}

impl Window {
    pub fn new() -> Result<Self, io::Error> {
        let sdl = sdl2::init().map_err(s2io)?;
        let video = sdl.video().map_err(s2io)?;
        let video2 = video.clone();
        let gl_attr = video.gl_attr();
        gl_attr.set_red_size(5);
        gl_attr.set_green_size(5);
        gl_attr.set_blue_size(5);
        gl_attr.set_double_buffer(true);
        gl_attr.set_multisample_buffers(1);
        gl_attr.set_multisample_samples(4);
        gl_attr.set_context_version(3, 3);
        let mut win_builder = video.window("Suzy Window", 480, 480);
        win_builder.opengl().allow_highdpi().resizable();
        let window = match win_builder.build() {
            Ok(window) => Ok(window),
            Err(WindowBuildError::SdlError(msg)) => Err(s2io(msg)),
            _ => panic!("Unexpected window builder error!"),
        }?;
        video.gl_set_swap_interval(sdl2::video::SwapInterval::VSync);
        let context = window.gl_create_context().map_err(s2io)?;
        gl::load_with(move |s| {
            video2.gl_get_proc_address(s) as *const std::ffi::c_void
        });
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
        let std_shader = Shader::standard().map_err(|err| {
            let msg = match err {
                ProgramCompileError::Vertex(msg) => msg,
                ProgramCompileError::Fragment(msg) => msg,
                ProgramCompileError::Link(msg) => msg,
            };
            s2io(msg.to_string_lossy().into_owned())
        })?;
        let std_shader = Rc::new(RefCell::new(std_shader));
        Ok(Window { sdl, video, window, context, vao, std_shader })
    }

    pub fn before_draw(&mut self) -> DrawContext {
        unsafe {
            gl::BindVertexArray(self.vao);
            let mut std_shader = self.std_shader.borrow_mut();
            std_shader.set_current();
            let (width, height) = self.get_size();
            std_shader.set_uniform_vec2("SCREEN_SIZE", (width, height));
        }
        let starting = DrawParams::new(self.std_shader.clone());
        DrawContext::new(starting)
    }

    pub fn get_size(&self) -> (f32, f32) {
        let ppd = self.get_pixels_per_dp();
        let (width, height) = self.window.drawable_size();
        ((width as f32) / ppd, (height as f32) / ppd)
    }

    pub fn get_pixels_per_dp(&self) -> f32 {
        let display = self.window.display_index()
            .expect("Failed to get window display index");
        match self.video.display_dpi(display) {
            Ok((ddpi, hdpi, vdpi)) => {
                let dpi = ((hdpi + vdpi) / 2.0) as f32;
                dpi / 160.0
            }
            Err(msg) => {
                panic!("Failed to get dpi from display {}: {}",
                    display, msg
                )
            }
        }
    }

    pub fn events(&self) -> Events {
        Events {
            window: self,
            events: self.sdl.event_pump().unwrap(),
        }
    }

    pub fn clear(&self) {
		unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }

    pub fn flip(&mut self) {
        unsafe { gl::Flush() };
        self.window.gl_swap_window();
    }
}

pub struct Events<'a> {
    window: &'a Window,
    events: sdl2::EventPump,
}

impl Events<'_> {
    fn win_event(&self, win_event: WindowEvent) -> Option<super::WindowEvent> {
        Some(match win_event {
            WindowEvent::SizeChanged(width, height) => {
                unsafe {
                    gl::Viewport(0, 0, width, height);
                }
                let (width, height) = self.window.get_size();
                super::WindowEvent::Resize(width, height)
            }
            WindowEvent::Close => {
                super::WindowEvent::Quit
            }
            _ => return None,
        })
    }
}

impl Iterator for Events<'_> {
    type Item = super::WindowEvent;
    fn next(&mut self) -> Option<super::WindowEvent> {
        while let Some(event) = self.events.poll_event() {
            return Some(match event {
                Event::Window { win_event, .. } => {
                    match self.win_event(win_event) {
                        Some(ev) => ev,
                        None => continue,
                    }
                }
                _ => continue,
            })
        }
        None
    }
}
