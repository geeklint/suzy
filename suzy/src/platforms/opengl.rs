/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

//! This module provides Suzy's default, built-in render platform, based on
//! OpenGL ES 2.0.
//!
//! In this module are a number of important Graphics implementations for
//! the platform.

mod texture;

mod context;
mod graphics;
mod matrix;
pub mod renderer;
mod shader;
mod stdshaders;
mod text;
mod window;

pub use context::bindings as opengl_bindings;
pub use context::{OpenGlBindings, OpenGlContext, DEBUG};
pub use graphics::*;
pub use matrix::Mat4;
pub use text::{Font, Text};
pub use texture::{
    PopulateTexture, PopulateTextureUtil, Texture, TextureId, TextureSize,
};
pub use window::Window;

/// The OpenGL render platform is the default "built-in" render platform
/// included with Suzy.
pub struct OpenGlRenderPlatform;

impl crate::platform::RenderPlatform for OpenGlRenderPlatform {
    type DrawPassInfo = ();
    type DrawContextBuilder = fn(&mut ()) -> DrawContext<'_>;

    type SlicedImage = SlicedImage;
    type TextStyle = text::TextStyle;
    type Text = Text;
}
