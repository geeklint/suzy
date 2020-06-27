use super::bindings::{
    ARRAY_BUFFER,
    DYNAMIC_DRAW,
    STATIC_DRAW,
};

use super::primitive::gl_object;

gl_object! { SingleBufferData, GenBuffers, DeleteBuffers, 1 }
gl_object! { TwoBufferData, GenBuffers, DeleteBuffers, 2 }

pub struct SingleVertexBuffer<T> {
    obj: SingleBufferData,
    tracker: WatchedMeta,
    dyn_draw: bool,
    _data: std::marker::PhantomData<[T]>,
}

impl<T> SingleVertexBuffer<T> {
    pub fn new(dyn_draw: bool) -> Self {
        Self {
            obj: SingleBufferData::new(),
            tracker: WatchedMeta::new(),
            dyn_draw,
            _data: std::marker::PhantomData,
        }
    }

    pub fn bind_if_ready(&mut self, gl: &Rc<OpenGlBindings>) -> bool {
        if self.obj.check_ready(gl) {
            gl.BindBuffer(ARRAY_BUFFER, ids[0]);
            true
        } else {
            self.tracker.trigger();
            false
        }
    }

    pub fn set_data(&mut self, data: &[T]) {
        self.tracker.watched();
        if let Some((ids, gl)) = self.obj.get() {
            gl.BindBuffer(ARRAY_BUFFER, ids[0]);
            gl.BufferData(
                ARRAY_BUFFER,
                (data.len() * mem::size_of::<T>()) as _,
                data.as_ptr() as *const c_void,
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
