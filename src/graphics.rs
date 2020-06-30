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

pub trait DrawParams<Ctx> {
    fn apply_all(&mut self, ctx: &mut Ctx);

    fn apply_change(current: &Self, new: &mut Self, ctx: &mut Ctx);
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum LastApplied<T> {
    History(usize),
    Current,
    None,
    Removed(T),
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
pub struct DrawContext<'a, P: RenderPlatform = DefaultRenderPlatform> {
    context: &'a mut P::Context,
    current: P::DrawParams,
    history: Vec<P::DrawParams>,
    last_applied: LastApplied<P::DrawParams>,
    pass: DrawPass,
}

impl<'a, P: RenderPlatform> DrawContext<'a, P> {
    pub fn new(ctx: &'a mut P::Context, starting: P::DrawParams) -> Self {
        Self {
            contex: ctx,
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
        match std::mem::replace(ctx.last_applied, LastApplied::Current) {
            LastApplied::Current => (),
            LastApplied::None => ctx.current.apply_all(ctx.context),
            LastApplied::Removed(old) => {
                DrawParams::apply_change(&old, &ctx.current, ctx.context);
            },
            LastApplied::History(index) => {
                DrawParams::apply_change(
                    &ctx.history[index],
                    &ctx.current,
                    ctx.context,
                );
            }
        }
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

impl<'a, P> DrawContext<'a, P>
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
                LastApplied::Removed(old),
            },
            LastApplied::Removed(params) => LastApplied::Removed(params),
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
