/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

mod color;
mod context;

pub use color::{Color, ParseColorError};
pub use context::{DrawPass, DrawParams, DrawContext};

use crate::platform::{
    DefaultRenderPlatform,
    RenderPlatform,
};

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
