/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::time;

use drying_paint::Watched;

use crate::platform::Platform;
use crate::window::{WindowBuilder, WindowSettings};

use super::{get_cell_size, App, AppValues};

/// Enables customizing an app before it is run.
#[derive(Default)]
pub struct AppBuilder {
    win: WindowBuilder,
}

impl AppBuilder {
    /// Build the app
    pub fn build<P: Platform>(self) -> App<P> {
        let mut platform = P::new();
        let window = platform
            .create_window(self.win)
            .expect("Failed to create window");
        let watch_ctx = drying_paint::WatchContext::new();

        let (width, height) = window.size();

        let now = time::Instant::now();

        let values = AppValues {
            frame_start: Watched::new(now),
            coarse_time: Watched::new(now),
            cell_size: Watched::new(get_cell_size(width, height)),
            px_per_dp: Watched::new(1.0),
            window_size: (width, height),
        };
        App {
            platform,
            watch_ctx,
            window,
            roots: Vec::new(),
            values,
            pointer_grab_map: HashMap::new(),
        }
    }
}

impl WindowSettings for AppBuilder {
    fn size(&self) -> (f32, f32) {
        self.win.size()
    }

    fn set_size(&mut self, size: (f32, f32)) {
        self.win.set_size(size);
    }

    fn title(&self) -> &str {
        self.win.title()
    }

    fn set_title(&mut self, title: String) {
        self.win.set_title(title);
    }

    fn fullscreen(&self) -> bool {
        self.win.fullscreen()
    }

    fn set_fullscreen(&mut self, fullscreen: bool) {
        self.win.set_fullscreen(fullscreen);
    }

    fn background_color(&self) -> crate::graphics::Color {
        self.win.background_color()
    }

    fn set_background_color(&mut self, color: crate::graphics::Color) {
        self.win.set_background_color(color);
    }
}
