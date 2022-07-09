/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use crate::{
    dims::Rect,
    platform::{Platform, RenderPlatform, SimpleEventLoopState},
    window::WindowSettings,
};

#[derive(Clone, Copy, Debug, Default)]
pub struct NoGraphics;

#[derive(Clone, Debug)]
pub struct Window {
    title: String,
    size: (f32, f32),
    fullscreen: bool,
    background_color: crate::graphics::Color,
}

pub enum Texture {}
pub enum Graphic {}

impl Platform for NoGraphics {
    type State = SimpleEventLoopState;

    type Renderer = Self;

    type Window = Window;

    fn new() -> Self {
        Self
    }

    fn create_window(
        &mut self,
        settings: crate::window::WindowBuilder,
    ) -> Result<Self::Window, String> {
        Ok(Window {
            size: settings.size(),
            fullscreen: settings.fullscreen(),
            background_color: settings.background_color(),
            title: settings.into_title(),
        })
    }

    fn run<F>(self, _event_handlerr: F) -> !
    where
        F: 'static + FnMut(&mut Self::State, crate::platform::Event<'_>),
    {
        unimplemented!(
            "NoGraphics platform has no concept of running as an app"
        );
    }
}

impl RenderPlatform for NoGraphics {
    type DrawPassInfo = ();

    type DrawContextBuilder = fn(&mut ()) -> Self;

    type Texture = Texture;

    type SlicedImage = Graphic;

    type SelectableSlicedImage = Graphic;

    type Text = Graphic;

    type TextEdit = Graphic;
}

impl crate::graphics::PlatformDrawContext<()> for NoGraphics {
    fn finish(self) -> Option<()> {
        None
    }
}

impl WindowSettings for Window {
    fn size(&self) -> (f32, f32) {
        self.size
    }

    fn set_size(&mut self, size: (f32, f32)) {
        self.size = size;
    }

    fn title(&self) -> &str {
        &self.title
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

    fn background_color(&self) -> crate::graphics::Color {
        self.background_color
    }

    fn set_background_color(&mut self, color: crate::graphics::Color) {
        self.background_color = color;
    }
}

impl crate::window::Window<NoGraphics> for Window {
    fn pixels_per_dp(&self) -> f32 {
        1.0
    }

    fn normalize_pointer_event(
        &self,
        event: &mut crate::pointer::PointerEventData,
    ) {
        event.normalized = true;
    }

    fn recalculate_viewport(&mut self) {}

    fn flip(&mut self) {}

    fn prepare_draw(
        &mut self,
        _pass_arg: Option<()>,
    ) -> crate::graphics::DrawContext<'_, NoGraphics> {
        NoGraphics
    }

    fn take_screenshot(&self) -> Box<[u8]> {
        unimplemented!("Can't take screenshot with a NoGraphics window");
    }
}

impl Default for Texture {
    fn default() -> Self {
        unimplemented!("Can't constuct a texture for NoGraphics platform")
    }
}

impl crate::platform::graphics::Texture for Texture {
    fn load_static(
        _width: u16,
        _height: u16,
        _alignment: u16,
        _pixels: &'static [u8],
    ) -> Self {
        unimplemented!("Can't constuct a texture for NoGraphics platform")
    }
}

impl Default for Graphic {
    fn default() -> Self {
        unimplemented!("Can't construct a graphic for the NoGraphics platform")
    }
}

impl crate::graphics::Graphic<NoGraphics> for Graphic {
    fn draw(
        &mut self,
        _ctx: &mut crate::graphics::DrawContext<'_, NoGraphics>,
    ) {
        unreachable!()
    }
}

impl Rect for Graphic {
    fn x(&self) -> crate::dims::Dim {
        unreachable!()
    }

    fn y(&self) -> crate::dims::Dim {
        unreachable!()
    }

    fn x_mut<F, R>(&mut self, _f: F) -> R
    where
        F: FnOnce(&mut crate::dims::Dim) -> R,
    {
        unreachable!()
    }

    fn y_mut<F, R>(&mut self, _f: F) -> R
    where
        F: FnOnce(&mut crate::dims::Dim) -> R,
    {
        unreachable!()
    }
}

impl crate::selectable::Selectable for Graphic {
    fn selection_changed(
        &mut self,
        _state: crate::selectable::SelectionState,
    ) {
        unreachable!()
    }
}

impl crate::platform::graphics::SlicedImage<Texture> for Graphic {
    fn set_image<P>(&mut self, _texture: Texture, _padding: P)
    where
        P: crate::dims::Padding2d,
    {
        unreachable!()
    }
}

impl crate::platform::graphics::SelectableSlicedImage<Texture> for Graphic {
    fn set_image<P>(
        &mut self,
        _texture: Texture,
        _padding: P,
        _states: &'static [crate::selectable::SelectionState],
    ) where
        P: crate::dims::Padding2d,
    {
        unreachable!()
    }
}

impl crate::platform::graphics::Text for Graphic {
    fn set_text<'a, T>(
        &mut self,
        _text: T,
        _pos: &crate::text::TextPosition,
        _settings: &crate::text::TextSettings,
    ) where
        T: 'a + Iterator<Item = crate::text::RichTextCommand<'a>>,
    {
        unreachable!()
    }
}

impl crate::platform::graphics::TextEdit for Graphic {
    fn set_text_plain(
        &mut self,
        _text: &str,
        _pos: &crate::text::TextPosition,
        _settings: &crate::text::TextSettings,
    ) {
        unreachable!()
    }

    fn char_at(&self, _x: f32, _y: f32) -> Option<usize> {
        unreachable!()
    }

    fn char_rect(&self, _index: usize) -> Option<crate::dims::SimpleRect> {
        unreachable!()
    }
}
