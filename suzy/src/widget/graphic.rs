/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::platform::{DefaultRenderPlatform, RenderPlatform};
use crate::graphics::Graphic;

/// A trait which represents a graphic a widget might contain.
///
/// Automatically implemented for anything which implements
/// [`Graphic`](../graphics/trait.Graphic.html).
///
/// Widget graphics are rendered in two passes: one before the widget's
/// children, and one after.  The typical behavior is to ignore the second
/// pass, however some functionality may require it, for instance to revert
/// a state-change applied in the first pass.
pub trait WidgetGraphic<'a, 'b, P = DefaultRenderPlatform>
where
    P: RenderPlatform + ?Sized,
{
    type Before: Graphic<P> + 'b;
    type After: Graphic<P> + 'a;

    fn before_children(&'b mut self) -> Self::Before;
    fn after_children(&'a mut self) -> Self::After;

    fn ordered() -> bool { true }
}

impl<'a, 'b, P, T> WidgetGraphic<'a, 'b, P> for T
where
    Self: Sized,
    T: Graphic<P> + 'b,
    P: RenderPlatform,
{
    type Before = WidgetGraphicProxy<'b, T>;
    type After = WidgetGraphicProxy<'a, [(); 0]>;

    fn before_children(&'b mut self) -> Self::Before {
        WidgetGraphicProxy { graphic: self }
    }

    fn after_children(&'a mut self) -> Self::After {
        WidgetGraphicProxy { graphic: &mut [] }
    }

    fn ordered() -> bool { false }
}

pub struct WidgetGraphicProxy<'a, T> {
    graphic: &'a mut T,
}

impl<'a, P, T> Graphic<P> for WidgetGraphicProxy<'a, T>
where
    P: RenderPlatform,
    T: Graphic<P>,
{
    fn draw(&mut self, ctx: &mut crate::graphics::DrawContext<P>) {
        self.graphic.draw(ctx);
    }
}
