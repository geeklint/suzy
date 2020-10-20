/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

/// Graphics control the visual appearance of Widgets.
///
/// Some widgets will simply contain other widgets (their "children"), but
/// in order to have visuals, widgets need a way to interface with the
/// renderer.  The Graphic trait in this module provides this interface.
///
/// Because Graphic implementations are tightly coupled with the renderer,
/// there are no actual implementations in this module; Suzy's "built-in"
/// graphics can be found in the
/// [module for the opengl platform](../platform/opengl/index.html)

mod color;
mod context;

pub use color::{Color, ParseColorError};
pub use context::{DrawPass, DrawParams, DrawContext};

use crate::platform::{
    DefaultRenderPlatform,
    RenderPlatform,
};

/// A trait which represents a drawable graphic.
///
/// See [module-level documentation](index.html) for more information.
pub trait Graphic<P = DefaultRenderPlatform>
where
    P: RenderPlatform + ?Sized,
{
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
