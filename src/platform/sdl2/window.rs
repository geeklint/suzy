use std::convert::{TryFrom, TryInto};

use sdl2::event::Event;
use sdl2::event::WindowEvent as sdl_WindowEvent;
use sdl2::video::WindowBuildError;

use crate::graphics::DrawContext;
use crate::window;
use crate::window::{WindowEvent, WindowSettings, WindowBuilder};
use crate::platform::opengl;

use super::texture_loader::load_texture;

#[derive(Copy, Clone, PartialEq)]
struct PixelInfo {
    display_index: i32,
    pixels_per_dp: f32,
    dp_per_screen_unit: f32,
    pixel_size: (u32, u32),
    screen_size: (u32, u32),
    size: (f32, f32),
}

impl PixelInfo {
    fn diff_to_event(&mut self, other: &Self) -> Option<WindowEvent> {
        if self.size != other.size {
            self.size = other.size;
            Some(WindowEvent::Resize(self.size.0, self.size.1))
        } else if self.pixels_per_dp != other.pixels_per_dp {
            self.pixels_per_dp = other.pixels_per_dp;
            Some(WindowEvent::DpScaleChange(self.pixels_per_dp))
        } else {
            None
        }
    }
}

impl TryFrom<&sdl2::video::Window> for PixelInfo {
    type Error = String;

    fn try_from(window: &sdl2::video::Window) -> Result<Self, Self::Error> {
        let display_index = window.display_index()?;
        let (ddpi, hdpi, vdpi) = {
            window.subsystem().display_dpi(display_index)?
        };
        let dpi = ((hdpi + vdpi) / 2.0) as f32;
        let pixels_per_dp = dpi / 160.0;
        let screen_size = window.size();
        let pixel_size = window.drawable_size();
        let x_px_per_su = (pixel_size.0 as f32) / (screen_size.0 as f32);
        let y_px_per_su = (pixel_size.1 as f32) / (screen_size.1 as f32);
        let px_per_su = (x_px_per_su + y_px_per_su) / 2.0;
        let width = (pixel_size.0 as f32) / pixels_per_dp;
        let height = (pixel_size.1 as f32) / pixels_per_dp;
        Ok(Self {
            display_index,
            pixels_per_dp,
            dp_per_screen_unit: 1.0 / (pixels_per_dp * px_per_su),
            pixel_size,
            screen_size,
            size: (width, height),
        })
    }
}

pub struct Window {
    title: String,
    pixel_info: PixelInfo,
    gl_win: opengl::Window,
    context: sdl2::video::GLContext,
    window: sdl2::video::Window,
    video: sdl2::VideoSubsystem,
    image: sdl2::image::Sdl2ImageContext,
    sdl: sdl2::Sdl,
}

impl TryFrom<WindowBuilder> for Window {
    type Error = String;

    fn try_from(builder: WindowBuilder) -> Result<Self, Self::Error> {
        // initialize systems
        let sdl = sdl2::init()?;
        let video = sdl.video()?;
        let image = sdl2::image::init(sdl2::image::InitFlag::all())?;
        // setup window parameters
        let gl_attr = video.gl_attr();
        gl_attr.set_red_size(5);
        gl_attr.set_green_size(5);
        gl_attr.set_blue_size(5);
        gl_attr.set_double_buffer(true);
        gl_attr.set_multisample_buffers(1);
        gl_attr.set_multisample_samples(4);
        gl_attr.set_context_version(3, 3);
        let (width, height) = builder.size();
        let guess_px_per_dp = {
            let (ddpi, hdpi, vdpi) = {
                video.display_dpi(0)?
            };
            let dpi = ((hdpi + vdpi) / 2.0) as f32;
            dpi / 160.0
        };
        let guess_width = width * guess_px_per_dp;
        let guess_height = height * guess_px_per_dp;
        let mut win_builder = video.window(
            builder.title(),
            guess_width as u32,
            guess_height as u32,
        );
        win_builder.opengl().allow_highdpi().resizable();
        // build window
        let mut window = win_builder.build().map_err(|err| {
            match err {
                WindowBuildError::SdlError(msg) => msg,
                _ => panic!("Unexpected window builder error!"),
            }
        })?;
        // ensure we made it at the correct size - tricky because display
        // units might be anything (vs what we care about, dp and px)
        let pixel_info: PixelInfo = {
            let pixel_info: PixelInfo = (&window).try_into()?;
            if (width - pixel_info.size.0).abs() < 1.0
                && (height - pixel_info.size.1).abs() < 1.0
            {
                pixel_info
            } else {
                let dp_per_su = pixel_info.dp_per_screen_unit;
                let calc_width = (width / dp_per_su) as u32;
                let calc_height = (height / dp_per_su) as u32;
                window.set_size(calc_width, calc_height).map_err(|err| {
                    match err {
                        sdl2::IntegerOrSdlError::SdlError(msg) => msg,
                        _ => panic!("Unexpected error resizing window!"),
                    }
                })?;
                (&window).try_into()?
            }
        };
        // setup opengl stuff
        opengl::Texture::set_loader(Some(load_texture));
        video.gl_set_swap_interval(sdl2::video::SwapInterval::VSync);
        let context = window.gl_create_context()?;
        gl::load_with(|s| video.gl_get_proc_address(s) as *const _);
        let gl_win = opengl::Window::new();
        Ok(Window {
            sdl, video, image, window, context,
            gl_win,
            pixel_info,
            title: builder.into_title(),
        })
    }
}

impl WindowSettings for Window {
    fn size(&self) -> (f32, f32) { self.pixel_info.size }
    
    fn set_size(&mut self, size: (f32, f32)) {
        let dp_per_su = self.pixel_info.dp_per_screen_unit;
        let calc_width = (size.0 / dp_per_su) as u32;
        let calc_height = (size.1 / dp_per_su) as u32;
        let _res = self.window.set_size(calc_width, calc_height);
        self.pixel_info = (&self.window).try_into().unwrap();
    }

    fn title(&self) -> &str { &self.title }

    fn set_title(&mut self, title: String) {
        let _res = self.window.set_title(&title);
        self.title = title;
    }

    fn fullscreen(&self) -> bool {
        match self.window.fullscreen_state() {
            sdl2::video::FullscreenType::Off => false,
            sdl2::video::FullscreenType::Desktop
            | sdl2::video::FullscreenType::True => true,
        }
    }

    fn set_fullscreen(&mut self, fullscreen: bool) {
        let _res = self.window.set_fullscreen(
            if fullscreen {
                sdl2::video::FullscreenType::Desktop
            } else {
                sdl2::video::FullscreenType::Off
            }
        );
    }
}

impl<'a> window::Window<'a> for Window {
    /*
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
    */

    fn pixels_per_dp(&self) -> f32 {
        self.pixel_info.pixels_per_dp
    }

    fn clear(&mut self) {
		self.gl_win.clear();
    }

    fn flip(&mut self) {
        self.gl_win.flip();
        self.window.gl_swap_window();
    }

    type Events = Events<'a>;

    fn events(&'a mut self) -> Self::Events {
        Events {
            events: self.sdl.event_pump().unwrap(),
            window: self,
            new_pixel_info: None,
        }
    }

    fn prepare_draw(&mut self) -> DrawContext {
        self.gl_win.prepare_draw(self.size())
    }
}

pub struct Events<'a> {
    window: &'a mut Window,
    events: sdl2::EventPump,
    new_pixel_info: Option<PixelInfo>,
}

impl Events<'_> {
    fn win_event(&mut self, win_event: sdl_WindowEvent)
        -> Option<WindowEvent>
    {
        Some(match win_event {
            sdl_WindowEvent::SizeChanged(_, _)
            | sdl_WindowEvent::Moved { .. } => {
                let new: PixelInfo = (&self.window.window).try_into().unwrap();
                self.window.gl_win.viewport(
                    0, 0,
                    new.pixel_size.0,
                    new.pixel_size.1,
                );
                let diff = self.window.pixel_info.diff_to_event(&new);
                if let Some(event) = diff {
                    self.new_pixel_info = Some(new);
                    event
                } else {
                    return None
                }
            }
            sdl_WindowEvent::Close => {
                WindowEvent::Quit
            }
            _ => return None,
        })
    }
}

impl Iterator for Events<'_> {
    type Item = WindowEvent;
    fn next(&mut self) -> Option<WindowEvent> {
        if let Some(new) = self.new_pixel_info.as_ref() {
            if let Some(event) = self.window.pixel_info.diff_to_event(new) {
                return Some(event);
            } else {
                self.new_pixel_info = None;
            }
        }
        while let Some(event) = self.events.poll_event() {
            return Some(match event {
                Event::Quit { .. } => WindowEvent::Quit,
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
