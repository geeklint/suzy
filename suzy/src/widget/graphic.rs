/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use crate::graphics::Graphic;
use crate::platform::{DefaultRenderPlatform, RenderPlatform};

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
    /// The type of graphic to render before the widget's children.
    type Before: Graphic<P> + 'b;

    /// The type of graphic to render after the widget's children.
    type After: Graphic<P> + 'a;

    /// Get the graphic to render before the widget's children.
    fn before_children(&'b mut self) -> Self::Before;

    /// Get the graphic to render after the widget's children.
    fn after_children(&'a mut self) -> Self::After;

    /// If this graphic is strongly ordered, such that `after_children` should
    /// be called in reverse order as `before_children`, relative to other
    /// graphics.
    fn ordered() -> bool {
        true
    }
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

    fn ordered() -> bool {
        false
    }
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
