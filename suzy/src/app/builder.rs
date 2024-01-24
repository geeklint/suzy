/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use std::{collections::HashMap, rc::Rc};

use crate::{
    platform::Platform,
    watch::WatchContext,
    window::{WindowBuilder, WindowSettings},
};

use super::{App, AppState};

/// Enables customizing an app before it is run.
#[derive(Default)]
pub struct AppBuilder {
    win: WindowBuilder,
}

impl AppBuilder {
    /// Build the app
    pub fn build<P: Platform>(self, platform: &mut P) -> App<P> {
        let window = platform
            .create_window(self.win)
            .expect("Failed to create window");

        let [width, height] = window.size();
        let state = Rc::new(AppState::new_now(width, height));

        let watch_ctx: WatchContext<'static> = WatchContext::new();

        App {
            watch_ctx,
            window,
            roots: Vec::new(),
            pointer_grab_map: HashMap::new(),
            state,
        }
    }
}

impl WindowSettings for AppBuilder {
    fn size(&self) -> [f32; 2] {
        self.win.size()
    }

    fn set_size(&mut self, size: [f32; 2]) {
        self.win.set_size(size);
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
