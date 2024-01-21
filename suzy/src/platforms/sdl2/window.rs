/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use std::convert::TryInto;

use sdl2::{
    event::{Event, WindowEvent as sdl_WindowEvent},
    video::WindowBuildError,
};

use crate::{
    graphics::{Color, DrawContext},
    platforms::opengl,
    pointer::{PointerAction, PointerEventData, PointerId},
    watch::WatchedValueCore,
    window::{self, WindowBuilder, WindowSettings},
};

#[derive(Copy, Clone, PartialEq)]
struct PixelInfo {
    dpi: [f32; 2],
    pixel_size: [u16; 2],
    screen_size: [u32; 2],
    size: [f32; 2],
}

impl From<&sdl2::video::Window> for PixelInfo {
    fn from(window: &sdl2::video::Window) -> Self {
        let (_, hdpi, vdpi) = window
            .display_index()
            .and_then(|display_index| {
                window.subsystem().display_dpi(display_index)
            })
            .unwrap_or((96.0, 96.0, 96.0));
        let (screen_width, screen_height) = window.size();
        let (px_width, px_height) = window.drawable_size();
        let px_width: u16 = px_width
            .try_into()
            .expect("window sizes of 2^16 and greater are not supported");
        let px_height: u16 = px_height
            .try_into()
            .expect("window sizes of 2^16 and greater are not supported");
        let width = f32::from(px_width);
        let height = f32::from(px_height);
        Self {
            dpi: [hdpi, vdpi],
            pixel_size: [px_width, px_height],
            screen_size: [screen_width, screen_height],
            size: [width, height],
        }
    }
}

pub(super) struct WindowInfo {
    pub(super) window: sdl2::video::Window,
    pub(super) gl_win: opengl::Window,
}

pub struct Window {
    title: String,
    pub(super) info: WindowInfo,
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
        let mut win_builder = video.window(
            builder.title(),
            requested_width as u32,
            requested_height as u32,
        );
        win_builder.opengl().allow_highdpi().resizable();
        // build window
        let window = win_builder.build().map_err(|err| match err {
            WindowBuildError::SdlError(msg) => msg,
            _ => panic!("Unexpected window builder error!"),
        })?;
        // setup opengl stuff
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
        let info: PixelInfo = (&self.info.window).into();
        info.size
    }

    fn set_size(&mut self, size: [f32; 2]) {
        let [set_width, set_height] = size;
        let _res = self
            .info
            .window
            .set_size(set_width as u32, set_height as u32);
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
    fn recalculate_viewport(&mut self) {
        let info: PixelInfo = (&self.info.window).into();
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

pub fn submit_event(
    app: &mut crate::app::App<super::SdlPlatform>,
    event: sdl2::event::Event,
    mouse_state: impl FnOnce() -> sdl2::mouse::MouseState,
) {
    use super::{AltMouseButtonResult, ToSuzyMouseButton};
    match event {
        Event::Window { win_event, .. } => {
            match win_event {
                sdl_WindowEvent::SizeChanged(_, _)
                | sdl_WindowEvent::Moved { .. } => {
                    let info: PixelInfo = (&app.window.info.window).into();
                    let [width, height] = info.size;
                    app.resize(width, height);
                    app.update_dpi(info.dpi);
                }
                sdl_WindowEvent::Leave => {
                    app.pointer_event(PointerEventData {
                        id: PointerId::Mouse,
                        action: PointerAction::Hover(f32::NAN, f32::NAN),
                        x: f32::NAN,
                        y: f32::NAN,
                    })
                }
                _ => {}
            };
        }
        Event::MouseButtonDown {
            mouse_btn, x, y, ..
        } => {
            let height = app.state().window_height().get_unwatched();
            let [x, y] = [x as f32, height - y as f32];
            let action = match mouse_btn.to_suzy_mouse_button() {
                AltMouseButtonResult::Primary => PointerAction::Down,
                AltMouseButtonResult::Alt(btn) => PointerAction::AltDown(btn),
                AltMouseButtonResult::Unknown => return,
            };
            app.pointer_event(PointerEventData {
                id: PointerId::Mouse,
                action,
                x,
                y,
            });
        }
        Event::MouseButtonUp {
            mouse_btn, x, y, ..
        } => {
            let height = app.state().window_height().get_unwatched();
            let [x, y] = [x as f32, height - y as f32];
            let action = match mouse_btn.to_suzy_mouse_button() {
                AltMouseButtonResult::Primary => PointerAction::Up,
                AltMouseButtonResult::Alt(btn) => PointerAction::AltUp(btn),
                AltMouseButtonResult::Unknown => return,
            };
            app.pointer_event(PointerEventData {
                id: PointerId::Mouse,
                action,
                x,
                y,
            });
        }
        Event::MouseMotion {
            mousestate,
            x,
            y,
            xrel,
            yrel,
            ..
        } => {
            let height = app.state().window_height().get_unwatched();
            let [x, y] = [x as f32, height - y as f32];
            let [xrel, yrel] = [xrel as f32, -(yrel as f32)];
            let pointer_event = if mousestate.left() {
                PointerEventData {
                    id: PointerId::Mouse,
                    action: PointerAction::Move(xrel, yrel),
                    x,
                    y,
                }
            } else {
                PointerEventData {
                    id: PointerId::Mouse,
                    action: PointerAction::Hover(xrel, yrel),
                    x,
                    y,
                }
            };
            app.pointer_event(pointer_event);
        }
        Event::MouseWheel { x, y, .. } => {
            let height = app.state().window_height().get_unwatched();
            let state = mouse_state();
            let xrel = x as f32 * 125.0;
            let yrel = -(y as f32 * 125.0);
            let x = state.x() as f32;
            let y = height - state.y() as f32;
            app.pointer_event(PointerEventData {
                id: PointerId::Mouse,
                action: PointerAction::Wheel(xrel, yrel),
                x,
                y,
            });
        }
        _ => {}
    }
}
