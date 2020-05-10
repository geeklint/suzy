
pub trait RenderPlatform: 'static {
    type Global: 'static;
    type DrawParams: crate::graphics::DrawParams;

    fn with<F, R>(global: Box<Self::Global>, func: F) -> (Box<Self::Global>, R)
        where F: FnOnce() -> R
    {
        let gd = Box::new(GlobalData {
            data: global,
            next: None,
        });
        let res = gd.with(func);
        (res.0.data, res.1)
    }

    fn global<F, R>(func: F) -> R
        where F: FnOnce(&Self::Global) -> R,
    {
        GlobalData::with_current(func)
    }

    fn try_global<F, R>(func: F) -> Option<R>
        where F: FnOnce(&Self::Global) -> R,
    {
        GlobalData::try_with_current(func)
    }

}

pub trait Platform: 'static {
    type Window: crate::window::Window<Self::Renderer>;
    type Renderer: RenderPlatform;

    fn get_renderer_data(window: &mut Self::Window)
        -> <Self::Renderer as RenderPlatform>::Global;
}

pub trait SubRenderPlatform<P>: RenderPlatform
where
    P: RenderPlatform<Global = Self::Global>
{
    fn inherit_params(source: &<P as RenderPlatform>::DrawParams)
        -> Self::DrawParams;
}

use std::any::{TypeId, Any};
use std::cell::RefCell;
use std::collections::HashMap;

thread_local!{
    pub(super) static GLOBAL_DATA: RefCell<
        HashMap<TypeId, Option<Box<dyn Any>>>
    > = Default::default();
}

pub(super) struct GlobalData<T: 'static> {
    data: Box<T>,
    next: Option<Box<Self>>,
}

impl<T: 'static> GlobalData<T> {
    pub fn with<F, R>(mut self: Box<Self>, func: F) -> (Box<Self>, R)
        where F: FnOnce() -> R
    {
        let id = TypeId::of::<T>();
        GLOBAL_DATA.with(|cell| {
            let mut map = cell.borrow_mut();
            let entry = map.entry(id);
            let bucket = entry.or_default();
            self.next = bucket.take().map(|prev| {
                prev.downcast().ok()
            }).flatten();
            *bucket = Some(self as Box<dyn Any>);
        });
        let res = (func)();
        let self_restored = GLOBAL_DATA.with(|cell| {
            let mut map = cell.borrow_mut();
            let bucket = map.get_mut(&id).unwrap();
            let mut self_restored: Box<Self> = bucket.take()
                .and_then(|prev| {
                    prev.downcast().ok()
                })
                .unwrap();
            *bucket = self_restored.next.take().map(|x| x as Box<dyn Any>);
            self_restored
        });
        (self_restored, res)
    }

    pub fn try_with_current<F, R>(func: F) -> Option<R>
        where F: FnOnce(&T) -> R
    {
        let id = TypeId::of::<T>();
        GLOBAL_DATA.with(|cell| {
            let map = cell.borrow();
            let node = map.get(&id)
                .map(Option::as_ref)
                .flatten()?
                .downcast_ref::<Self>()
                .unwrap();
            Some((func)(&node.data))
        })
    }

    pub fn with_current<F, R>(func: F) -> R
        where F: FnOnce(&T) -> R
    {
        Self::try_with_current(func)
            .expect("no global data currently active")
    }
}
