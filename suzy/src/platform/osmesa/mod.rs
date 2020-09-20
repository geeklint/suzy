/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::{
    Event,
    SimpleEventLoopState,
};

use crate::platform::opengl::OpenGlRenderPlatform;

mod bindings;
mod window;

#[derive(Debug)]
pub struct OSMesaPlatform {
    ctx: bindings::OSMesaContext,
}

impl crate::platform::Platform for OSMesaPlatform {
    type State = SimpleEventLoopState;
    type Window = window::OSMesaWindow;
    type Renderer = OpenGlRenderPlatform;

    fn new() -> Self {
        let format = 0x1908;  // GL_RGBA
        let ctx = unsafe {
            bindings::OSMesaCreateContext(format, std::ptr::null_mut())
        };
        Self { ctx }
    }

    fn create_window(&mut self, settings: crate::window::WindowBuilder)
        -> Result<Self::Window, String>
    {
        Ok(window::OSMesaWindow::new(self.ctx, settings))
    }

    fn run<F>(self, _event_handler: F) -> !
    where
        F: 'static + FnMut(&mut Self::State, Event)
    {
        unimplemented!("Platform::run called for OSMesa Platform");
    }
}

impl Drop for OSMesaPlatform {
    fn drop(&mut self) {
        unsafe {
            bindings::OSMesaDestroyContext(self.ctx);
        }
    }
}
