/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use std::convert::TryInto;

use sdl2::video::WindowBuildError;

use crate::{
    graphics::Color,
    platforms::opengl,
    window::{self},
};

#[derive(Clone, Copy, Debug)]
pub struct WindowSettings<'a> {
    pub title: &'a str,
    pub width: u32,
    pub height: u32,
    pub background_color: Color,
}

impl Default for WindowSettings<'static> {
    fn default() -> Self {
        Self {
            title: "Suzy Window",
            width: 809,
            height: 500,
            background_color: Color::from_rgba(
                0.026113365,
                0.026113365,
                0.026113365,
                1.0,
            ),
        }
    }
}

pub struct Window {
    _video: sdl2::VideoSubsystem,
    pub(super) window: sdl2::video::Window,
    _context: sdl2::video::GLContext,
    pub gl_win: opengl::Window,
}

impl Window {
    pub fn new_window(
        sdl: &sdl2::Sdl,
        settings: WindowSettings<'_>,
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
        let mut win_builder =
            video.window(settings.title, settings.width, settings.height);
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
        gl_win.clear_color(settings.background_color);
        Ok(Window {
            _video: video,
            window,
            _context: context,
            gl_win,
        })
    }

    pub fn take_screenshot(&self) -> Box<[u8]> {
        self.gl_win.take_screenshot()
    }

    pub fn flip(&self) {
        self.window.gl_swap_window();
    }

    pub fn recalculate_viewport(&mut self) {
        let [width, height] = self.physical_size();
        self.gl_win.viewport(0, 0, width, height);
    }

    pub fn set_background_color(&mut self, color: Color) {
        self.gl_win.clear_color(color);
    }

    pub(super) fn dpi(&self) -> [f32; 2] {
        let (_, hdpi, vdpi) = self
            .window
            .display_index()
            .and_then(|display_index| {
                self.window.subsystem().display_dpi(display_index)
            })
            .unwrap_or((96.0, 96.0, 96.0));
        [hdpi, vdpi]
    }

    pub(super) fn logical_size(&self) -> [f32; 2] {
        const LIMIT: u32 = 1_u32 << 24_u8;
        let (width, height) = self.window.size();
        if width > LIMIT || height > LIMIT {
            panic!("logical screen size is too big for an f32");
        }
        [width as f32, height as f32]
    }

    pub(super) fn physical_size(&self) -> [u16; 2] {
        let (width, height) = self.window.drawable_size();
        let width: u16 = width
            .try_into()
            .expect("window sizes of 2^16 and greater are not supported");
        let height: u16 = height
            .try_into()
            .expect("window sizes of 2^16 and greater are not supported");
        [width, height]
    }
}

impl window::Window<opengl::OpenGlRenderPlatform> for Window {
    fn take_screenshot(&mut self) -> Box<[u8]> {
        (*self).take_screenshot()
    }

    fn size(&self) -> [f32; 2] {
        self.logical_size()
    }
}
