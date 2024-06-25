/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

#![allow(missing_docs)]

use std::convert::TryFrom;

use crate::graphics::Color;

use super::{
    context::bindings::types::GLint,
    context::bindings::{
        BLEND, COLOR_BUFFER_BIT, COLOR_CLEAR_VALUE, ONE, ONE_MINUS_SRC_ALPHA,
        PACK_ALIGNMENT, RGBA, UNSIGNED_BYTE, VIEWPORT,
    },
    {Mat4, OpenGlContext, OpenGlRenderPlatform},
};

pub struct Window {
    ctx: OpenGlContext,
}

impl Window {
    /// Create an opengl window
    #[must_use]
    pub fn new(ctx: OpenGlContext) -> Self {
        Window { ctx }
    }

    pub fn clear_color(&mut self, color: Color) {
        let Color { r, g, b, a } = color;
        unsafe {
            self.ctx.bindings.ClearColor(r, g, b, a);
        }
    }

    #[must_use]
    pub fn get_clear_color(&self) -> Color {
        let mut array = [0f32; 4];
        unsafe {
            self.ctx
                .bindings
                .GetFloatv(COLOR_CLEAR_VALUE, array.as_mut_ptr());
        }
        Color::from_rgba(array[0], array[1], array[2], array[3])
    }

    pub fn viewport(&mut self, x: i16, y: i16, width: u16, height: u16) {
        unsafe {
            self.ctx.bindings.Viewport(
                x.into(),
                y.into(),
                width.into(),
                height.into(),
            );
            self.ctx.mask.configure_for_size(
                &self.ctx.bindings,
                width,
                height,
            );
        }
    }

    pub fn draw_app(
        &mut self,
        app: &mut crate::app::App<OpenGlRenderPlatform>,
    ) {
        use crate::watch::WatchedValueCore;
        let screen_width = app.state().window_width().get_unwatched();
        let screen_height = app.state().window_height().get_unwatched();
        unsafe {
            self.ctx.bindings.Enable(BLEND);
            self.ctx.bindings.BlendFunc(ONE, ONE_MINUS_SRC_ALPHA);
        }
        let matrix = Mat4::translate(-1.0, -1.0)
            * Mat4::scale(2.0 / screen_width, 2.0 / screen_height);
        app.draw(&mut super::DrawContext::gather_textures(&mut self.ctx));
        self.ctx.run_texture_populators();
        let mut current_batches = super::renderer::BatchPool::new(matrix);
        app.draw(&mut super::DrawContext::main_draw_pass(
            &mut self.ctx,
            &mut current_batches,
        ));
        super::renderer::render(&mut self.ctx, current_batches);
    }

    /// Issue opengl call to clear the screen.
    pub fn clear(&mut self) {
        unsafe {
            self.ctx.bindings.Clear(COLOR_BUFFER_BIT);
        }
    }

    #[must_use]
    pub fn take_screenshot(&self) -> Box<[u8]> {
        let mut answer: [GLint; 4] = [0; 4];
        unsafe {
            self.ctx.bindings.GetIntegerv(VIEWPORT, answer.as_mut_ptr());
            self.ctx.bindings.PixelStorei(PACK_ALIGNMENT, 1);
        }
        let x = answer[0];
        let y = answer[1];
        let width = u16::try_from(answer[2]).expect("can't take screenshot of window with a width larger than 65535 or less than 0");
        let height = u16::try_from(answer[3]).expect("can't take screenshot of window with a height larger than 65535 or less than 0");
        let pixel_size = 4;
        let buflen = pixel_size * usize::from(width) * usize::from(height);
        let mut buffer = vec![0u8; buflen].into_boxed_slice();
        unsafe {
            self.ctx.bindings.ReadPixels(
                x,
                y,
                width.into(),
                height.into(),
                RGBA,
                UNSIGNED_BYTE,
                buffer.as_mut_ptr().cast(),
            );
        }
        buffer
    }

    pub fn draw_and_take_screenshot(
        &mut self,
        app: &mut crate::app::App<OpenGlRenderPlatform>,
    ) -> Box<[u8]> {
        app.update_watches();
        self.clear();
        self.draw_app(app);
        self.take_screenshot()
    }
}
