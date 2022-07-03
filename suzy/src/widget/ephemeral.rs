/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use crate::{adapter::Adaptable, dims::Rect, platform::DefaultRenderPlatform};

use super::Widget;

struct Inner<T: ?Sized, P> {
    initialized: Cell<bool>,
    widget: RefCell<Widget<T, P>>,
}

pub struct Ephemeral<T: ?Sized, P = DefaultRenderPlatform> {
    ptr: Rc<Inner<T, P>>,
}

impl<T, P, Data> Adaptable<Data> for Ephemeral<T, P>
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

impl<T, P> Default for Ephemeral<T, P>
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
impl<T, P> Ephemeral<T, P> {
    fn from_widget(widget: Widget<T, P>) -> Self {
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

impl<T, P> Ephemeral<T, P>
where
    T: ?Sized,
{
    pub fn access<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&Widget<T, P>) -> R,
    {
        let wid_ref = self.ptr.widget.borrow();
        f(&*wid_ref)
    }

    pub fn access_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Widget<T, P>) -> R,
    {
        let mut wid_ref = self.ptr.widget.borrow_mut();
        f(&mut *wid_ref)
    }

    pub(super) fn uninit_holder(&self) -> Option<EphemeralHolder<T, P>> {
        (!self.ptr.initialized.get()).then(|| {
            self.ptr.initialized.set(true);
            EphemeralHolder {
                ptr: Rc::downgrade(&self.ptr),
            }
        })
    }
}

impl<T, P> Rect for Ephemeral<T, P>
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
    use std::rc::Weak;

    use super::Inner;

    use crate::{
        watch::{DefaultOwner, WatcherHolder},
        widget,
    };
    pub(in crate::widget) struct EphemeralHolder<T: ?Sized, P> {
        pub(super) ptr: Weak<Inner<T, P>>,
    }

    impl<T, P> Clone for EphemeralHolder<T, P>
    where
        T: ?Sized,
    {
        fn clone(&self) -> Self {
            let ptr = Weak::clone(&self.ptr);
            Self { ptr }
        }
    }

    impl<T, P> WatcherHolder<'static, DefaultOwner> for EphemeralHolder<T, P>
    where
        T: widget::Content<P>,
        P: 'static,
    {
        type Content = widget::WidgetInternal<P, T>;

        fn get_mut<F, R>(&self, _owner: &mut DefaultOwner, f: F) -> Option<R>
        where
            F: FnOnce(&mut Self::Content) -> R,
        {
            self.ptr.upgrade().map(|strong| {
                let mut wid_ref = strong.widget.borrow_mut();
                f(&mut wid_ref.internal)
            })
        }
    }
}
