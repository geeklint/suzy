/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use crate::graphics::Graphic;
use crate::platform::{DefaultRenderPlatform, RenderPlatform};

pub trait GetGraphicMethod<'a, P>
where
    P: RenderPlatform + ?Sized,
{
    type Graphic: Graphic<P>;
}

impl<'a, G, F, P> GetGraphicMethod<'a, P> for F
where
    F: FnOnce(&'a mut ()) -> G,
    G: Graphic<P>,
    P: RenderPlatform + ?Sized,
{
    type Graphic = G;
}

/// A trait which represents a graphic a widget might contain.
///
/// Automatically implemented for anything which implements
/// [`Graphic`](../graphics/trait.Graphic.html).
///
/// Widget graphics are rendered in two passes: one before the widget's
/// children, and one after.  The typical behavior is to ignore the second
/// pass, however some functionality may require it, for instance to revert
/// a state-change applied in the first pass.
pub trait WidgetGraphic<P = DefaultRenderPlatform>
where
    P: RenderPlatform + ?Sized,
{
    /// The type of graphic to render before the widget's children.
    type BeforeGetter: for<'a> GetGraphicMethod<'a, P>;

    /// The type of graphic to render after the widget's children.
    type AfterGetter: for<'a> GetGraphicMethod<'a, P>;

    /// Get the graphic to render before the widget's children.
    fn before_children(
        &mut self,
    ) -> <Self::BeforeGetter as GetGraphicMethod<'_, P>>::Graphic;

    /// Get the graphic to render after the widget's children.
    fn after_children(
        &mut self,
    ) -> <Self::AfterGetter as GetGraphicMethod<'_, P>>::Graphic;

    /// If this graphic is strongly ordered, such that `after_children` should
    /// be called in reverse order as `before_children`, relative to other
    /// graphics.
    fn ordered() -> bool {
        true
    }
}

impl<P, T> WidgetGraphic<P> for T
where
    Self: Sized,
    T: Graphic<P>,
    P: RenderPlatform,
{
    type BeforeGetter = fn(&mut ()) -> &mut T;
    type AfterGetter = fn(&mut ()) -> &mut [(); 0];

    fn before_children(&mut self) -> &mut T {
        self
    }

    fn after_children(&mut self) -> &mut [(); 0] {
        &mut []
    }

    fn ordered() -> bool {
        false
    }
}
