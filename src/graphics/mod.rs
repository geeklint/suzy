use crate::math::Color;

mod drawparams;
pub mod image;

pub trait Graphic {
    fn draw(&self, ctx: &mut DrawContext);
}

impl Graphic for () {
    fn draw(&self, ctx: &mut DrawContext) {}
}

impl<T: Graphic> Graphic for [T] {
    fn draw(&self, ctx: &mut DrawContext) {
        for graphic in self {
            graphic.draw(ctx);
        }
    }
}

impl Graphic for [Box<dyn Graphic>] {
    fn draw(&self, ctx: &mut DrawContext) {
        for graphic in self {
            graphic.draw(ctx);
        }
    }
}

pub use drawparams::DrawParams;

pub struct DrawContext {
    current: DrawParams,
    history: Vec<DrawParams>,
}

impl std::ops::Deref for DrawContext {
    type Target = DrawParams;
    fn deref(&self) -> &DrawParams { &self.current }
}

impl DrawContext {
    pub(crate) fn new(starting: DrawParams) -> Self {
        Self {
            current: starting,
            history: Vec::new(),
        }
    }

    pub fn push(ctx: &mut Self) -> &mut DrawParams {
        let mut params = ctx.current.clone();
        std::mem::swap(&mut ctx.current, &mut params);
        ctx.history.push(params);
        &mut ctx.current
    }

    pub fn pop(ctx: &mut Self) {
        ctx.current = ctx.history.pop().expect(
            "DrawContext::pop called more times than push!"
        );
    }
}
