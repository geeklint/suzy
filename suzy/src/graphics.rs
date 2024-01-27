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
//! [module for the opengl platform](crate::platforms::opengl)

mod color;
mod context;

pub use color::{Color, ParseColorError};
pub use context::{BuildDrawContext, DrawContext, PlatformDrawContext};

use crate::platform::RenderPlatform;

/// A trait which represents a drawable graphic.
///
/// See [module-level documentation](self) for more information.
pub trait Graphic<P>
where
    P: ?Sized + RenderPlatform,
{
    /// Draw this graphic.
    fn draw(&mut self, ctx: &mut DrawContext<'_, P>);
}

impl<P: ?Sized + RenderPlatform> Graphic<P> for () {
    fn draw(&mut self, _ctx: &mut DrawContext<'_, P>) {}
}

impl<P: ?Sized + RenderPlatform> Graphic<P> for [(); 0] {
    fn draw(&mut self, _ctx: &mut DrawContext<'_, P>) {}
}

impl<P: ?Sized + RenderPlatform, T: Graphic<P>> Graphic<P> for [T] {
    fn draw(&mut self, ctx: &mut DrawContext<'_, P>) {
        for graphic in self {
            graphic.draw(ctx);
        }
    }
}

impl<P: ?Sized + RenderPlatform, T: Graphic<P>> Graphic<P> for &mut T {
    fn draw(&mut self, ctx: &mut DrawContext<'_, P>) {
        T::draw(self, ctx)
    }
}

pub struct Conditional<T> {
    pub enable: bool,
    pub graphic: T,
}

impl<T: Default> Default for Conditional<T> {
    fn default() -> Self {
        Self {
            enable: true,
            graphic: T::default(),
        }
    }
}

impl<P: ?Sized + RenderPlatform, T: Graphic<P>> Graphic<P> for Conditional<T> {
    fn draw(&mut self, ctx: &mut DrawContext<'_, P>) {
        if self.enable {
            self.graphic.draw(ctx);
        }
    }
}

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub enum CornerStyle {
    #[default]
    NotRounded,
    Rounded,
}
