/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

//! The OS Mesa platform uses the Mesa library to render graphics
//! offscreen, without involving a window manager.
//!
//! The OS Mesa platform does not implement an event loop, since there is
//! no window to recieve events from.  Mostly it should be used for
//! automation, e.g. tests.

use crate::{
    platform::{Event, SimpleEventLoopState},
    platforms::opengl::OpenGlRenderPlatform,
};

mod bindings;
mod window;

/// OS Mesa Platform.  See [the module level documentation](self)
/// for more.
#[derive(Debug)]
pub struct OsMesaPlatform {
    ctx: bindings::OsMesaContext,
}

impl crate::platform::Platform for OsMesaPlatform {
    type State = SimpleEventLoopState;
    type Window = window::OsMesaWindow;
    type Renderer = OpenGlRenderPlatform;

    fn new() -> Self {
        let format = 0x1908; // GL_RGBA
        let ctx = unsafe {
            bindings::OSMesaCreateContext(format, std::ptr::null_mut())
        };
        Self { ctx }
    }

    fn create_window(
        &mut self,
        settings: crate::window::WindowBuilder,
    ) -> Result<Self::Window, String> {
        Ok(window::OsMesaWindow::new(self.ctx, settings))
    }

    #[allow(clippy::unimplemented)]
    fn run<F>(self, _event_handler: F) -> !
    where
        F: 'static + FnMut(&mut Self::State, Event<'_>),
    {
        unimplemented!("Platform::run called for OSMesa Platform");
    }
}

impl Drop for OsMesaPlatform {
    fn drop(&mut self) {
        unsafe {
            bindings::OSMesaDestroyContext(self.ctx);
        }
    }
}
