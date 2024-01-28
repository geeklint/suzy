/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

//! Types associated with the creation and control of windows.

use crate::{
    graphics::{Color, DrawContext},
    platform::RenderPlatform,
};

/// A structure which defines the initial creation parameters for a window
pub struct WindowBuilder {
    pub size: [f32; 2],
    pub title: String,
    pub background_color: Color,
}

impl WindowBuilder {
    /// Consumes the window builder, returning just the title string.
    pub fn into_title(self) -> String {
        self.title
    }

    pub fn set_title(&mut self, title: String) {
        self.title = title;
    }

    pub fn set_size(&mut self, size: [f32; 2]) {
        self.size = size;
    }

    pub fn set_background_color(&mut self, color: Color) {
        self.background_color = color;
    }
}

impl Default for WindowBuilder {
    fn default() -> Self {
        Self {
            size: [1000.0, 500.0],
            title: "Suzy Window".to_string(),
            background_color: Color::from_rgba(
                0.026113365,
                0.026113365,
                0.026113365,
                1.0,
            ),
        }
    }
}

/// A trait which represents a window.
pub trait Window<P>
where
    P: ?Sized + RenderPlatform,
{
    /// Prepare to draw to this window, create a DrawContext.
    fn prepare_draw(
        &mut self,
        pass_arg: Option<P::DrawPassInfo>,
    ) -> DrawContext<'_, P>;

    /// Take a screenshot of the contents of window.
    fn take_screenshot(&self) -> Box<[u8]>;

    fn size(&self) -> [f32; 2];
}
