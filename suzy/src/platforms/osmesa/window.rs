/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use crate::{graphics::Color, graphics::DrawContext, window::Window};

use crate::platforms::opengl;

use super::bindings;

#[derive(Clone, Copy, Debug)]
pub struct WindowSettings {
    pub width: u16,
    pub height: u16,
    pub background_color: Color,
}

impl Default for WindowSettings {
    fn default() -> Self {
        Self {
            width: 512,
            height: 512,
            background_color: Color::BLACK,
        }
    }
}

pub struct OsMesaWindow {
    width: u16,
    height: u16,
    gl_win: opengl::Window,
    _buffer_ptr: *mut [u8],
}

impl OsMesaWindow {
    pub(super) fn new(
        ctx: bindings::OsMesaContext,
        settings: WindowSettings,
    ) -> Self {
        let buffer = vec![
            0_u8;
            4 * usize::from(settings.width)
                * usize::from(settings.height)
        ];
        let buffer_ptr = Box::into_raw(buffer.into_boxed_slice());
        unsafe {
            bindings::OSMesaMakeCurrent(
                ctx,
                buffer_ptr.cast(),
                0x1401, // GL_UNSIGNED_BYTE
                settings.width.into(),
                settings.height.into(),
            );
        }
        let plat_gl_context = opengl::OpenGlContext::new(|s| {
            let name = std::ffi::CString::new(s).expect(
                "Requested OpenGL function name contained a null byte",
            );
            unsafe { bindings::OSMesaGetProcAddress(name.as_ptr()) }
        });
        let mut gl_win = opengl::Window::new(plat_gl_context);
        gl_win.clear_color(settings.background_color);
        gl_win.viewport(0, 0, settings.width, settings.height);
        Self {
            width: settings.width,
            height: settings.height,
            gl_win,
            _buffer_ptr: buffer_ptr,
        }
    }
}

impl Window<opengl::OpenGlRenderPlatform> for OsMesaWindow {
    fn prepare_draw(
        &mut self,
        frame_arg: Option<()>,
    ) -> DrawContext<'_, opengl::OpenGlRenderPlatform> {
        let size = [self.width.into(), self.height.into()];
        self.gl_win.clear();
        self.gl_win.prepare_draw(size, frame_arg.is_none())
    }

    fn take_screenshot(&self) -> Box<[u8]> {
        self.gl_win.take_screenshot()
    }

    fn size(&self) -> [f32; 2] {
        [self.width.into(), self.height.into()]
    }
}
