/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

#![allow(missing_docs)]

use crate::graphics::{Color, DrawContext, Graphic};
use crate::widget::WidgetGraphic;

use super::super::OpenGlRenderPlatform;

pub trait Effect {
    fn push(&mut self, ctx: &mut DrawContext<'_, OpenGlRenderPlatform>);
    fn pop(&mut self, ctx: &mut DrawContext<'_, OpenGlRenderPlatform>);
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
    fn draw(&mut self, ctx: &mut DrawContext<'_, OpenGlRenderPlatform>) {
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
    fn draw(&mut self, ctx: &mut DrawContext<'_, OpenGlRenderPlatform>) {
        self.inner.effect.pop(ctx);
        if let Some(params) = self.inner.backup_params.take() {
            ctx.manually_pop(params);
        }
    }
}

impl<T> WidgetGraphic<OpenGlRenderPlatform> for BaseEffect<T>
where
    T: Effect + ?Sized,
{
    type BeforeGetter = fn(&mut ()) -> BaseEffectPush<'_, T>;
    type AfterGetter = fn(&mut ()) -> BaseEffectPop<'_, T>;

    fn before_children(&mut self) -> BaseEffectPush<'_, T> {
        BaseEffectPush { inner: self }
    }

    fn after_children(&mut self) -> BaseEffectPop<'_, T> {
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
    fn push(&mut self, ctx: &mut DrawContext<'_, OpenGlRenderPlatform>) {
        ctx.params().tint(self.color);
    }

    fn pop(&mut self, _ctx: &mut DrawContext<'_, OpenGlRenderPlatform>) {}
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

impl WidgetGraphic<OpenGlRenderPlatform> for Tint {
    type BeforeGetter = fn(&mut ()) -> BaseEffectPush<'_, TintEffect>;
    type AfterGetter = fn(&mut ()) -> BaseEffectPop<'_, TintEffect>;

    fn before_children(&mut self) -> BaseEffectPush<'_, TintEffect> {
        self.inner.before_children()
    }

    fn after_children(&mut self) -> BaseEffectPop<'_, TintEffect> {
        self.inner.after_children()
    }
}
