/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

mod effects;
mod image;
mod masker;

pub use image::{SelectableSlicedImage, SlicedImage};

pub use effects::{BaseEffect, Effect, Tint};
pub use masker::Masker;

use super::DrawParams;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DrawPass {
    DrawAll,
    UpdateContext,
    DrawRemaining,
}

pub struct DrawContext<'a> {
    context: &'a mut super::context::OpenGlContext,
    current: DrawParams,
    last_applied: Option<DrawParams>,
    pass: DrawPass,
}

impl<'a> crate::graphics::PlatformDrawContext<()> for DrawContext<'a> {
    fn finish(self) -> Option<()> {
        (self.pass == DrawPass::UpdateContext).then(|| ())
    }
}

impl<'a> DrawContext<'a> {
    pub(crate) fn new(
        ctx: &'a mut super::context::OpenGlContext,
        starting: DrawParams,
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

    pub fn pass(&self) -> DrawPass {
        self.pass
    }

    pub fn render_ctx(&self) -> &super::context::OpenGlContext {
        self.context
    }

    pub fn render_ctx_mut(&mut self) -> &mut super::context::OpenGlContext {
        &mut self.context
    }

    pub fn params(&mut self) -> &mut super::DrawParams {
        &mut self.current
    }

    pub fn graphic_not_ready(&mut self) {
        self.pass = DrawPass::UpdateContext;
    }

    pub fn force_redraw<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
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

    #[inline]
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
            }
            None => self.current.apply_all(self.context),
        }
        self.last_applied = Some(self.current.clone());
    }

    /// Any changes to the current DrawParams within this closure will be
    /// reverted when it finishes, allowing temporary adjustments.
    pub fn push<R, F: FnOnce(&mut Self) -> R>(&mut self, func: F) -> R {
        let backup = self.manually_push();
        let ret = func(self);
        self.manually_pop(backup);
        ret
    }

    #[must_use]
    pub fn manually_push(&mut self) -> DrawParams {
        self.current.clone()
    }

    pub fn manually_pop(&mut self, restore: DrawParams) {
        self.current = restore;
    }
}

impl Drop for DrawContext<'_> {
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
