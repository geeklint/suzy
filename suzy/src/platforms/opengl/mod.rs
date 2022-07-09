/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

//! This module provides Suzy's default, built-in render platform, based on
//! OpenGL ES 2.0.
//!
//! In this module are a number of important Graphics implementations for
//! the platform.

#[macro_use]
mod primitive;
mod buffer;
mod texture;

mod context;
mod drawparams;
mod graphics;
mod mask;
mod matrix;
mod shader;
mod stdshaders;
mod text;
mod window;

pub use buffer::{
    DualVertexBuffer, DualVertexBufferIndexed, SingleVertexBuffer,
};
pub use context::{OpenGlBindings, OpenGlContext, DEBUG};
pub use drawparams::DrawParams;
pub use graphics::*;
pub use matrix::Mat4;
pub use text::{
    FontFamily, FontFamilyDynamic, FontFamilySource, FontFamilySourceDynamic,
    RawText, Text, TextEdit, TextLayoutSettings,
};
pub use texture::{
    PopulateTexture, PopulateTextureDynClone, PopulateTextureUtil, Texture,
    TextureCacheKey, TextureSize,
};
pub use window::Window;

/// The OpenGL render platform is the default "built-in" render platform
/// included with Suzy.
pub struct OpenGlRenderPlatform;

impl crate::platform::RenderPlatform for OpenGlRenderPlatform {
    type DrawPassInfo = ();
    type DrawContextBuilder = fn(&mut ()) -> DrawContext<'_>;

    type Texture = Texture;
    type SlicedImage = SlicedImage;
    type SelectableSlicedImage = SelectableSlicedImage;
    type Text = Text;
    type TextEdit = TextEdit;
}
