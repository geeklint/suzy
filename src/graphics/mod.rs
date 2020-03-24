use crate::platform::DrawParams;
pub use crate::platform::graphics::image;

pub trait Graphic {
    fn draw(&self, ctx: &mut DrawContext);
}

impl Graphic for () {
    fn draw(&self, _ctx: &mut DrawContext) {}
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

/*
pub trait DrawParams {
    fn apply_change(current: &Self, new: &mut Self);
}
*/

pub struct DrawContext {
    current: DrawParams,
    history: Vec<DrawParams>,
}

impl DrawContext {
    pub fn new(starting: DrawParams) -> Self {
        Self {
            current: starting,
            history: Vec::new(),
        }
    }

    pub fn push(ctx: &mut Self, new: DrawParams) {
        ctx.history.push(new);
        std::mem::swap(&mut ctx.current, ctx.history.last_mut().unwrap());
        DrawParams::apply_change(ctx.history.last().unwrap(), &mut ctx.current);
    }

    pub fn pop(ctx: &mut Self) {
        let mut last = ctx.history.pop().expect(
            "DrawContext::pop called more times than push!"
        );
        std::mem::swap(&mut ctx.current, &mut last);
        DrawParams::apply_change(&last, &mut ctx.current);
    }

    pub fn clone_current(&self) -> DrawParams {
        self.current.clone()
    }
}
