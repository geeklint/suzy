
use std::io;

use sdl2::event::{Event, WindowEvent};
use sdl2::video::WindowBuildError;

fn s2io(msg: String) -> io::Error {
    io::Error::new(io::ErrorKind::Other, msg)
}

pub struct Window {
    sdl: sdl2::Sdl,
    video: sdl2::VideoSubsystem,
    window: sdl2::video::Window,
    context: sdl2::video::GLContext,
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
        let mut win_builder = video.window("Suzy Window", 1024, 480);
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
        Ok(Window { sdl, video, window, context })
    }

    pub fn before_draw(&mut self) {
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
            WindowEvent::SizeChanged(_, _) => {
                let (width, height) = self.window.get_size();
                super::WindowEvent::Resize(width, height)
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
