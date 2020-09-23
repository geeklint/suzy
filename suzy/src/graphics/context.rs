/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::platform::{
    DefaultRenderPlatform,
    RenderPlatform,
};

pub trait DrawParams<Ctx> {
    fn apply_all(&mut self, ctx: &mut Ctx);

    fn apply_change(current: &Self, new: &mut Self, ctx: &mut Ctx);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DrawPass {
    DrawAll,
    UpdateContext,
    DrawRemaining,
}

#[derive(Debug)]
pub struct DrawContext<'a, P: RenderPlatform = DefaultRenderPlatform> {
    context: &'a mut P::Context,
    current: P::DrawParams,
    last_applied: Option<P::DrawParams>,
    pass: DrawPass,
}

impl<'a, P: RenderPlatform> DrawContext<'a, P> {
    pub fn new(
        ctx: &'a mut P::Context,
        starting: P::DrawParams,
        first_pass: bool,
    ) -> Self {
        let pass = if first_pass {
            DrawPass::DrawAll
        } else {
            DrawPass::DrawRemaining
        };
        Self {
            context: ctx,
            current: starting,
            last_applied: None,
            pass,
        }
    }

    pub fn pass(&self) -> DrawPass { self.pass }

    pub fn render_ctx(&self) -> &P::Context { &self.context }

    pub fn render_ctx_mut(&mut self) -> &mut P::Context { &mut self.context }

    pub fn params(&mut self) -> &mut P::DrawParams { &mut self.current }

    pub fn graphic_not_ready(&mut self) {
        self.pass = DrawPass::UpdateContext;
    }

    pub fn force_redraw<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Self) -> R
    {
        let restore = if self.pass == DrawPass::DrawRemaining {
            self.pass = DrawPass::DrawAll;
            true
        } else {
            false
        };
        let ret = f(self);
        if restore && self.pass != DrawPass::UpdateContext {
            self.pass = DrawPass::DrawRemaining;
        }
        ret
    }

    pub(crate) fn draw<'b, I>(&mut self, roots: I) -> bool
    where
        I: 'b + Iterator<Item = &'b mut dyn crate::widget::AnonWidget<P>>,
    {
        if self.pass == DrawPass::UpdateContext {
            self.pass = DrawPass::DrawRemaining;
        }
        for widget in roots {
            widget.draw(self);
        }
        self.pass == DrawPass::UpdateContext
    }
}

impl<'a, P> DrawContext<'a, P>
where
    P: RenderPlatform,
    P::DrawParams: Clone
{
    pub fn prepare_draw(&mut self) {
        assert_ne!(
            self.pass, 
            DrawPass::UpdateContext,
            "prepare_draw called during an UpdateContext pass",
        );
        match self.last_applied.take() {
            Some(old) => {
                DrawParams::apply_change(
                    &old,
                    &mut self.current,
                    self.context,
                );
            },
            None => self.current.apply_all(self.context),
        }
        self.last_applied = Some(self.current.clone());
    }

    pub fn push<R, F: FnOnce(&mut Self) -> R>(&mut self, func: F) -> R {
        let backup = self.manually_push();
        let ret = func(self);
        self.manually_pop(backup);
        ret
    }

    #[must_use]
    pub fn manually_push(&mut self) -> P::DrawParams {
        self.current.clone()
    }

    pub fn manually_pop(&mut self, restore: P::DrawParams) {
        self.current = restore;
    }
}

impl<P: RenderPlatform> Drop for DrawContext<'_, P> {
    fn drop(&mut self) {
        if self.pass != DrawPass::UpdateContext {
            if let Some(old) = self.last_applied.take() {
                DrawParams::apply_change(
                    &old,
                    &mut self.current,
                    self.context,
                );
            }
        }
    }
}
