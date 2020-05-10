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

    fn apply_change(current: &Self, new: &Self);
}

pub struct DrawContext<P: RenderPlatform = DefaultRenderPlatform> {
    current: P::DrawParams,
    history: Vec<P::DrawParams>,
}

impl<P: RenderPlatform> DrawContext<P> {
    pub fn new(starting: P::DrawParams) -> Self {
        starting.apply_all();
        Self {
            current: starting,
            history: Vec::new(),
        }
    }

    pub fn push(ctx: &mut Self, new: P::DrawParams) {
        ctx.history.push(new);
        std::mem::swap(&mut ctx.current, ctx.history.last_mut().unwrap());
        DrawParams::apply_change(
            ctx.history.last().unwrap(),
            &mut ctx.current,
        );
    }

    pub fn pop(ctx: &mut Self) {
        let mut last = ctx.history.pop().expect(
            "DrawContext::pop called more times than push!"
        );
        std::mem::swap(&mut ctx.current, &mut last);
        DrawParams::apply_change(&last, &mut ctx.current);
    }

    pub fn clone_current(&self) -> P::DrawParams
        where P::DrawParams: Clone
    {
        self.current.clone()
    }

    pub fn descend<S, F, R>(&self, func: F) -> R
    where
        S: SubRenderPlatform<P> + RenderPlatform<Global = P::Global>,
        F: FnOnce(&mut DrawContext<S>) -> R,
    {
        let dp = S::inherit_params(&self.current);
        let res = (func)(&mut DrawContext::new(dp));
        self.current.apply_all();
        res
    }
}
