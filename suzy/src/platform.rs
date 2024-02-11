/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

//! A flexible interface for the low-level aspects of the GUI system.
//!
//! One render platform is currently included, based on OpenGL ES 2.0
//!
//! Two windowing platforms are currently included; the primary is based on
//! SDL2.  A windowless OSMesa interface is also included, intended for
//! automated testing.

use crate::dims::Rect;

pub mod graphics;

/// A platform handles window creation and manages an event loop.
pub trait Platform: 'static {
    /// The RenderPlatform this platform supports.
    type Renderer: RenderPlatform;
}

/// A RenderPlatform provides tools to create Graphics.
pub trait RenderPlatform: 'static {
    type DrawPassInfo;
    /// The shared context passed along to draw calls.
    type DrawContextBuilder: for<'a> crate::graphics::BuildDrawContext<'a>;

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
