/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use std::convert::{TryFrom, TryInto};

use sdl2::{
    event::{Event, WindowEvent as sdl_WindowEvent},
    video::WindowBuildError,
};

use crate::{
    graphics::{Color, DrawContext},
    platforms::opengl,
    pointer::{AltMouseButton, PointerAction, PointerEventData, PointerId},
    window::{self, WindowBuilder, WindowSettings},
};

//use super::texture_loader::load_texture;

#[derive(Copy, Clone, PartialEq)]
struct PixelInfo {
    display_index: i32,
    pixels_per_dp: f32,
    dp_per_screen_unit: f32,
    pixel_size: [u16; 2],
    screen_size: [u32; 2],
    size: [f32; 2],
}

impl TryFrom<&sdl2::video::Window> for PixelInfo {
    type Error = String;

    fn try_from(window: &sdl2::video::Window) -> Result<Self, Self::Error> {
        let display_index = window.display_index()?;
        let (_ddpi, hdpi, vdpi) = window
            .subsystem()
            .display_dpi(display_index)
            .unwrap_or((1.0, 1.0, 1.0));
        let dpi = (hdpi + vdpi) / 2.0;
        let pixels_per_dp = dpi / crate::units::DPI;
        let (screen_width, screen_height) = window.size();
        let (px_width, px_height) = window.drawable_size();
        let px_width: u16 = px_width
            .try_into()
            .expect("window sizes of 2^16 and greater are not supported");
        let px_height: u16 = px_height
            .try_into()
            .expect("window sizes of 2^16 and greater are not supported");
        let x_px_per_su = f32::from(px_width) / (screen_width as f32);
        let y_px_per_su = f32::from(px_height) / (screen_height as f32);
        let px_per_su = (x_px_per_su + y_px_per_su) / 2.0;
        let width = f32::from(px_width);
        let height = f32::from(px_height);
        Ok(Self {
            display_index,
            pixels_per_dp,
            dp_per_screen_unit: 1.0 / (pixels_per_dp * px_per_su),
            pixel_size: [px_width, px_height],
            screen_size: [screen_width, screen_height],
            size: [width, height],
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
    pub fn new_window(
        sdl: &sdl2::Sdl,
        builder: WindowBuilder,
    ) -> Result<Self, String> {
        // initialize systems
        let video = sdl.video()?;
        // setup window parameters
        let gl_attr = video.gl_attr();
        gl_attr.set_red_size(5);
        gl_attr.set_green_size(5);
        gl_attr.set_blue_size(5);
        gl_attr.set_framebuffer_srgb_compatible(true);
        gl_attr.set_double_buffer(true);
        gl_attr.set_multisample_buffers(1);
        gl_attr.set_multisample_samples(4);
        gl_attr.set_context_profile(sdl2::video::GLProfile::GLES);
        gl_attr.set_context_version(2, 0);
        if opengl::DEBUG {
            gl_attr.set_context_flags().debug().set();
        }
        let [requested_width, requested_height] = builder.size();
        let guess_px_per_dp = {
            let (_ddpi, hdpi, vdpi) =
                video.display_dpi(0).unwrap_or((1.0, 1.0, 1.0));
            let dpi = (hdpi + vdpi) / 2.0;
            dpi / crate::units::DPI
        };
        let guess_width = requested_width * guess_px_per_dp;
        let guess_height = requested_height * guess_px_per_dp;
        let mut win_builder = video.window(
            builder.title(),
            guess_width as u32,
            guess_height as u32,
        );
        win_builder.opengl().allow_highdpi().resizable();
        // build window
        let mut window = win_builder.build().map_err(|err| match err {
            WindowBuildError::SdlError(msg) => msg,
            _ => panic!("Unexpected window builder error!"),
        })?;
        // ensure we made it at the correct size - tricky because display
        // units might be anything (vs what we care about, dp and px)
        {
            let pixel_info: PixelInfo = (&window).try_into()?;
            let [width, height] = pixel_info.size;
            if (requested_width - width).abs() >= 1.0
                || (requested_height - height).abs() >= 1.0
            {
                let dp_per_su = pixel_info.dp_per_screen_unit;
                let calc_width = (requested_width / dp_per_su) as u32;
                let calc_height = (requested_height / dp_per_su) as u32;
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
            opengl::OpenGlContext::new(|s| video.gl_get_proc_address(s).cast())
        };
        let mut gl_win = opengl::Window::new(plat_gl_context);
        gl_win.clear_color(builder.background_color());
        Ok(Window {
            title: builder.into_title(),
            info: WindowInfo { window, gl_win },
            _video: video,
            _context: context,
        })
    }
}

impl WindowSettings for Window {
    fn size(&self) -> [f32; 2] {
        let info: PixelInfo = (&self.info.window)
            .try_into()
            .expect("Unable to get pixel info from current SDL window");
        info.size
    }

    fn set_size(&mut self, size: [f32; 2]) {
        let info: PixelInfo = (&self.info.window)
            .try_into()
            .expect("Unable to get pixel info from current SDL window");
        let [set_width, set_height] = size;
        let dp_per_su = info.dp_per_screen_unit;
        let calc_width = (set_width / dp_per_su) as u32;
        let calc_height = (set_height / dp_per_su) as u32;
        let _res = self.info.window.set_size(calc_width, calc_height);
    }

    fn title(&self) -> &str {
        &self.title
    }

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
        let _res = self.info.window.set_fullscreen(if fullscreen {
            sdl2::video::FullscreenType::Desktop
        } else {
            sdl2::video::FullscreenType::Off
        });
    }

    fn background_color(&self) -> Color {
        self.info.gl_win.get_clear_color()
    }

    fn set_background_color(&mut self, color: Color) {
        self.info.gl_win.clear_color(color);
    }
}

impl window::Window<opengl::OpenGlRenderPlatform> for Window {
    fn pixels_per_dp(&self) -> f32 {
        let info: PixelInfo = (&self.info.window)
            .try_into()
            .expect("Unable to get pixel info from current SDL window");
        info.pixels_per_dp
    }

    fn normalize_pointer_event(&self, event: &mut PointerEventData) {
        let info: PixelInfo = (&self.info.window)
            .try_into()
            .expect("Unable to get pixel info from current SDL window");
        event.x *= info.dp_per_screen_unit;
        event.y *= info.dp_per_screen_unit;
        event.y = info.size[1] - event.y;
        match event.action {
            PointerAction::Move(ref mut x, ref mut y) => {
                *x *= info.dp_per_screen_unit;
                *y *= -info.dp_per_screen_unit;
            }
            PointerAction::Wheel(ref mut _x, ref mut y) => {
                *y *= -1.0;
            }
            PointerAction::Hover(ref mut x, ref mut y) => {
                *x *= info.dp_per_screen_unit;
                *y *= -info.dp_per_screen_unit;
            }
            _ => (),
        }
        event.normalized = true;
    }

    fn recalculate_viewport(&mut self) {
        let info: PixelInfo = (&self.info.window)
            .try_into()
            .expect("Unable to get pixel info from current SDL window");
        let [pixel_width, pixel_height] = info.pixel_size;
        self.info.gl_win.viewport(0, 0, pixel_width, pixel_height);
    }

    fn flip(&mut self) {
        self.info.gl_win.flip();
        self.info.window.gl_swap_window();
    }

    fn prepare_draw(
        &mut self,
        pass_arg: Option<()>,
    ) -> DrawContext<'_, opengl::OpenGlRenderPlatform> {
        let first_pass = pass_arg.is_none();
        if first_pass {
            self.info.gl_win.clear();
        }
        self.info.gl_win.prepare_draw(self.size(), first_pass)
    }

    fn take_screenshot(&self) -> Box<[u8]> {
        self.info.gl_win.take_screenshot()
    }
}

fn handle_win_event(
    win_event: sdl_WindowEvent,
    state: &mut crate::platform::SimpleEventLoopState,
    app: &mut crate::app::App,
) {
    match win_event {
        sdl_WindowEvent::SizeChanged(_, _) | sdl_WindowEvent::Moved { .. } => {
            app.update_window_size();
            app.update_scale_factor();
        }
        sdl_WindowEvent::Close => state.running = false,
        sdl_WindowEvent::Leave => app.pointer_event(PointerEventData {
            id: PointerId::Mouse,
            action: PointerAction::Hover(f32::NAN, f32::NAN),
            x: f32::NAN,
            y: f32::NAN,
            normalized: true,
        }),
        _ => {}
    }
}

pub fn pump(
    events: &mut sdl2::EventPump,
    state: &mut crate::platform::SimpleEventLoopState,
    app: &mut crate::app::App,
) {
    while let Some(event) = events.poll_event() {
        match event {
            Event::Quit { .. } => state.running = false,
            Event::Window { win_event, .. } => {
                handle_win_event(win_event, state, app);
            }
            Event::MouseButtonDown {
                mouse_btn, x, y, ..
            } => {
                let [x, y] = [x as f32, y as f32];
                let action = match to_alt_mouse_button(mouse_btn) {
                    AltMouseButtonResult::Primary => PointerAction::Down,
                    AltMouseButtonResult::Alt(btn) => {
                        PointerAction::AltDown(btn)
                    }
                    AltMouseButtonResult::Unknown => continue,
                };
                app.pointer_event(PointerEventData::new(
                    PointerId::Mouse,
                    action,
                    x,
                    y,
                ));
            }
            Event::MouseButtonUp {
                mouse_btn, x, y, ..
            } => {
                let [x, y] = [x as f32, y as f32];
                let action = match to_alt_mouse_button(mouse_btn) {
                    AltMouseButtonResult::Primary => PointerAction::Up,
                    AltMouseButtonResult::Alt(btn) => {
                        PointerAction::AltUp(btn)
                    }
                    AltMouseButtonResult::Unknown => continue,
                };
                app.pointer_event(PointerEventData::new(
                    PointerId::Mouse,
                    action,
                    x,
                    y,
                ));
            }
            Event::MouseMotion {
                mousestate,
                x,
                y,
                xrel,
                yrel,
                ..
            } => {
                let [x, y] = [x as f32, y as f32];
                let [xrel, yrel] = [xrel as f32, yrel as f32];
                let pointer_event = if mousestate.left() {
                    PointerEventData::new(
                        PointerId::Mouse,
                        PointerAction::Move(xrel, yrel),
                        x,
                        y,
                    )
                } else {
                    PointerEventData::new(
                        PointerId::Mouse,
                        PointerAction::Hover(xrel, yrel),
                        x,
                        y,
                    )
                };
                app.pointer_event(pointer_event);
            }
            Event::MouseWheel { x, y, .. } => {
                let state = events.mouse_state();
                let [xrel, yrel] = [x as f32, y as f32];
                let x = state.x() as f32;
                let y = state.y() as f32;
                app.pointer_event(PointerEventData::new(
                    PointerId::Mouse,
                    PointerAction::Wheel(xrel, yrel),
                    x,
                    y,
                ));
            }
            _ => continue,
        }
    }
}

enum AltMouseButtonResult {
    Primary,
    Alt(AltMouseButton),
    Unknown,
}

fn to_alt_mouse_button(
    button: sdl2::mouse::MouseButton,
) -> AltMouseButtonResult {
    use AltMouseButtonResult::*;
    match button {
        sdl2::mouse::MouseButton::Unknown => Unknown,
        sdl2::mouse::MouseButton::Left => Primary,
        sdl2::mouse::MouseButton::Middle => Alt(AltMouseButton::Middle),
        sdl2::mouse::MouseButton::Right => Alt(AltMouseButton::Right),
        sdl2::mouse::MouseButton::X1 => Alt(AltMouseButton::X1),
        sdl2::mouse::MouseButton::X2 => Alt(AltMouseButton::X2),
    }
}
