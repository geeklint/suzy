/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use crate::{
    graphics::Color,
    graphics::DrawContext,
    window::{Window, WindowBuilder, WindowSettings},
};

use crate::platforms::opengl;

use super::bindings;

pub struct OsMesaWindow {
    size: [u16; 2],
    gl_win: opengl::Window,
    buffer: Vec<u8>,
}

impl OsMesaWindow {
    pub(super) fn new(
        ctx: bindings::OsMesaContext,
        builder: WindowBuilder,
    ) -> Self {
        let [width, height] = builder.size();
        let background_color = builder.background_color();
        let width = width.max(1.0).min(1280.0) as u16;
        let height = height.max(1.0).min(1024.0) as u16;
        let mut buffer =
            vec![0_u8; 4 * usize::from(width) * usize::from(height)];
        unsafe {
            bindings::OSMesaMakeCurrent(
                ctx,
                buffer.as_mut_ptr().cast(),
                0x1401, // GL_UNSIGNED_BYTE
                width as _,
                height as _,
            );
        }
        let plat_gl_context = opengl::OpenGlContext::new(|s| {
            let name = std::ffi::CString::new(s).expect(
                "Requested OpenGL function name contained a null byte",
            );
            unsafe { bindings::OSMesaGetProcAddress(name.as_ptr()) }
        });
        let mut gl_win = opengl::Window::new(plat_gl_context);
        gl_win.clear_color(background_color);
        gl_win.viewport(0, 0, width, height);
        Self {
            size: [width, height],
            gl_win,
            buffer,
        }
    }
}

impl WindowSettings for OsMesaWindow {
    fn size(&self) -> [f32; 2] {
        let [width, height] = self.size;
        [width.into(), height.into()]
    }

    fn set_size(&mut self, size: [f32; 2]) {
        let [width, height] = size;
        let width = width.max(1.0).min(1280.0) as u16;
        let height = height.max(1.0).min(1024.0) as u16;
        self.size = [width, height];
        let bufsize = 4 * (width as usize) * (height as usize);
        self.buffer.resize(bufsize, 0);
    }

    fn background_color(&self) -> Color {
        self.gl_win.get_clear_color()
    }

    fn set_background_color(&mut self, color: Color) {
        self.gl_win.clear_color(color);
    }
}

impl Window<opengl::OpenGlRenderPlatform> for OsMesaWindow {
    fn prepare_draw(
        &mut self,
        frame_arg: Option<()>,
    ) -> DrawContext<'_, opengl::OpenGlRenderPlatform> {
        self.gl_win.clear();
        self.gl_win.prepare_draw(self.size(), frame_arg.is_none())
    }

    fn take_screenshot(&self) -> Box<[u8]> {
        self.gl_win.take_screenshot()
    }
}
