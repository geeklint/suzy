/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

//! The OS Mesa platform uses the Mesa library to render graphics
//! offscreen, without involving a window manager.
//!
//! The OS Mesa platform does not implement an event loop, since there is
//! no window to recieve events from.  Mostly it should be used for
//! automation, e.g. tests.

use std::ops::{Deref, DerefMut};

use crate::{graphics::Color, platforms::opengl};

mod bindings;

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
    type Renderer = opengl::OpenGlRenderPlatform;
}

impl Default for OsMesaPlatform {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct TestEnvironment;

impl super::TestEnvironment for TestEnvironment {
    unsafe fn initialize(
        &self,
        width: u16,
        height: u16,
    ) -> Box<dyn DerefMut<Target = super::opengl::Window>> {
        const GL_RGBA: std::ffi::c_uint = 0x1908;
        const GL_UNSIGNED_BYTE: std::ffi::c_uint = 0x1401;
        let buffer = vec![0_u8; 4 * usize::from(width) * usize::from(height)];
        let buffer_ptr = Box::into_raw(buffer.into_boxed_slice());
        let ctx;
        unsafe {
            ctx = bindings::OSMesaCreateContext(GL_RGBA, std::ptr::null_mut());
            bindings::OSMesaMakeCurrent(
                ctx,
                buffer_ptr.cast(),
                GL_UNSIGNED_BYTE,
                width.into(),
                height.into(),
            );
        }
        let plat_gl_context = opengl::OpenGlContext::new(|s| {
            let name = std::ffi::CString::new(s).expect(
                "Requested OpenGL function name contained a null byte",
            );
            unsafe { bindings::OSMesaGetProcAddress(name.as_ptr()) }
        });
        let mut gl_win = opengl::Window::new(plat_gl_context);
        gl_win.clear_color(Color::BLACK);
        gl_win.viewport(0, 0, width, height);

        struct OsMesaTestEnvironment {
            buffer_ptr: *mut [u8],
            ctx: bindings::OsMesaContext,
            gl_win: opengl::Window,
        }

        impl Drop for OsMesaTestEnvironment {
            fn drop(&mut self) {
                unsafe {
                    bindings::OSMesaDestroyContext(self.ctx);
                    std::mem::drop(Box::from_raw(self.buffer_ptr));
                }
            }
        }

        impl Deref for OsMesaTestEnvironment {
            type Target = opengl::Window;

            fn deref(&self) -> &Self::Target {
                &self.gl_win
            }
        }

        impl DerefMut for OsMesaTestEnvironment {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.gl_win
            }
        }

        Box::new(OsMesaTestEnvironment {
            buffer_ptr,
            ctx,
            gl_win,
        })
    }
}
