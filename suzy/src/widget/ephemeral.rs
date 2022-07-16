/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use crate::{adapter::Adaptable, dims::Rect};

use super::Widget;

struct Inner<T: ?Sized> {
    initialized: Cell<bool>,
    widget: RefCell<Widget<T>>,
}

pub struct Ephemeral<T: ?Sized> {
    ptr: Rc<Inner<T>>,
}

impl<T, Data> Adaptable<Data> for Ephemeral<T>
where
    T: Adaptable<Data>,
{
    fn adapt(&mut self, data: &Data) {
        self.ptr.widget.borrow_mut().adapt(data);
    }

    fn from(data: &Data) -> Self {
        Self {
            ptr: Rc::new(Inner {
                initialized: Cell::new(false),
                widget: RefCell::new(Adaptable::from(data)),
            }),
        }
    }
}

impl<T> Default for Ephemeral<T>
where
    T: Default,
{
    fn default() -> Self {
        Self {
            ptr: Rc::new(Inner {
                initialized: Cell::new(false),
                widget: RefCell::default(),
            }),
        }
    }
}

// Constructors
impl<T> Ephemeral<T> {
    fn from_widget(widget: Widget<T>) -> Self {
        Self {
            ptr: Rc::new(Inner {
                initialized: Cell::new(false),
                widget: RefCell::new(widget),
            }),
        }
    }

    pub fn new(content: T) -> Self {
        Self::from_widget(Widget::new(content))
    }

    pub fn new_with_rect<R>(content: T, rect: &R) -> Self
    where
        R: ?Sized + Rect,
    {
        Self::from_widget(Widget::new_with_rect(content, rect))
    }

    pub fn default_with_rect<R>(rect: &R) -> Self
    where
        T: Default,
        R: ?Sized + Rect,
    {
        Self::from_widget(Widget::default_with_rect(rect))
    }

    pub fn create_from<Data>(data: &Data) -> Self
    where
        T: Adaptable<Data>,
        Data: ?Sized,
    {
        Self::from_widget(Widget::create_from(data))
    }

    pub fn create_with_rect<Data, R>(data: &Data, rect: &R) -> Self
    where
        T: Adaptable<Data>,
        Data: ?Sized,
        R: ?Sized + Rect,
    {
        Self::from_widget(Widget::create_with_rect(data, rect))
    }
}

impl<T> Ephemeral<T>
where
    T: ?Sized,
{
    pub fn access<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&Widget<T>) -> R,
    {
        let wid_ref = self.ptr.widget.borrow();
        f(&*wid_ref)
    }

    pub fn access_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Widget<T>) -> R,
    {
        let mut wid_ref = self.ptr.widget.borrow_mut();
        f(&mut *wid_ref)
    }

    pub(super) fn uninit_holder<P>(&self) -> Option<EphemeralHolder<T, P>> {
        (!self.ptr.initialized.get()).then(|| {
            self.ptr.initialized.set(true);
            EphemeralHolder {
                ptr: Rc::downgrade(&self.ptr),
                marker: std::marker::PhantomData,
            }
        })
    }
}

impl<T> Rect for Ephemeral<T>
where
    T: ?Sized,
{
    fn x(&self) -> crate::dims::Dim {
        self.access(Rect::x)
    }

    fn y(&self) -> crate::dims::Dim {
        self.access(Rect::y)
    }

    fn x_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut crate::dims::Dim) -> R,
    {
        self.access_mut(|wid| wid.x_mut(f))
    }

    fn y_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut crate::dims::Dim) -> R,
    {
        self.access_mut(|wid| wid.y_mut(f))
    }
}

pub(super) use holder::EphemeralHolder;

mod holder {
    use std::rc::{Rc, Weak};

    use super::Inner;

    use crate::{app, watch, widget};
    pub(in crate::widget) struct EphemeralHolder<T: ?Sized, P> {
        pub(super) ptr: Weak<Inner<T>>,
        pub(super) marker: std::marker::PhantomData<fn() -> P>,
    }

    impl<T, P> Clone for EphemeralHolder<T, P>
    where
        T: ?Sized,
    {
        fn clone(&self) -> Self {
            let ptr = Weak::clone(&self.ptr);
            let marker = std::marker::PhantomData;
            Self { ptr, marker }
        }
    }

    impl<T, P> widget::receivers::Holder for EphemeralHolder<T, P>
    where
        T: ?Sized,
    {
        type Content = T;

        fn get_mut<F>(&self, f: F)
        where
            F: FnOnce(&mut Self::Content, &mut widget::WidgetRect),
        {
            if let Some(strong) = self.ptr.upgrade() {
                let mut widget = strong.widget.borrow_mut();
                let internal = &mut widget.internal;
                f(&mut internal.content, &mut internal.rect)
            }
        }
    }

    impl<T, P> EphemeralHolder<T, P>
    where
        T: ?Sized + widget::Content<P>,
        P: 'static,
    {
        pub(crate) fn init(
            self,
            watch_ctx: &mut watch::WatchContext<'static, watch::DefaultOwner>,
            state: &Rc<app::AppState>,
        ) {
            use crate::widget::receivers::WidgetInitImpl;
            T::desc(WidgetInitImpl {
                watch_ctx,
                state,
                path: self,
            })
        }
    }
}
