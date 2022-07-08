/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use crate::{
    dims::{Dim, Rect},
    watch,
};

use super::WidgetRect;

pub(super) struct WidgetInternal<T>
where
    T: ?Sized,
{
    pub(super) rect: WidgetRect,
    pub(super) content: T,
}

impl<T> Default for WidgetInternal<T>
where
    T: Default,
{
    fn default() -> Self {
        Self {
            rect: WidgetRect::default(),
            content: T::default(),
        }
    }
}

/// This type is provided to widget event methods, providing access to the
/// widget's Rect and other functionality.
pub struct WidgetExtra<'a> {
    pub(super) rect: &'a mut WidgetRect,
}

impl Rect for WidgetExtra<'_> {
    fn x(&self) -> Dim {
        self.rect.x()
    }
    fn y(&self) -> Dim {
        self.rect.y()
    }

    fn x_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Dim) -> R,
    {
        self.rect.x_mut(f)
    }

    fn y_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Dim) -> R,
    {
        self.rect.y_mut(f)
    }
}

#[repr(transparent)]
pub(super) struct WatcherImpl<T: ?Sized, P> {
    _platform: std::marker::PhantomData<fn() -> P>,
    internal: T,
}

impl<T: ?Sized> WidgetInternal<T> {
    pub fn as_watcher<P>(&mut self) -> &mut WatcherImpl<Self, P>
    where
        T: super::Content<P>,
    {
        let ptr = self as *mut WidgetInternal<T> as *mut WatcherImpl<Self, P>;
        unsafe { &mut *ptr }
    }
}

impl<T, P> watch::Watcher<'static> for WatcherImpl<WidgetInternal<T>, P>
where
    T: ?Sized + super::Content<P>,
    Self: 'static,
{
    fn init(mut init: impl watch::WatcherInit<'static, Self>) {
        super::Content::desc(super::receivers::WidgetInitImpl {
            init: &mut init,
            getter: |this: &mut Self| {
                (&mut this.internal.content, &mut this.internal.rect)
            },
            _marker: std::marker::PhantomData,
        });
    }
}
