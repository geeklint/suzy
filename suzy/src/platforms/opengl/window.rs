/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

#![allow(missing_docs)]

use crate::graphics::{Color, DrawContext};

use super::{
    context::bindings::types::*,
    context::bindings::{
        BLEND, COLOR_BUFFER_BIT, COLOR_CLEAR_VALUE, ONE_MINUS_SRC_ALPHA,
        PACK_ALIGNMENT, RGBA, SRC_ALPHA, UNSIGNED_BYTE, VIEWPORT,
    },
    {Mat4, OpenGlContext, OpenGlRenderPlatform},
};

/// opengl::Window provides a subset of the methods to implement the Window
/// trait. It can be embedded in another window implementation which
/// provides an opengl context.
pub struct Window {
    ctx: OpenGlContext,
}

impl Window {
    /// Create an opengl window
    pub fn new(ctx: OpenGlContext) -> Self {
        Window { ctx }
    }

    pub fn clear_color(&mut self, color: Color) {
        let Color { r, g, b, a } = color;
        unsafe {
            self.ctx.bindings.ClearColor(r, g, b, a);
        }
    }

    pub fn get_clear_color(&self) -> Color {
        let mut array = [0f32; 4];
        unsafe {
            self.ctx
                .bindings
                .GetFloatv(COLOR_CLEAR_VALUE, array.as_mut_ptr());
        }
        Color::from_rgba(array[0], array[1], array[2], array[3])
    }

    /// Set the viewport. Wrapping windows will probably want to do this
    /// when they detect a resize.
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

    pub fn prepare_draw(
        &mut self,
        screen_size: [f32; 2],
        first_pass: bool,
    ) -> DrawContext<'_, OpenGlRenderPlatform> {
        unsafe {
            self.ctx.bindings.Enable(BLEND);
            self.ctx.bindings.BlendFunc(SRC_ALPHA, ONE_MINUS_SRC_ALPHA);
        }
        if first_pass {
            super::DrawContext::gather_textures(&mut self.ctx)
        } else {
            let [screen_width, screen_height] = screen_size;
            let matrix = Mat4::translate(-1.0, -1.0)
                * Mat4::scale(2.0 / screen_width, 2.0 / screen_height);
            let batch_pool = super::renderer::BatchPool::new(matrix);
            super::DrawContext::main_draw_pass(&mut self.ctx, batch_pool)
        }
    }

    pub fn draw_app<P>(&mut self, app: &mut crate::app::App<P>)
    where
        P: crate::platform::Platform<Renderer = OpenGlRenderPlatform>,
    {
        use crate::watch::WatchedValueCore;
        let screen_width = app.state().window_width().get_unwatched();
        let screen_height = app.state().window_height().get_unwatched();
        unsafe {
            self.ctx.bindings.Enable(BLEND);
            self.ctx.bindings.BlendFunc(SRC_ALPHA, ONE_MINUS_SRC_ALPHA);
        }
        let matrix = Mat4::translate(-1.0, -1.0)
            * Mat4::scale(2.0 / screen_width, 2.0 / screen_height);
        app.draw(&mut super::DrawContext::gather_textures(&mut self.ctx));
        self.ctx.run_texture_populators();
        let current_batches = super::renderer::BatchPool::new(matrix);
        app.draw(&mut super::DrawContext::main_draw_pass(
            &mut self.ctx,
            current_batches,
        ));
        //super::renderer::render(&mut self.ctx, current_batches);
    }

    /// Issue opengl call to clear the screen.
    pub fn clear(&mut self) {
        unsafe {
            self.ctx.bindings.Clear(COLOR_BUFFER_BIT);
        }
    }

    pub fn take_screenshot(&self) -> Box<[u8]> {
        let mut answer: [GLint; 4] = [0; 4];
        unsafe {
            self.ctx.bindings.GetIntegerv(VIEWPORT, answer.as_mut_ptr());
            self.ctx.bindings.PixelStorei(PACK_ALIGNMENT, 1);
        }
        let x = answer[0];
        let y = answer[1];
        let width = answer[2] as GLsizei;
        let height = answer[3] as GLsizei;
        let pixel_size = 4;
        let buflen = pixel_size * (width as usize) * (height as usize);
        let mut buffer = vec![0u8; buflen].into_boxed_slice();
        unsafe {
            self.ctx.bindings.ReadPixels(
                x,
                y,
                width,
                height,
                RGBA,
                UNSIGNED_BYTE,
                buffer.as_mut_ptr().cast(),
            );
        }
        buffer
    }
}
