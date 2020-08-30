/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::math::Color;
use crate::graphics::DrawContext;

use super::{
    OpenGlRenderPlatform,
    OpenGlContext,
    Mat4,
};
use super::context::bindings::types::*;
use super::context::bindings::{
    COLOR_BUFFER_BIT,
    BLEND,
    SRC_ALPHA,
    ONE_MINUS_SRC_ALPHA,
    COLOR_CLEAR_VALUE,
    PACK_ALIGNMENT,
    VIEWPORT,
    RGBA,
    UNSIGNED_BYTE,
};
use super::drawparams::DrawParams;

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
        let (r, g, b, a) = color.rgba();
        unsafe {
            self.ctx.bindings.ClearColor(r, g, b, a);
        }
    }

    pub fn get_clear_color(&self) -> Color {
        let mut array = [0f32; 4];
        unsafe {
            self.ctx.bindings.GetFloatv(
                COLOR_CLEAR_VALUE,
                array.as_mut_ptr(),
            );
        }
        Color::create_rgba(array[0], array[1], array[2], array[3])
    }

    /// Set the viewport. Wrapping windows will probably want to do this
    /// when they detect a resize.
    pub fn viewport(&mut self, x: i32, y: i32, width: u32, height: u32) {
        unsafe {
            self.ctx.bindings.Viewport(
                x as GLint,
                y as GLint,
                width as GLsizei,
                height as GLsizei,
            );
        }
    }

    pub fn prepare_draw(&mut self, screen_size: (f32, f32), first_pass: bool)
        -> DrawContext<OpenGlRenderPlatform>
    {
        unsafe {
            self.ctx.bindings.Enable(BLEND);
            self.ctx.bindings.BlendFunc(SRC_ALPHA, ONE_MINUS_SRC_ALPHA);
        }
        let mut params = DrawParams::new();
        params.transform(
            Mat4::translate(-1.0, -1.0)
            * Mat4::scale(2.0 / screen_size.0, 2.0 / screen_size.1)
        );
        DrawContext::new(&mut self.ctx, params, first_pass)
    }

    /// Issue opengl call to clear the screen.
    pub fn clear(&mut self) {
        unsafe {
            self.ctx.bindings.Clear(COLOR_BUFFER_BIT);
        }
    }

    /// This function does not block like window::Window requires.
    pub fn flip(&mut self) {
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
                x, y,
                width, height, 
                RGBA,
                UNSIGNED_BYTE,
                buffer.as_mut_ptr() as *mut _,
            );
        }
        buffer
    }
}
