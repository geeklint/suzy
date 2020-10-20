/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This module provides Suzy's default, built-in render platform, based on
//! OpenGL ES 2.0.
//!
//! In this module are a number of important Graphics implementations for
//! the platform.

#[macro_use] mod primitive;
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
mod widgets;

pub use buffer::{
    SingleVertexBuffer,
    DualVertexBuffer,
    DualVertexBufferIndexed,
};
pub use texture::{
    PopulateTexture,
    PopulateTextureDynClone,
    PopulateTextureUtil,
    Texture,
    TextureCacheKey,
    TextureSize,
};
pub use context::{
    OpenGlContext,
    OpenGlBindings,
};
pub use drawparams::DrawParams;
pub use graphics::*;
pub use matrix::Mat4;
pub use text::{
    FontFamily,
    FontFamilyDynamic,
    FontFamilySource,
    FontFamilySourceDynamic,
    FontStyle,
    RichTextCommand,
    RichTextParser,
    Text,
    TextAlignment,
    TextLayoutSettings,
};
pub use window::Window;

pub struct OpenGlRenderPlatform;

impl super::RenderPlatform for OpenGlRenderPlatform {
    type Context = OpenGlContext;
    type DrawParams = drawparams::DrawParams;

    type DefaultButtonContent = widgets::DefaultOpenGlButton;
}
