/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

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
mod widgets;
mod window;

pub use buffer::{
    DualVertexBuffer, DualVertexBufferIndexed, SingleVertexBuffer,
};
pub use context::{OpenGlBindings, OpenGlContext};
pub use drawparams::DrawParams;
pub use graphics::*;
pub use matrix::Mat4;
pub use text::{
    FontFamily, FontFamilyDynamic, FontFamilySource, FontFamilySourceDynamic,
    RawText, Text, TextLayoutSettings,
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
    type Context = OpenGlContext;
    type DrawParams = drawparams::DrawParams;

    type DefaultButtonContent = widgets::DefaultOpenGlButton;
    type Texture = Texture;
    type SlicedImage = SlicedImage;
    type SelectableSlicedImage = SelectableSlicedImage;
}
