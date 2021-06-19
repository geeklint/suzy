/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use crate::dims::Rect;
use crate::graphics::{DrawContext, DrawPass, Graphic};
use crate::widget::WidgetGraphic;

use super::super::OpenGlRenderPlatform;

#[derive(Default)]
pub struct MaskEffect<T: ?Sized> {
    popped: bool,
    item: T,
}

impl<T> super::Effect for MaskEffect<T>
where
    T: Graphic<OpenGlRenderPlatform> + ?Sized,
{
    fn push(&mut self, ctx: &mut DrawContext<OpenGlRenderPlatform>) {
        ctx.params().push_mask();
        self.item.draw(ctx);
        ctx.params().commit_mask();
    }

    fn pop(&mut self, ctx: &mut DrawContext<OpenGlRenderPlatform>) {
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

impl<'a, 'b, T> WidgetGraphic<'a, 'b, OpenGlRenderPlatform> for Masker<T>
where
    T: 'a + 'b + Graphic + ?Sized,
{
    type Before = super::effects::BaseEffectPush<'b, MaskEffect<T>>;
    type After = super::effects::BaseEffectPop<'a, MaskEffect<T>>;

    fn before_children(&'b mut self) -> Self::Before {
        self.inner.before_children()
    }

    fn after_children(&'a mut self) -> Self::After {
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
