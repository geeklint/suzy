/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dims::Rect;
use crate::widget::WidgetGraphic;
use crate::graphics::{
    Graphic,
    DrawContext,
    DrawPass,
};

use super::super::OpenGlRenderPlatform;

#[derive(Default)]
pub struct MaskerInner<T> {
    item: T,
    popped: bool,
    backup_params: Option<super::super::DrawParams>,
}

#[repr(transparent)]
pub struct MaskerPush<'a, T> {
    inner: &'a mut MaskerInner<T>,
}

impl<'a, T> Graphic<OpenGlRenderPlatform> for MaskerPush<'a, T>
where
    T: Graphic<OpenGlRenderPlatform>,
{
    fn draw(&mut self, ctx: &mut DrawContext<OpenGlRenderPlatform>) {
        self.inner.backup_params = Some(ctx.manually_push());
        ctx.params().push_mask();
        self.inner.item.draw(ctx);
        ctx.params().commit_mask();
    }
}

#[repr(transparent)]
pub struct MaskerPop<'a, T> {
    inner: &'a mut MaskerInner<T>,
}

impl<'a, T> Graphic<OpenGlRenderPlatform> for MaskerPop<'a, T>
where
    T: Graphic<OpenGlRenderPlatform>,
{
    fn draw(&mut self, ctx: &mut DrawContext<OpenGlRenderPlatform>) {
        ctx.params().pop_mask();
        let draw = match (ctx.pass(), self.inner.popped) {
            (DrawPass::DrawRemaining, true) => false,
            _ => true,
        };
        if draw {
            self.inner.popped = ctx.force_redraw(|ctx| {
                self.inner.item.draw(ctx);
                ctx.pass() != DrawPass::UpdateContext
            });
        }
        if let Some(params) = self.inner.backup_params.take() {
            ctx.manually_pop(params);
        }
    }
}

#[derive(Default)]
pub struct Masker<T> {
    inner: MaskerInner<T>,
}

impl<T> Masker<T> {
    pub fn graphic(&self) -> &T { &self.inner.item }
    pub fn graphic_mut(&mut self) -> &mut T { &mut self.inner.item }
}


impl<'a, 'b, T> WidgetGraphic<'a, 'b, OpenGlRenderPlatform> for Masker<T>
where
    T: Graphic<OpenGlRenderPlatform> + 'a + 'b,
{
    type Before = MaskerPush<'b, T>;
    type After = MaskerPop<'a, T>;

    fn before_children(&'b mut self) -> Self::Before {
        MaskerPush { inner: &mut self.inner }
    }

    fn after_children(&'a mut self) -> Self::After {
        MaskerPop { inner: &mut self.inner }
    }
    
}

impl<T: Rect> Rect for Masker<T> {
    fn x(&self) -> crate::dims::Dim {
        self.inner.item.x()
    }
    fn y(&self) -> crate::dims::Dim {
        self.inner.item.y()
    }
    fn x_mut<F, R>(&mut self, f: F) -> R
    where F: FnOnce(&mut crate::dims::Dim) -> R
    {
        self.inner.item.x_mut(f)
    }
    fn y_mut<F, R>(&mut self, f: F) -> R
    where F: FnOnce(&mut crate::dims::Dim) -> R
    {
        self.inner.item.y_mut(f)
    }
    
}
