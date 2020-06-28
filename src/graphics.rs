use crate::platform::{
    DefaultRenderPlatform,
    RenderPlatform,
    SubRenderPlatform,
};

pub trait Graphic<P: RenderPlatform = DefaultRenderPlatform> {
    fn draw(&self, ctx: &mut DrawContext<P>);
}

impl<P: RenderPlatform> Graphic<P> for () {
    fn draw(&self, _ctx: &mut DrawContext<P>) {}
}

impl<P: RenderPlatform, T: Graphic<P>> Graphic<P> for [T] {
    fn draw(&self, ctx: &mut DrawContext<P>) {
        for graphic in self {
            graphic.draw(ctx);
        }
    }
}

pub trait DrawParams {
    fn apply_all(&self);

    fn apply_change(current: &Self, new: &mut Self);
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum LastApplied {
    History(usize),
    Current,
    None,
}

impl Default for LastApplied {
    fn default() -> Self { Self::None }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DrawPass {
    DrawAll,
    UpdateContext,
    DrawRemaining,
}

#[derive(Debug, Default)]
pub struct DrawContext<P: RenderPlatform = DefaultRenderPlatform> {
    current: P::DrawParams,
    history: Vec<P::DrawParams>,
    last_applied: LastApplied,
    pass: DrawPass,
}

impl<P: RenderPlatform> DrawContext<P> {
    pub fn new(starting: P::DrawParams) -> Self {
        Self {
            current: starting,
            history: Vec::new(),
            last_applied: LastApplied::None,
            pass: DrawPass::DrawAll,
        }
    }

    pub fn pass(ctx: &Self) -> DrawPass { ctx.pass }

    pub fn graphic_not_ready(ctx: &mut Self) {
        self.pass = DrawPass::UpdateContext;
    }

    pub fn prepare_draw(ctx: &mut Self) {
        match ctx.last_applied {
            LastApplied::Current => (),
            LastApplied::None => ctx.current.apply_all(),
            LastApplied::History(index) => {
                DrawParams::apply_change(
                    &ctx.history[index],
                    &ctx.current,
                );
            }
        }
        ctx.last_applied = LastApplied::Current;
    }

    pub(crate) fn draw<T>(ctx: &mut Self, root: &mut Widget<T, P>) -> bool
    where
        T: WidgetContent<P>,
    {
        if self.pass == DrawPass::UpdateContext {
            self.pass = DrawRemaining;
        }
        Widget::draw(root, &mut ctx);
        ctx.pass == DrawPass::UpdateContext
    }
}

impl<P> DrawContext<P>
where
    P: RenderPlatform,
    P::DrawParams: Clone
{
    pub fn push(ctx: &mut Self) {
        let new = ctx.current.clone();
        let old = std::mem::replace(&mut ctx.current, new);
        ctx.history.push(old);
        if ctx.last_applied == LastApplied::Current {
            ctx.last_applied = LastApplied::History(ctx.history.len() - 1),
        }
    }

    pub fn pop(ctx: &mut Self) {
        let new = ctx.history.pop().expect(
            "DrawContext::pop called more times than push!"
        );
        let old = std::mem::replace(&mut ctx.current, new);
        ctx.last_applied = match ctx.last_applied {
            LastApplied::History(index) => {
                if index < ctx.history.len() {
                    LastApplied::History(index)
                } else if index == ctx.history.len() {
                    LastApplied::Current
                } else {
                    debug_assert!(false, "DrawContext corrupted");
                    LastApplied::None
                }
            },
            LastApplied::Current => {
                DrawParams::apply_change(&old, &ctx.current);
                LastApplied::Current;
            },
            LastApplied::None => LastApplied::None,
        }
    }
}

impl<P: RenderPlatform> Deref for DrawContext<P> {
    type Target = P::DrawParams;
    fn deref(&self) -> &P::DrawParams { &self.current }
}

impl<P: RenderPlatform> DerefMut for DrawContext<P> {
    fn deref_mut(&mut self) -> &mut P::DrawParams { &mut self.current }
}
