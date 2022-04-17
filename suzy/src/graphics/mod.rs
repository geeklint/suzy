/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

//! Graphics control the visual appearance of Widgets.
//!
//! Some widgets will simply contain other widgets (their "children"), but
//! in order to have visuals, widgets need a way to interface with the
//! renderer.  The Graphic trait in this module provides this interface.
//!
//! Because Graphic implementations are tightly coupled with the renderer,
//! there are no actual implementations in this module; Suzy's "built-in"
//! graphics can be found in the
//! [module for the opengl platform](../platform/opengl/index.html)

mod color;
mod context;

pub use color::{Color, ParseColorError};
pub use context::{DrawContext, DrawParams, DrawPass};

use crate::platform::{DefaultRenderPlatform, RenderPlatform};

/// A trait which represents a drawable graphic.
///
/// See [module-level documentation](index.html) for more information.
pub trait Graphic<P = DefaultRenderPlatform>
where
    P: RenderPlatform + ?Sized,
{
    /// Draw this graphic.
    fn draw(&mut self, ctx: &mut DrawContext<P>);
}

impl<P: RenderPlatform + ?Sized> Graphic<P> for () {
    fn draw(&mut self, _ctx: &mut DrawContext<P>) {}
}

impl<P: RenderPlatform + ?Sized> Graphic<P> for [(); 0] {
    fn draw(&mut self, _ctx: &mut DrawContext<P>) {}
}

impl<P: RenderPlatform + ?Sized, T: Graphic<P>> Graphic<P> for [T] {
    fn draw(&mut self, ctx: &mut DrawContext<P>) {
        for graphic in self {
            graphic.draw(ctx);
        }
    }
}

impl<P: RenderPlatform + ?Sized, T: Graphic<P>> Graphic<P> for &mut T {
    fn draw(&mut self, ctx: &mut DrawContext<P>) {
        T::draw(self, ctx)
    }
}
