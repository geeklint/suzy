/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use crate::{
    dims::Rect,
    graphics::{DrawContext, Graphic},
    widget::WidgetGraphic,
};

use super::super::{DrawPass, OpenGlRenderPlatform};

#[derive(Default)]
pub struct MaskEffect<T: ?Sized> {
    popped: bool,
    item: T,
}

impl<T> super::Effect for MaskEffect<T>
where
    T: Graphic<OpenGlRenderPlatform> + ?Sized,
{
    fn push(&mut self, ctx: &mut DrawContext<'_, OpenGlRenderPlatform>) {
        ctx.params().push_mask();
        self.item.draw(ctx);
        ctx.params().commit_mask();
    }

    fn pop(&mut self, ctx: &mut DrawContext<'_, OpenGlRenderPlatform>) {
        ctx.params().pop_mask();
        if ctx.pass() != DrawPass::DrawRemaining || !self.popped {
            self.popped = ctx.force_redraw(|ctx| {
                self.item.draw(ctx);
                ctx.pass() != DrawPass::UpdateContext
            });
        }
    }
}

/// A Masker takes a graphic and uses it as a transparency mask, applied to
/// the subsequent graphics of the children of the widget with the Masker.
#[derive(Default)]
pub struct Masker<T: ?Sized> {
    inner: super::BaseEffect<MaskEffect<T>>,
}

impl<T: ?Sized> Masker<T> {
    /// Get a reference to the graphic used as a mask.
    pub fn graphic(&self) -> &T {
        &self.inner.effect.item
    }

    /// Get a mutable reference to the graphic used as a mask.
    pub fn graphic_mut(&mut self) -> &mut T {
        &mut self.inner.effect.item
    }
}

impl<T> WidgetGraphic<OpenGlRenderPlatform> for Masker<T>
where
    T: Graphic + ?Sized,
{
    type BeforeGetter =
        fn(&mut ()) -> super::effects::BaseEffectPush<'_, MaskEffect<T>>;
    type AfterGetter =
        fn(&mut ()) -> super::effects::BaseEffectPop<'_, MaskEffect<T>>;

    fn before_children(
        &mut self,
    ) -> super::effects::BaseEffectPush<'_, MaskEffect<T>> {
        self.inner.before_children()
    }

    fn after_children(
        &mut self,
    ) -> super::effects::BaseEffectPop<'_, MaskEffect<T>> {
        self.inner.after_children()
    }
}

impl<T: Rect + ?Sized> Rect for Masker<T> {
    fn x(&self) -> crate::dims::Dim {
        self.inner.effect.item.x()
    }
    fn y(&self) -> crate::dims::Dim {
        self.inner.effect.item.y()
    }
    fn x_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut crate::dims::Dim) -> R,
    {
        self.inner.effect.item.x_mut(f)
    }
    fn y_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut crate::dims::Dim) -> R,
    {
        self.inner.effect.item.y_mut(f)
    }
}
