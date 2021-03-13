/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::selectable::Selectable;
use crate::widget::WidgetContent;
use crate::widgets::TextContent;
use crate::window::{Window, WindowBuilder, WindowEvent};

/// A RenderPlatform provides tools to create Graphics.
pub trait RenderPlatform: 'static {
    /// The shared context passed along to draw calls.
    type Context: 'static;

    /// The parameters passed to draw calls.
    type DrawParams: crate::graphics::DrawParams<Self::Context>;

    /// A default type of WidgetContent this RenderPlatform provides for
    /// buttons.
    type DefaultButtonContent: Default
        + Selectable
        + WidgetContent<Self>
        + TextContent;
}

/// Possible events a platform could generate.
pub enum Event<'a> {
    /// Window event, such as resize
    WindowEvent(WindowEvent),

    /// Emitted when a new frame is being started.
    StartFrame(std::time::Instant),

    /// Update all pending watch closures.
    Update,

    /// Render all widgets.
    Draw,

    /// After rendering, finalize displaying the results to the user.
    FinishDraw,

    /// Take a screenshot.
    ///
    /// Sent by testing platform in some circumstances.
    TakeScreenshot(&'a mut Box<[u8]>),
}

/// A trait which the event handler can use to shutdown the event loop.
pub trait EventLoopState {
    /// Signal that the event loop should stop.
    fn request_shutdown(&mut self);
}

/// A platform handles window creation and manages an event loop.
pub trait Platform: 'static {
    /// The event loop state tracked by this platform.
    type State: EventLoopState;

    /// The RenderPlatform this platform supports.
    type Renderer: RenderPlatform;

    /// The type of window this platform creates.
    type Window: Window<Self::Renderer>;

    /// Initialize the platform.
    fn new() -> Self;

    /// Create a window.
    fn create_window(
        &mut self,
        settings: WindowBuilder,
    ) -> Result<Self::Window, String>;

    /// Run, the event loop, calling the provided closure with each new event.
    fn run<F>(self, event_handler: F) -> !
    where
        F: 'static + FnMut(&mut Self::State, Event);
}

/// A type which implements EventLoopState with a single boolean flag.
#[derive(Clone, Copy, Debug)]
pub struct SimpleEventLoopState {
    /// A flag indicating if the event loop should keep running.
    pub running: bool,
}

impl Default for SimpleEventLoopState {
    fn default() -> Self {
        Self { running: true }
    }
}

impl EventLoopState for SimpleEventLoopState {
    fn request_shutdown(&mut self) {
        self.running = false;
    }
}
