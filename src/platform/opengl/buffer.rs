use drying_paint::WatchedMeta;

use crate::graphics::{DrawContext, DrawPass};

use super::OpenGlRenderPlatform;
use super::context::bindings::{
    ARRAY_BUFFER,
    DYNAMIC_DRAW,
    STATIC_DRAW,
};

#[macro_use] use super::primitive;

gl_object! { SingleBufferData, GenBuffers, DeleteBuffers, 1 }
gl_object! { TwoBufferData, GenBuffers, DeleteBuffers, 2 }

pub struct SingleVertexBuffer<T> {
    obj: SingleBufferData,
    tracker: WatchedMeta,
    dyn_draw: bool,
    remaining_to_draw: bool,
    _data: std::marker::PhantomData<[T]>,
}

impl<T> SingleVertexBuffer<T> {
    pub fn new(dyn_draw: bool) -> Self {
        Self {
            obj: SingleBufferData::new(),
            tracker: WatchedMeta::new(),
            dyn_draw,
            remaining_to_draw: true,
            _data: std::marker::PhantomData,
        }
    }

    pub fn bind_if_ready(
        &mut self,
        draw_ctx: &mut DrawContext<OpenGlRenderPlatform>,
    ) -> bool {
        let gl = &DrawContext::render_ctx(draw_ctx).bindings;
        if self.obj.check_ready(gl) {
            match (DrawContext::pass(draw_ctx), self.remaining_to_draw) {
                (DrawPass::DrawRemaining, false) => false,
                (DrawPass::DrawAll, _)
                | (DrawPass::DrawRemaining, true) => {
                    DrawContext::prepare_draw(draw_ctx);
                    gl.BindBuffer(ARRAY_BUFFER, self.obj.ids[0]);
                    self.remaining_to_draw = false;
                    true
                },
                (DrawPass::UpdateContext, _) => {
                    self.remaining_to_draw = true;
                    false
                },
            }
        } else {
            DrawContext::graphic_not_ready(draw_ctx);
            self.tracker.trigger();
            false
        }
    }

    pub fn set_data<'a, F>(&mut self, make_data: F)
    where
        F: FnOnce() -> &'a [u8] + 'a
    {
        self.tracker.watched();
        if let Some((ids, gl)) = self.obj.get() {
            let data = (make_data)();
            gl.BindBuffer(ARRAY_BUFFER, ids[0]);
            gl.BufferData(
                ARRAY_BUFFER,
                (data.len() * std::mem::size_of::<T>()) as _,
                data.as_ptr() as *const _,
                if self.dyn_draw {
                    DYNAMIC_DRAW
                } else {
                    STATIC_DRAW
                },
            );
            self.obj.mark_ready();
        }
    }
}
