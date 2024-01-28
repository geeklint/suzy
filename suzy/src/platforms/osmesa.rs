/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

//! The OS Mesa platform uses the Mesa library to render graphics
//! offscreen, without involving a window manager.
//!
//! The OS Mesa platform does not implement an event loop, since there is
//! no window to recieve events from.  Mostly it should be used for
//! automation, e.g. tests.

use crate::platforms::opengl::OpenGlRenderPlatform;

mod bindings;
mod window;

/// OS Mesa Platform.  See [the module level documentation](self)
/// for more.
#[derive(Debug)]
pub struct OsMesaPlatform {
    ctx: bindings::OsMesaContext,
}

impl Drop for OsMesaPlatform {
    fn drop(&mut self) {
        unsafe {
            bindings::OSMesaDestroyContext(self.ctx);
        }
    }
}

impl OsMesaPlatform {
    pub fn new() -> Self {
        let format = 0x1908; // GL_RGBA
        let ctx = unsafe {
            bindings::OSMesaCreateContext(format, std::ptr::null_mut())
        };
        Self { ctx }
    }
}

impl crate::platform::Platform for OsMesaPlatform {
    type Window = window::OsMesaWindow;
    type Renderer = OpenGlRenderPlatform;

    fn create_window(
        &mut self,
        settings: crate::window::WindowBuilder,
    ) -> Result<Self::Window, String> {
        let [width, height] = settings.size;
        Ok(window::OsMesaWindow::new(
            self.ctx,
            window::WindowSettings {
                width: width as u16,
                height: height as u16,
                background_color: settings.background_color,
            },
        ))
    }
}

impl Default for OsMesaPlatform {
    fn default() -> Self {
        Self::new()
    }
}
