/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

//! Types associated with the creation and control of windows.

use crate::{
    graphics::{Color, DrawContext},
    platform::RenderPlatform,
};

/// A trait which represents the settings a window might have.
pub trait WindowSettings {
    /// Get the size of the window in dp
    fn size(&self) -> [f32; 2];

    /// Set the size of the window in dp
    fn set_size(&mut self, size: [f32; 2]);

    /// Set the window title
    fn set_title(&mut self, title: String);

    /// Get the window fullscreen state
    fn fullscreen(&self) -> bool;

    /// Set the fullscreen state
    fn set_fullscreen(&mut self, fullscreen: bool);

    /// Get the window background color
    fn background_color(&self) -> Color;

    /// Set the window background color
    fn set_background_color(&mut self, color: Color);
}

/// A structure which defines the initial creation parameters for a window
pub struct WindowBuilder {
    size: [f32; 2],
    title: String,
    fullscreen: bool,
    background_color: Color,
}

impl WindowBuilder {
    /// Consumes the window builder, returning just the title string.
    pub fn into_title(self) -> String {
        self.title
    }
}

impl Default for WindowBuilder {
    fn default() -> Self {
        Self {
            size: [1000.0, 500.0],
            title: "Suzy Window".to_string(),
            fullscreen: false,
            background_color: Color::from_rgba(
                0.026113365,
                0.026113365,
                0.026113365,
                1.0,
            ),
        }
    }
}

impl WindowSettings for WindowBuilder {
    fn size(&self) -> [f32; 2] {
        self.size
    }

    fn set_size(&mut self, size: [f32; 2]) {
        self.size = size;
    }

    fn set_title(&mut self, title: String) {
        self.title = title;
    }

    fn fullscreen(&self) -> bool {
        self.fullscreen
    }

    fn set_fullscreen(&mut self, fullscreen: bool) {
        self.fullscreen = fullscreen;
    }

    fn background_color(&self) -> Color {
        self.background_color
    }

    fn set_background_color(&mut self, color: Color) {
        self.background_color = color;
    }
}

/// A trait which represents a window.
pub trait Window<P>: WindowSettings
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
}
