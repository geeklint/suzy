/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use crate::platform::{DefaultRenderPlatform, RenderPlatform};

/// A trait for graphical state, with methods to apply changes to that state.
pub trait DrawParams<Ctx> {
    /// Apply the entirety of the current state, as if starting fresh.
    fn apply_all(&mut self, ctx: &mut Ctx);

    /// Apply just the differences between the states.
    fn apply_change(current: &Self, new: &mut Self, ctx: &mut Ctx);
}

/// At the start of drawing a frame, the pass associated with a context will
/// be set to `DrawAll`.  If, while drawing a frame, a graphic is encountered
/// which is not yet ready to be drawn, it should call
/// `DrawContext::graphic_not_ready`, which will update the current pass to
/// `UpdateContext`.  In this case, the remaining graphics should do what they
/// need to get ready.  If not all graphics have been drawn, a second pass
/// will run, with `DrawRemaining`.  If a graphic had previously been drawn
/// during a `DrawAll` pass, it should now do nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DrawPass {
    /// A pass during which all widgets should re-draw.
    DrawAll,
    /// A pass during which widgets should prepare to be drawn, but not
    /// actually draw.
    UpdateContext,
    /// A pass during which widgets which have not previously been drawn
    /// should draw now.
    DrawRemaining,
}

/// This type will get passed to Graphic::draw.
#[derive(Debug)]
pub struct DrawContext<'a, P = DefaultRenderPlatform>
where
    P: RenderPlatform + ?Sized,
{
    context: &'a mut P::Context,
    current: P::DrawParams,
    last_applied: Option<P::DrawParams>,
    pass: DrawPass,
}

impl<'a, P: RenderPlatform> DrawContext<'a, P> {
    pub(crate) fn new(
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

    /// Get the current state of the draw pass.  See the documenation of
    /// [`DrawPass`](./enum.DrawPass.html) for more information.
    pub fn pass(&self) -> DrawPass {
        self.pass
    }

    /// Get the RenderPlatfrom context associated with this DrawContex.
    pub fn render_ctx(&self) -> &P::Context {
        self.context
    }

    /// Get a mutable reference to the RenderPlatfrom context associated with
    /// this DrawContext.
    pub fn render_ctx_mut(&mut self) -> &mut P::Context {
        &mut self.context
    }

    /// Get a mutable reference to the current draw params.
    pub fn params(&mut self) -> &mut P::DrawParams {
        &mut self.current
    }

    /// Notify the context that a graphic is not ready to be drawn, and
    /// subsequent draw passes are needed.
    pub fn graphic_not_ready(&mut self) {
        self.pass = DrawPass::UpdateContext;
    }

    /// Run a closure with a version of this DrawContext as if it is a new
    /// frame (`DrawPass::DrawAll`) even if the current pass is DrawRemaing.
    ///
    /// A graphic drawn in this closure will be re-drawn, even if it was
    /// already drawn this frame.
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
    P::DrawParams: Clone,
{
    /// Apply all pending changes to the current draw params.
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

    /// Get a clone of the current DrawParams, which you can use to restore
    /// later using `manually_pop`.
    #[must_use]
    pub fn manually_push(&mut self) -> P::DrawParams {
        self.current.clone()
    }

    /// Restore a set of DrawParams.
    pub fn manually_pop(&mut self, restore: P::DrawParams) {
        self.current = restore;
    }
}

impl<P: RenderPlatform + ?Sized> Drop for DrawContext<'_, P> {
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
