use drying_paint::WatchedMeta;

use crate::graphics::{DrawContext, DrawPass};

use super::OpenGlRenderPlatform;
use super::OpenGlBindings;
use super::context::bindings::types::*;
use super::context::bindings::{
    ARRAY_BUFFER,
    DYNAMIC_DRAW,
    ELEMENT_ARRAY_BUFFER,
    STATIC_DRAW,
};

gl_object! { SingleBufferData, GenBuffers, DeleteBuffers, 1 }
gl_object! { TwoBufferData, GenBuffers, DeleteBuffers, 2 }
gl_object! { ThreeBufferData, GenBuffers, DeleteBuffers, 3 }

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
                    let gl = &DrawContext::render_ctx(draw_ctx).bindings;
                    unsafe {
                        gl.BindBuffer(ARRAY_BUFFER, self.obj.ids[0]);
                    }
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

    pub fn set_data<'a, F, OptT>(&mut self, make_data: F)
    where
        F: 'a + FnOnce() -> OptT,
        T: 'a,
        OptT: Into<Option<&'a [T]>>
    {
        self.tracker.watched();
        if let Some((ids, gl)) = self.obj.get() {
            if let Some(data) = (make_data)().into() {
                unsafe {
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
                }
                self.obj.mark_ready();
            }
        }
    }
}

pub struct DualVertexBuffer<T> {
    obj: TwoBufferData,
    tracker: WatchedMeta,
    dyn_draw: [bool; 2],
    ready: [bool; 2],
    remaining_to_draw: bool,
    _data: std::marker::PhantomData<[T]>,
}

impl<T> DualVertexBuffer<T> {
    pub fn new(dyn_draw_0: bool, dyn_draw_1: bool) -> Self {
        Self {
            obj: TwoBufferData::new(),
            tracker: WatchedMeta::new(),
            dyn_draw: [dyn_draw_0, dyn_draw_1],
            ready: [false, false],
            remaining_to_draw: true,
            _data: std::marker::PhantomData,
        }
    }

    pub fn check_ready<'a>(
        &'a mut self,
        draw_ctx: &'a mut DrawContext<OpenGlRenderPlatform>,
    ) -> Option<ReadyDualVertexBuffer<'a>> {
        let gl = &DrawContext::render_ctx(draw_ctx).bindings;
        match (self.ready[0], self.ready[1], self.obj.check_ready(gl)) {
            (true, true, true) => {
                match (DrawContext::pass(draw_ctx), self.remaining_to_draw) {
                    (DrawPass::DrawRemaining, false) => None,
                    (DrawPass::DrawAll, _)
                    | (DrawPass::DrawRemaining, true) => {
                        DrawContext::prepare_draw(draw_ctx);
                        self.remaining_to_draw = false;
                        let gl = &DrawContext::render_ctx(draw_ctx).bindings;
                        Some(ReadyDualVertexBuffer {
                            ids: &self.obj.ids,
                            gl,
                        })
                    },
                    (DrawPass::UpdateContext, _) => {
                        self.remaining_to_draw = true;
                        None
                    },
                }
            },
            (_, _, true) => {
                DrawContext::graphic_not_ready(draw_ctx);
                self.tracker.trigger();
                None
            },
            (_, _, false) => {
                self.ready = [false, false];
                DrawContext::graphic_not_ready(draw_ctx);
                self.tracker.trigger();
                None
            },
        }
    }

    pub fn set_data_0<'a, F, OptT>(&mut self, make_data: F)
    where
        F: 'a + FnOnce() -> OptT,
        T: 'a,
        OptT: Into<Option<&'a [T]>>
    {
        self.set_data(0, make_data);
    }

    pub fn set_data_1<'a, F, OptT>(&mut self, make_data: F)
    where
        F: 'a + FnOnce() -> OptT,
        T: 'a,
        OptT: Into<Option<&'a [T]>>
    {
        self.set_data(1, make_data);
    }

    fn set_data<'a, F, OptT>(&mut self, index: usize, make_data: F)
    where
        F: 'a + FnOnce() -> OptT,
        T: 'a,
        OptT: Into<Option<&'a [T]>>
    {
        self.tracker.watched();
        if let Some((ids, gl)) = self.obj.get() {
            if let Some(data) = (make_data)().into() {
                unsafe {
                    gl.BindBuffer(ARRAY_BUFFER, ids[index]);
                    gl.BufferData(
                        ARRAY_BUFFER,
                        (data.len() * std::mem::size_of::<T>()) as _,
                        data.as_ptr() as *const _,
                        if self.dyn_draw[index] {
                            DYNAMIC_DRAW
                        } else {
                            STATIC_DRAW
                        },
                    );
                }
                self.ready[index] = true;
                if self.ready == [true, true] {
                    self.obj.mark_ready();
                }
            }
        }
    }
}

pub struct ReadyDualVertexBuffer<'a> {
    ids: &'a [u32; 2],
    pub gl: &'a OpenGlBindings,
}

impl ReadyDualVertexBuffer<'_> {
    pub fn bind_0(&self) {
        unsafe {
            self.gl.BindBuffer(ARRAY_BUFFER, self.ids[0]);
        }
    }

    pub fn bind_1(&self) {
        unsafe {
            self.gl.BindBuffer(ARRAY_BUFFER, self.ids[1]);
        }
    }
}

pub struct DualVertexBufferIndexed<T> {
    obj: ThreeBufferData,
    tracker: WatchedMeta,
    dyn_draw: [bool; 3],
    ready: [bool; 3],
    remaining_to_draw: bool,
    _data: std::marker::PhantomData<[T]>,
}

impl<T> DualVertexBufferIndexed<T> {
    pub fn new(
        dyn_draw_0: bool,
        dyn_draw_1: bool,
        dyn_draw_indices: bool,
    ) -> Self {
        Self {
            obj: ThreeBufferData::new(),
            tracker: WatchedMeta::new(),
            dyn_draw: [dyn_draw_0, dyn_draw_1, dyn_draw_indices],
            ready: [false, false, false],
            remaining_to_draw: true,
            _data: std::marker::PhantomData,
        }
    }

    pub fn check_ready<'a>(
        &'a mut self,
        draw_ctx: &'a mut DrawContext<OpenGlRenderPlatform>,
    ) -> Option<ReadyDualVertexBufferIndexed<'a>> {
        let gl = &DrawContext::render_ctx(draw_ctx).bindings;
        match (self.ready[0], self.ready[1], self.obj.check_ready(gl)) {
            (true, true, true) => {
                match (DrawContext::pass(draw_ctx), self.remaining_to_draw) {
                    (DrawPass::DrawRemaining, false) => None,
                    (DrawPass::DrawAll, _)
                    | (DrawPass::DrawRemaining, true) => {
                        DrawContext::prepare_draw(draw_ctx);
                        self.remaining_to_draw = false;
                        let gl = &DrawContext::render_ctx(draw_ctx).bindings;
                        Some(ReadyDualVertexBufferIndexed {
                            ids: &self.obj.ids,
                            gl,
                        })
                    },
                    (DrawPass::UpdateContext, _) => {
                        self.remaining_to_draw = true;
                        None
                    },
                }
            },
            (_, _, true) => {
                DrawContext::graphic_not_ready(draw_ctx);
                self.tracker.trigger();
                None
            },
            (_, _, false) => {
                self.ready = [false, false, false];
                DrawContext::graphic_not_ready(draw_ctx);
                self.tracker.trigger();
                None
            },
        }
    }

    pub fn set_data_0<'a, F, OptT>(&mut self, make_data: F)
    where
        F: 'a + FnOnce() -> OptT,
        T: 'a,
        OptT: Into<Option<&'a [T]>>
    {
        self.set_data(0, make_data, ARRAY_BUFFER);
    }

    pub fn set_data_1<'a, F, OptT>(&mut self, make_data: F)
    where
        F: 'a + FnOnce() -> OptT,
        T: 'a,
        OptT: Into<Option<&'a [T]>>
    {
        self.set_data(1, make_data, ARRAY_BUFFER);
    }

    pub fn set_indices<'a, F, OptI>(&mut self, make_indices: F)
    where
        F: 'a + FnOnce() -> OptI,
        T: 'a,
        OptI: Into<Option<&'a [u8]>>
    {
        self.set_data(2, make_indices, ELEMENT_ARRAY_BUFFER);
    }

    fn set_data<'a, F, U, OptU>(
        &mut self,
        index: usize,
        make_data: F,
        target: GLenum,
    )
    where
        F: 'a + FnOnce() -> OptU,
        U: 'a,
        OptU: Into<Option<&'a [U]>>
    {
        self.tracker.watched();
        if let Some((ids, gl)) = self.obj.get() {
            if let Some(data) = (make_data)().into() {
                unsafe {
                    gl.BindBuffer(target, ids[index]);
                    gl.BufferData(
                        target,
                        (data.len() * std::mem::size_of::<U>()) as _,
                        data.as_ptr() as *const _,
                        if self.dyn_draw[index] {
                            DYNAMIC_DRAW
                        } else {
                            STATIC_DRAW
                        },
                    );
                }
                self.ready[index] = true;
                if self.ready == [true, true, true] {
                    self.obj.mark_ready();
                }
            }
        }
    }
}

pub struct ReadyDualVertexBufferIndexed<'a> {
    ids: &'a [u32; 3],
    pub gl: &'a OpenGlBindings,
}

impl ReadyDualVertexBufferIndexed<'_> {
    pub fn bind_indices(&self) {
        unsafe {
            self.gl.BindBuffer(ELEMENT_ARRAY_BUFFER, self.ids[2]);
        }
    }

    pub fn bind_0(&self) {
        unsafe {
            self.gl.BindBuffer(ARRAY_BUFFER, self.ids[0]);
        }
    }

    pub fn bind_1(&self) {
        unsafe {
            self.gl.BindBuffer(ARRAY_BUFFER, self.ids[1]);
        }
    }
}
