use crate::math::Rect;
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
}

#[repr(transparent)]
pub struct MaskerPush<T> {
    inner: MaskerInner<T>,
}

impl<T> Graphic<OpenGlRenderPlatform> for MaskerPush<T>
where
    T: Graphic<OpenGlRenderPlatform>,
{
    fn draw(&mut self, ctx: &mut DrawContext<OpenGlRenderPlatform>) {
        DrawContext::manually_push(ctx);
        ctx.push_mask();
        self.inner.item.draw(ctx);
        DrawContext::manually_push(ctx);
        ctx.commit_mask();
    }
}

#[repr(transparent)]
pub struct MaskerPop<T> {
    inner: MaskerInner<T>,
}

impl<T> Graphic<OpenGlRenderPlatform> for MaskerPop<T>
where
    T: Graphic<OpenGlRenderPlatform>,
{
    fn draw(&mut self, ctx: &mut DrawContext<OpenGlRenderPlatform>) {
        DrawContext::manually_push(ctx);
        ctx.pop_mask();
        let draw = match (DrawContext::pass(ctx), self.inner.popped) {
            (DrawPass::DrawRemaining, true) => false,
            _ => true,
        };
        if draw {
            self.inner.popped = DrawContext::force_redraw(ctx, |ctx| {
                self.inner.item.draw(ctx);
                DrawContext::pass(ctx) != DrawPass::UpdateContext
            });
        }
        DrawContext::manually_pop(ctx);
        DrawContext::manually_pop(ctx);
        DrawContext::manually_pop(ctx);
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


impl<T> WidgetGraphic<OpenGlRenderPlatform> for Masker<T>
where
    T: Graphic<OpenGlRenderPlatform>,
{
    type Before = MaskerPush<T>;
    type After = MaskerPop<T>;

    fn before_children(&mut self) -> &mut Self::Before {
        let ptr = (&mut self.inner) as *mut MaskerInner<T> as *mut MaskerPush<T>;
        // TODO: remove unsafe
        unsafe {
            &mut *ptr
        }
    }

    fn after_children(&mut self) -> &mut Self::After {
        let ptr = (&mut self.inner) as *mut MaskerInner<T> as *mut MaskerPop<T>;
        // TODO: remove unsafe
        unsafe {
            &mut *ptr
        }
    }
    
}

impl<T: Rect> Rect for Masker<T> {
    fn x(&self) -> crate::math::Dim {
        self.inner.item.x()
    }
    fn y(&self) -> crate::math::Dim {
        self.inner.item.y()
    }
    fn x_mut<F, R>(&mut self, f: F) -> R
    where F: FnOnce(&mut crate::math::Dim) -> R
    {
        self.inner.item.x_mut(f)
    }
    fn y_mut<F, R>(&mut self, f: F) -> R
    where F: FnOnce(&mut crate::math::Dim) -> R
    {
        self.inner.item.y_mut(f)
    }
    
}
