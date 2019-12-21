
use std::io;
use std::mem::MaybeUninit;
use std::os::raw::{c_int, c_float};

use crate::platform::sdl as ffi;

pub struct Window {
    window: *mut ffi::SDL_Window,
    context: ffi::SDL_GLContext,
}

impl Window {
    pub fn new() -> Result<Self, io::Error> {
        if unsafe { ffi::SDL_Init(ffi::SDL_INIT_EVERYTHING) } < 0 {
            return Err(io::Error::new(io::ErrorKind::Other, "SDL_Init failed"));
        }
        unsafe {
            ffi::SDL_GL_SetAttribute(ffi::SDL_GLattr_SDL_GL_RED_SIZE, 5);
            ffi::SDL_GL_SetAttribute(ffi::SDL_GLattr_SDL_GL_GREEN_SIZE, 5);
            ffi::SDL_GL_SetAttribute(ffi::SDL_GLattr_SDL_GL_BLUE_SIZE, 5);
            ffi::SDL_GL_SetAttribute(ffi::SDL_GLattr_SDL_GL_DOUBLEBUFFER, 1);
            ffi::SDL_GL_SetAttribute(ffi::SDL_GLattr_SDL_GL_MULTISAMPLEBUFFERS, 1);
            ffi::SDL_GL_SetAttribute(ffi::SDL_GLattr_SDL_GL_MULTISAMPLESAMPLES, 4);
            ffi::SDL_GL_SetAttribute(ffi::SDL_GLattr_SDL_GL_CONTEXT_MAJOR_VERSION, 3);
            ffi::SDL_GL_SetAttribute(ffi::SDL_GLattr_SDL_GL_CONTEXT_MINOR_VERSION, 3);
        }

        let title = std::ffi::CString::new("Suzy Window").unwrap();

        let window = unsafe {
            ffi::SDL_CreateWindow(
                title.as_ptr(),
                100,
                100,
                1024, 480,
                ffi::SDL_WindowFlags_SDL_WINDOW_OPENGL | ffi::SDL_WindowFlags_SDL_WINDOW_ALLOW_HIGHDPI,
            )
        };
        assert!(!window.is_null(), "Failed to create SDL Window");
        unsafe {
            ffi::SDL_GL_SetSwapInterval(1);
        }
        let context = unsafe { ffi::SDL_GL_CreateContext(window) };
        assert!(!context.is_null(), "Failed to create OpenGL Context");
        Ok(Window { window, context })
    }

    pub fn get_size(&self) -> (f32, f32) {
        let ppd = self.get_pixels_per_dp();
        let width = 0;
        let height = 0;
        unsafe {
            ffi::SDL_GL_GetDrawableSize(
                self.window,
                &mut width as *mut c_int,
                &mut height as *mut c_int,
            );
        }
        ((width as f32) / ppd, (height as f32) / ppd)
    }

    pub fn get_pixels_per_dp(&self) -> f32 {
        let display = unsafe { ffi::SDL_GetWindowDisplayIndex(self.window) };
        if display < 0 {
            panic!("failed to get display index");
        }
        let nan = std::f32::NAN as c_float;
        let ddpi = nan;
        let hdpi = nan;
        let vdpi = nan;
        let error = unsafe {
            ffi::SDL_GetDisplayDPI(
                display,
                &mut ddpi as *mut c_float,
                &mut hdpi as *mut c_float,
                &mut vdpi as *mut c_float,
            )
        };
        if error != 0 {
            panic!("failed to get dpi information from display {}", display);
        }
        let dpi = ((hdpi + vdpi) / 2.0) as f32;
        dpi / 160.0
    }

    pub fn events(&self) -> Events {
        Events { window: self }
    }

    pub fn clear(&self) {
		unsafe {
            ffi::glClear(ffi::GL_COLOR_BUFFER_BIT | ffi::GL_DEPTH_BUFFER_BIT);
        }
    }

    pub fn flip(&mut self) {
		unsafe { ffi::SDL_GL_SwapWindow(self.window); }
    }
}

pub struct Events<'a> {
    window: &'a Window,
}

impl Iterator for Events<'_> {
    type Item = super::WindowEvent;
    fn next(&mut self) -> Option<super::WindowEvent> {
        let event = MaybeUninit::uninit();
        while unsafe { ffi::SDL_PollEvent(event.as_mut_ptr()) } != 0 {
            let event = &*event.as_ptr();
            match event.type_ {
                ffi::SDL_EventType_SDL_WINDOWEVENT => {
                    match event.window.event as u32 {
                        ffi::SDL_WindowEventID_SDL_WINDOWEVENT_SIZE_CHANGED => {
                            let (width, height) = self.window.get_size();
                            return Some(super::WindowEvent::Resize(width, height));
                        }
                        _ => continue,
                    }
                }
                _ => continue,
            }
        }
        None
    }
}
