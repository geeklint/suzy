/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::convert::{TryFrom, TryInto};

use sdl2::event::Event;
use sdl2::event::WindowEvent as sdl_WindowEvent;
use sdl2::video::WindowBuildError;

use crate::graphics::Color;
use crate::graphics::DrawContext;
use crate::window;
use crate::window::{WindowEvent, WindowSettings, WindowBuilder};
use crate::platform::opengl;
use crate::pointer::{
    PointerAction,
    PointerEventData,
};

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

impl TryFrom<&sdl2::video::Window> for PixelInfo {
    type Error = String;

    fn try_from(window: &sdl2::video::Window) -> Result<Self, Self::Error> {
        let display_index = window.display_index()?;
        let (_ddpi, hdpi, vdpi) = {
            window.subsystem().display_dpi(display_index)?
        };
        let dpi = ((hdpi + vdpi) / 2.0) as f32;
        let pixels_per_dp = dpi / crate::units::DPI;
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
    window: sdl2::video::Window,
    gl_win: opengl::Window,
}

pub struct Window {
    title: String,
    info: WindowInfo,
    _context: sdl2::video::GLContext,
    _video: sdl2::VideoSubsystem,
}

impl Window {
    pub fn new_window(sdl: &sdl2::Sdl, builder: WindowBuilder)
        -> Result<Self, String>
    {
        // initialize systems
        let video = sdl.video()?;
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
            dpi / crate::units::DPI
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
        {
            let pixel_info: PixelInfo = (&window).try_into()?;
            if (width - pixel_info.size.0).abs() >= 1.0
                || (height - pixel_info.size.1).abs() >= 1.0
            {
                let dp_per_su = pixel_info.dp_per_screen_unit;
                let calc_width = (width / dp_per_su) as u32;
                let calc_height = (height / dp_per_su) as u32;
                window.set_size(calc_width, calc_height).map_err(|err| {
                    match err {
                        sdl2::IntegerOrSdlError::SdlError(msg) => msg,
                        _ => panic!("Unexpected error resizing window!"),
                    }
                })?;
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
                gl_win,
            },
            _video: video,
            _context: context,
        })
    }
}

impl WindowSettings for Window {
    fn size(&self) -> (f32, f32) {
        let info: PixelInfo = (&self.info.window).try_into().unwrap();
        info.size
    }
    
    fn set_size(&mut self, size: (f32, f32)) {
        let info: PixelInfo = (&self.info.window).try_into().unwrap();
        let dp_per_su = info.dp_per_screen_unit;
        let calc_width = (size.0 / dp_per_su) as u32;
        let calc_height = (size.1 / dp_per_su) as u32;
        let _res = self.info.window.set_size(calc_width, calc_height);
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
        let info: PixelInfo = (&self.info.window).try_into().unwrap();
        info.pixels_per_dp
    }

    fn normalize_pointer_event(&self, event: &mut PointerEventData) {
        let info: PixelInfo = (&self.info.window).try_into().unwrap();
        event.x *= info.dp_per_screen_unit;
        event.y *= info.dp_per_screen_unit;
        event.y = info.size.1 - event.y;
        match event.action {
            PointerAction::Move(ref mut x, ref mut y) => {
                *x *= info.dp_per_screen_unit;
                *y *= info.dp_per_screen_unit;
                *y = info.size.1 - *y;
            },
            PointerAction::Wheel(ref mut x, ref mut y) => {
                *x *= info.dp_per_screen_unit;
                *y *= info.dp_per_screen_unit;
                *y = info.size.1 - *y;
            },
            PointerAction::Hover(ref mut x, ref mut y) => {
                *x *= info.dp_per_screen_unit;
                *y *= info.dp_per_screen_unit;
                *y = info.size.1 - *y;
            },
            _ => (),
        }
        event.normalized = true;
    }

    fn recalculate_viewport(&mut self) {
        let info: PixelInfo = (&self.info.window).try_into().unwrap();
        self.info.gl_win.viewport(
            0,
            0,
            info.pixel_size.0,
            info.pixel_size.1,
        );
    }

    fn flip(&mut self) {
        self.info.gl_win.flip();
        self.info.window.gl_swap_window();
    }

    fn prepare_draw(&mut self, first_pass: bool)
        -> DrawContext<opengl::OpenGlRenderPlatform>
    {
        self.info.gl_win.clear();
        self.info.gl_win.prepare_draw(self.size(), first_pass)
    }

    fn take_screenshot(&self) -> Box<[u8]> {
        self.info.gl_win.take_screenshot()
    }
}

pub struct Events {
    pub(super) events: sdl2::EventPump,
    pub(super) send_dp: bool,
}

impl Events {
    fn win_event(&mut self, win_event: sdl_WindowEvent)
        -> Option<WindowEvent>
    {
        Some(match win_event {
            sdl_WindowEvent::SizeChanged(_, _)
            | sdl_WindowEvent::Moved { .. } => {
                self.send_dp = true;
                WindowEvent::Resize
            }
            sdl_WindowEvent::Close => {
                WindowEvent::Quit
            }
            sdl_WindowEvent::Leave => {
                WindowEvent::Pointer(
                    PointerEventData::new(
                        crate::pointer::PointerId::Mouse,
                        PointerAction::Hover(f32::NAN, f32::NAN),
                        f32::NAN, f32::NAN,
                    )
                )
            }
            _ => return None,
        })
    }

    pub fn next(&mut self) -> Option<WindowEvent> {
        if self.send_dp {
            self.send_dp = false;
            return Some(WindowEvent::DpScaleChange);
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
                Event::MouseButtonDown { mouse_btn, x, y, .. } => {
                    use sdl2::mouse::MouseButton::*;
                    use crate::pointer::*;
                    let (x, y) = (x as f32, y as f32);
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
                Event::MouseMotion { mousestate, x, y, xrel, yrel, .. } => {
                    use crate::pointer::*;
                    let (x, y) = (x as f32, y as f32);
                    let (xrel, yrel) = (xrel as f32, yrel as f32);
                    if mousestate.left() {
                        WindowEvent::Pointer(
                            PointerEventData::new(
                                PointerId::Mouse,
                                PointerAction::Move(xrel, yrel),
                                x, y,
                            )
                        )
                    } else {
                        WindowEvent::Pointer(
                            PointerEventData::new(
                                PointerId::Mouse,
                                PointerAction::Hover(xrel, yrel),
                                x, y,
                            )
                        )
                    }
                }
                _ => continue,
            })
        }
        None
    }
}
