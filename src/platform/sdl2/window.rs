use std::convert::{TryFrom, TryInto};

use sdl2::event::Event;
use sdl2::event::WindowEvent as sdl_WindowEvent;
use sdl2::video::WindowBuildError;

use crate::math::Color;
use crate::graphics::DrawContext;
use crate::window;
use crate::window::{WindowEvent, WindowSettings, WindowBuilder};
use crate::platform::opengl;

//use super::texture_loader::load_texture;

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
        let size_change = self.size != other.size;
        #[allow(clippy::float_cmp)]
        let dp_change = self.pixels_per_dp != other.pixels_per_dp;

        if size_change {
            self.size = other.size;
            Some(WindowEvent::Resize(self.size.0, self.size.1))
        } else if dp_change {
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
        let (_ddpi, hdpi, vdpi) = {
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

struct WindowInfo {
    pixel_info: PixelInfo,
    window: sdl2::video::Window,
    gl_win: opengl::Window,
}

pub struct Window {
    title: String,
    info: WindowInfo,
    _context: sdl2::video::GLContext,
    _video: sdl2::VideoSubsystem,
    _image: sdl2::image::Sdl2ImageContext,
    sdl: sdl2::Sdl,
    events: Option<Events>,
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
        gl_attr.set_context_profile(sdl2::video::GLProfile::GLES);
        gl_attr.set_context_version(2, 0);
        if cfg!(debug_assertions) {
            gl_attr.set_context_flags().debug().set();
        }
        let (width, height) = builder.size();
        let guess_px_per_dp = {
            let (_ddpi, hdpi, vdpi) = {
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
        //opengl::Texture::set_loader(Some(load_texture));
        let _ = video.gl_set_swap_interval(sdl2::video::SwapInterval::VSync);
        let context = window.gl_create_context()?;
        let plat_gl_context = {
            opengl::OpenGlContext::new(
                |s| video.gl_get_proc_address(s) as *const _
            )
        };
        let gl_win = opengl::Window::new(plat_gl_context);
        Ok(Window {
            title: builder.into_title(),
            info: WindowInfo {
                window,
                pixel_info,
                gl_win,
            },
            _video: video,
            _image: image,
            _context: context,
            sdl,
            events: None,
        })
    }
}

impl WindowSettings for Window {
    fn size(&self) -> (f32, f32) { self.info.pixel_info.size }
    
    fn set_size(&mut self, size: (f32, f32)) {
        let dp_per_su = self.info.pixel_info.dp_per_screen_unit;
        let calc_width = (size.0 / dp_per_su) as u32;
        let calc_height = (size.1 / dp_per_su) as u32;
        let _res = self.info.window.set_size(calc_width, calc_height);
        self.info.pixel_info = (&self.info.window).try_into().unwrap();
    }

    fn title(&self) -> &str { &self.title }

    fn set_title(&mut self, title: String) {
        let _res = self.info.window.set_title(&title);
        self.title = title;
    }

    fn fullscreen(&self) -> bool {
        match self.info.window.fullscreen_state() {
            sdl2::video::FullscreenType::Off => false,
            sdl2::video::FullscreenType::Desktop
            | sdl2::video::FullscreenType::True => true,
        }
    }

    fn set_fullscreen(&mut self, fullscreen: bool) {
        let _res = self.info.window.set_fullscreen(
            if fullscreen {
                sdl2::video::FullscreenType::Desktop
            } else {
                sdl2::video::FullscreenType::Off
            }
        );
    }

    fn background_color(&self) -> Color {
        self.info.gl_win.get_clear_color()
    }

    fn set_background_color(&mut self, color: Color) {
        self.info.gl_win.clear_color(color);
    }
}

impl window::Window<opengl::OpenGlRenderPlatform> for Window {
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
        self.info.pixel_info.pixels_per_dp
    }

    fn clear(&mut self) {
		self.info.gl_win.clear();
    }

    fn flip(&mut self) {
        self.info.gl_win.flip();
        self.info.window.gl_swap_window();
    }

    fn prepare_draw(&mut self) -> DrawContext<opengl::OpenGlRenderPlatform> {
        self.info.gl_win.prepare_draw(self.size())
    }

    fn next_event(&mut self) -> Option<WindowEvent> {
        let sdl = &self.sdl;
        let events = self.events.get_or_insert_with(|| Events {
            events: sdl.event_pump().unwrap(),
            new_pixel_info: None,
        });
        if let Some(event) = events.next(&mut self.info) {
            Some(event)
        } else {
            self.events = None;
            None
        }
    }
}

pub struct Events {
    events: sdl2::EventPump,
    new_pixel_info: Option<PixelInfo>,
}

impl Events {
    fn win_event(&mut self, window: &mut WindowInfo, win_event: sdl_WindowEvent)
        -> Option<WindowEvent>
    {
        Some(match win_event {
            sdl_WindowEvent::SizeChanged(_, _)
            | sdl_WindowEvent::Moved { .. } => {
                let new: PixelInfo = (&window.window).try_into().unwrap();
                window.gl_win.viewport(
                    0, 0,
                    new.pixel_size.0,
                    new.pixel_size.1,
                );
                let diff = window.pixel_info.diff_to_event(&new);
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

    fn next(&mut self, window: &mut WindowInfo) -> Option<WindowEvent> {
        if let Some(new) = self.new_pixel_info.as_ref() {
            if let Some(event) = window.pixel_info.diff_to_event(new) {
                return Some(event);
            } else {
                self.new_pixel_info = None;
            }
        }
        while let Some(event) = self.events.poll_event() {
            return Some(match event {
                Event::Quit { .. } => WindowEvent::Quit,
                Event::Window { win_event, .. } => {
                    match self.win_event(window, win_event) {
                        Some(ev) => ev,
                        None => continue,
                    }
                }
                Event::MouseButtonDown { mouse_btn, x, y, .. } => {
                    use sdl2::mouse::MouseButton::*;
                    use crate::pointer::*;
                    let (x, y) = (x as f32, y as f32);
                    let x = x * window.pixel_info.dp_per_screen_unit;
                    let y = y * window.pixel_info.dp_per_screen_unit;
                    let action = match mouse_btn {
                        Left => PointerAction::Down,
                        X1 => PointerAction::AltDown(AltMouseButton::X1),
                        X2 => PointerAction::AltDown(AltMouseButton::X2),
                        Middle => PointerAction::AltDown(
                            AltMouseButton::Middle
                        ),
                        Right => PointerAction::AltDown(
                            AltMouseButton::Right
                        ),
                        Unknown => continue,
                    };
                    WindowEvent::Pointer(
                        PointerEventData::new(
                            PointerId::Mouse,
                            action,
                            x, y,
                        )
                    )
                }
                Event::MouseButtonUp { mouse_btn, x, y, .. } => {
                    use sdl2::mouse::MouseButton::*;
                    use crate::pointer::*;
                    let (x, y) = (x as f32, y as f32);
                    let x = x * window.pixel_info.dp_per_screen_unit;
                    let y = y * window.pixel_info.dp_per_screen_unit;
                    let action = match mouse_btn {
                        Left => PointerAction::Up,
                        X1 => PointerAction::AltUp(AltMouseButton::X1),
                        X2 => PointerAction::AltUp(AltMouseButton::X2),
                        Middle => PointerAction::AltUp(
                            AltMouseButton::Middle
                        ),
                        Right => PointerAction::AltUp(
                            AltMouseButton::Right
                        ),
                        Unknown => continue,
                    };
                    WindowEvent::Pointer(
                        PointerEventData::new(
                            PointerId::Mouse,
                            action,
                            x, y,
                        )
                    )
                }
                _ => continue,
            })
        }
        None
    }
}
