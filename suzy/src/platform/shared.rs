/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::selectable::Selectable;
use crate::window::{
    Window,
    WindowEvent,
    WindowBuilder,
};
use crate::widget::WidgetContent;
use crate::widgets::TextContent;

pub trait RenderPlatform: 'static {
    type Context: 'static;
    type DrawParams: crate::graphics::DrawParams<Self::Context>;

    type DefaultButtonContent:
        Default + Selectable + WidgetContent<Self> + TextContent;
}

pub enum Event<'a> {
    WindowEvent(WindowEvent),
    StartFrame(std::time::Instant),
    Update,
    Draw,
    FinishDraw,
    TakeScreenshot(&'a mut Box<[u8]>),
}

pub trait EventLoopState {
    fn request_shutdown(&mut self);
}

pub trait Platform: 'static {
    type State: EventLoopState;
    type Renderer: RenderPlatform;
    type Window: Window<Self::Renderer>;

    fn new() -> Self;

    fn create_window(&mut self, settings: WindowBuilder)
        -> Result<Self::Window, String>;

    fn run<F>(self, event_handler: F) -> !
    where
        F: 'static + FnMut(&mut Self::State, Event);
}

#[derive(Clone, Copy, Debug)]
pub struct SimpleEventLoopState {
    pub running: bool,
}

impl Default for SimpleEventLoopState {
    fn default() -> Self { Self { running: true } }
}

impl EventLoopState for SimpleEventLoopState {
    fn request_shutdown(&mut self) {
        self.running = false;
    }
}
