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

#[derive(Clone, Copy, Debug)]
enum LastApplied<T> {
    History(usize),
    Current,
    None,
    Removed(T),
}

impl<T> Default for LastApplied<T> {
    fn default() -> Self { Self::None }
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
    history: Vec<P::DrawParams>,
    last_applied: LastApplied<P::DrawParams>,
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
            history: Vec::new(),
            last_applied: LastApplied::None,
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

    pub fn prepare_draw(&mut self) {
        assert_ne!(
            self.pass, 
            DrawPass::UpdateContext,
            "prepare_draw called during an UpdateContext pass",
        );
        match std::mem::replace(&mut self.last_applied, LastApplied::Current) {
            LastApplied::Current => (),
            LastApplied::None => self.current.apply_all(self.context),
            LastApplied::Removed(old) => {
                DrawParams::apply_change(&old, &mut self.current, self.context);
            },
            LastApplied::History(index) => {
                DrawParams::apply_change(
                    &self.history[index],
                    &mut self.current,
                    self.context,
                );
            }
        }
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
    pub fn push<R, F: FnOnce(&mut Self) -> R>(&mut self, func: F) -> R {
        self.manually_push();
        let ret = func(self);
        self.manually_pop();
        ret
    }

    pub fn manually_push(&mut self) {
        let new = self.current.clone();
        let old = std::mem::replace(&mut self.current, new);
        self.history.push(old);
        if let LastApplied::Current = self.last_applied {
            self.last_applied = LastApplied::History(self.history.len() - 1);
        }
    }

    pub fn manually_pop(&mut self) {
        let new = self.history.pop().expect(
            "DrawContext::pop called more times than push!"
        );
        let old = std::mem::replace(&mut self.current, new);
        match &self.last_applied {
            LastApplied::History(index) => {
                use std::cmp::Ordering;
                self.last_applied = match index.cmp(&self.history.len()) {
                    Ordering::Less => LastApplied::History(*index),
                    Ordering::Equal => LastApplied::Current,
                    Ordering::Greater => {
                        debug_assert!(false, "DrawContext corrupted");
                        LastApplied::None
                    },
                };
            },
            LastApplied::Current => {
                self.last_applied = LastApplied::Removed(old);
            }
            _ => (),
        };
    }
}

impl<P: RenderPlatform> Drop for DrawContext<'_, P> {
    fn drop(&mut self) {
        if self.pass != DrawPass::UpdateContext {
            Self::prepare_draw(self);
        }
    }
}
