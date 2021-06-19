/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

#![allow(missing_docs)]

use crate::graphics::{Color, DrawContext, Graphic};
use crate::widget::WidgetGraphic;

use super::super::OpenGlRenderPlatform;

pub trait Effect {
    fn push(&mut self, ctx: &mut DrawContext<OpenGlRenderPlatform>);
    fn pop(&mut self, ctx: &mut DrawContext<OpenGlRenderPlatform>);
}

#[derive(Default)]
pub struct BaseEffect<T: ?Sized> {
    backup_params: Option<super::super::DrawParams>,
    pub effect: T,
}

pub struct BaseEffectPush<'a, T: Effect + ?Sized> {
    inner: &'a mut BaseEffect<T>,
}

impl<'a, T> Graphic<OpenGlRenderPlatform> for BaseEffectPush<'a, T>
where
    T: Effect + ?Sized,
{
    fn draw(&mut self, ctx: &mut DrawContext<OpenGlRenderPlatform>) {
        self.inner.backup_params = Some(ctx.manually_push());
        self.inner.effect.push(ctx);
    }
}

pub struct BaseEffectPop<'a, T: Effect + ?Sized> {
    inner: &'a mut BaseEffect<T>,
}

impl<'a, T> Graphic<OpenGlRenderPlatform> for BaseEffectPop<'a, T>
where
    T: Effect + ?Sized,
{
    fn draw(&mut self, ctx: &mut DrawContext<OpenGlRenderPlatform>) {
        self.inner.effect.pop(ctx);
        if let Some(params) = self.inner.backup_params.take() {
            ctx.manually_pop(params);
        }
    }
}

impl<'a, 'b, T> WidgetGraphic<'a, 'b, OpenGlRenderPlatform> for BaseEffect<T>
where
    T: 'a + 'b + Effect + ?Sized,
{
    type Before = BaseEffectPush<'b, T>;
    type After = BaseEffectPop<'a, T>;

    fn before_children(&'b mut self) -> Self::Before {
        BaseEffectPush { inner: self }
    }

    fn after_children(&'a mut self) -> Self::After {
        BaseEffectPop { inner: self }
    }
}

pub struct TintEffect {
    color: Color,
}

impl Default for TintEffect {
    fn default() -> Self {
        TintEffect {
            color: Color::WHITE,
        }
    }
}

impl Effect for TintEffect {
    fn push(&mut self, ctx: &mut DrawContext<OpenGlRenderPlatform>) {
        ctx.params().tint(self.color);
    }

    fn pop(&mut self, _ctx: &mut DrawContext<OpenGlRenderPlatform>) {}
}

#[derive(Default)]
pub struct Tint {
    inner: BaseEffect<TintEffect>,
}

impl Tint {
    pub fn set_tint_color(&mut self, color: Color) {
        self.inner.effect.color = color;
    }

    pub fn current_tint_color(&self) -> Color {
        self.inner.effect.color
    }
}

impl<'a, 'b> WidgetGraphic<'a, 'b, OpenGlRenderPlatform> for Tint {
    type Before = BaseEffectPush<'b, TintEffect>;
    type After = BaseEffectPop<'a, TintEffect>;

    fn before_children(&'b mut self) -> Self::Before {
        self.inner.before_children()
    }

    fn after_children(&'a mut self) -> Self::After {
        self.inner.after_children()
    }
}
