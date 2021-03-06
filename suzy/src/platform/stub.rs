/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::widget::{
    WidgetChildReceiver, WidgetContent, WidgetGraphicReceiver, WidgetInit,
};

use super::{Event, SimpleEventLoopState};

macro_rules! stub {
    () => {
        unimplemented!("StubPlatform used at runtime")
    };
}

/// The stub platform is used as a placeholder when no other platforms are
/// enabled.  All its methods will panic.
#[derive(Default)]
pub struct StubPlatform;

/// The stub platform is used as a placeholder when no other platforms are
/// enabled.  All its methods will panic.
#[derive(Default)]
pub struct StubWindow;

/// The stub platform is used as a placeholder when no other platforms are
/// enabled.  All its methods will panic.
#[derive(Default)]
pub struct StubRenderPlatform;

/// The stub platform is used as a placeholder when no other platforms are
/// enabled.  All its methods will panic.
#[derive(Default)]
pub struct StubDrawParams;

/// The stub platform is used as a placeholder when no other platforms are
/// enabled.  All its methods will panic.
#[derive(Default)]
pub struct StubButtonContent;

/// The stub platform is used as a placeholder when no other platforms are
/// enabled.  All its methods will panic.
#[cfg(feature = "platform_opengl")]
#[derive(Default)]
pub struct StubOpenglPlatform;

impl crate::platform::RenderPlatform for StubRenderPlatform {
    type Context = ();
    type DrawParams = StubDrawParams;

    type DefaultButtonContent = StubButtonContent;
}

impl crate::graphics::DrawParams<()> for StubDrawParams {
    fn apply_all(&mut self, _ctx: &mut ()) {
        stub!()
    }
    fn apply_change(_c: &Self, _n: &mut Self, _ctx: &mut ()) {
        stub!()
    }
}

impl crate::platform::Platform for StubPlatform {
    type State = SimpleEventLoopState;
    type Window = StubWindow;
    type Renderer = StubRenderPlatform;

    fn new() -> Self {
        stub!()
    }

    fn create_window(
        &mut self,
        _settings: crate::window::WindowBuilder,
    ) -> Result<Self::Window, String> {
        stub!()
    }

    fn run<F>(self, _event_handler: F) -> !
    where
        F: 'static + FnMut(&mut Self::State, Event),
    {
        stub!()
    }
}

#[cfg(feature = "platform_opengl")]
impl crate::platform::Platform for StubOpenglPlatform {
    type State = SimpleEventLoopState;
    type Window = StubWindow;
    type Renderer = super::opengl::OpenGlRenderPlatform;

    fn new() -> Self {
        stub!()
    }

    fn create_window(
        &mut self,
        _settings: crate::window::WindowBuilder,
    ) -> Result<Self::Window, String> {
        stub!()
    }

    fn run<F>(self, _event_handler: F) -> !
    where
        F: 'static + FnMut(&mut Self::State, Event),
    {
        stub!()
    }
}

impl crate::window::WindowSettings for StubWindow {
    fn size(&self) -> (f32, f32) {
        stub!()
    }
    fn set_size(&mut self, _size: (f32, f32)) {
        stub!()
    }
    fn title(&self) -> &str {
        stub!()
    }
    fn set_title(&mut self, _title: String) {
        stub!()
    }
    fn fullscreen(&self) -> bool {
        stub!()
    }
    fn set_fullscreen(&mut self, _fullscreen: bool) {
        stub!()
    }
    fn background_color(&self) -> crate::graphics::Color {
        stub!()
    }
    fn set_background_color(&mut self, _color: crate::graphics::Color) {
        stub!()
    }
}

impl crate::window::Window<StubRenderPlatform> for StubWindow {
    fn pixels_per_dp(&self) -> f32 {
        stub!()
    }

    fn normalize_pointer_event(
        &self,
        _event: &mut crate::pointer::PointerEventData,
    ) {
        stub!()
    }

    fn recalculate_viewport(&mut self) {
        stub!()
    }

    fn flip(&mut self) {
        stub!()
    }

    fn prepare_draw(
        &mut self,
        _first_pass: bool,
    ) -> crate::graphics::DrawContext<StubRenderPlatform> {
        stub!()
    }

    fn take_screenshot(&self) -> Box<[u8]> {
        stub!()
    }
}

#[cfg(feature = "platform_opengl")]
impl crate::window::Window<super::opengl::OpenGlRenderPlatform>
    for StubWindow
{
    fn pixels_per_dp(&self) -> f32 {
        stub!()
    }

    fn normalize_pointer_event(
        &self,
        _event: &mut crate::pointer::PointerEventData,
    ) {
        stub!()
    }

    fn recalculate_viewport(&mut self) {
        stub!()
    }

    fn flip(&mut self) {
        stub!()
    }

    fn prepare_draw(
        &mut self,
        _first_pass: bool,
    ) -> crate::graphics::DrawContext<super::opengl::OpenGlRenderPlatform>
    {
        stub!()
    }

    fn take_screenshot(&self) -> Box<[u8]> {
        stub!()
    }
}

impl crate::widgets::TextContent for StubButtonContent {
    fn set_text(&mut self, _text: &str) {
        stub!()
    }
}

impl<P: super::RenderPlatform> WidgetContent<P> for StubButtonContent {
    fn init(_init: impl WidgetInit<Self, P>) {
        stub!()
    }

    fn children(&mut self, _receiver: impl WidgetChildReceiver<P>) {
        stub!()
    }

    fn graphics(&mut self, _receiver: impl WidgetGraphicReceiver<P>) {
        stub!()
    }
}

impl crate::selectable::Selectable for StubButtonContent {
    fn selection_changed(
        &mut self,
        _state: crate::selectable::SelectionState,
    ) {
        stub!()
    }
}
