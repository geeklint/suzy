/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

//! A flexible interface for the low-level aspects of the GUI system.
//!
//! One render platform is currently included, based on OpenGL ES 2.0
//!
//! Two windowing platforms are currently included; the primary is based on
//! SDL2.  A windowless OSMesa interface is also included, intended for
//! automated testing.

mod event;
pub use event::{Event, EventLoopState, SimpleEventLoopState};

use crate::dims::Rect;

pub mod graphics;

/// A platform handles window creation and manages an event loop.
pub trait Platform: 'static {
    /// The event loop state tracked by this platform.
    type State: EventLoopState;

    /// The RenderPlatform this platform supports.
    type Renderer: RenderPlatform;

    /// The type of window this platform creates.
    type Window: crate::window::Window<Self::Renderer>;

    /// Initialize the platform.
    fn new() -> Self;

    /// Create a window.
    fn create_window(
        &mut self,
        settings: crate::window::WindowBuilder,
    ) -> Result<Self::Window, String>;

    /// Run, the event loop, calling the provided closure with each new event.
    fn run(self, app: &mut crate::app::App<Self>) -> !;
}

/// A RenderPlatform provides tools to create Graphics.
pub trait RenderPlatform: 'static {
    type DrawPassInfo;
    /// The shared context passed along to draw calls.
    type DrawContextBuilder: for<'a> crate::graphics::BuildDrawContext<
        'a,
        Self::DrawPassInfo,
    >;

    /// The platform's graphic primitive for 9-sliced images.
    type SlicedImage: graphics::SlicedImage
        + Rect
        + Default
        + crate::graphics::Graphic<Self>;

    type TextStyle: graphics::TextStyle;

    /// The platform's graphic primitive for text
    type Text: graphics::Text<Self::TextStyle>
        + Default
        + crate::graphics::Graphic<Self>;
}
