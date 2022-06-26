/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use crate::{
    dims::{Dim, Rect},
    watch,
};

use super::WidgetRect;

pub(super) struct WidgetInternal<P, T>
where
    T: super::Content<P> + ?Sized,
{
    pub(super) initialized: bool,
    pub(super) rect: WidgetRect,
    pub(super) _platform: std::marker::PhantomData<P>,
    pub(super) content: T,
}

impl<P, T> Default for WidgetInternal<P, T>
where
    T: super::Content<P> + Default + ?Sized,
{
    fn default() -> Self {
        Self {
            initialized: false,
            rect: WidgetRect::default(),
            _platform: std::marker::PhantomData,
            content: T::default(),
        }
    }
}

impl<P, T> watch::Watcher<'static> for WidgetInternal<P, T>
where
    T: super::Content<P>,
    Self: 'static,
{
    fn init(mut init: impl watch::WatcherInit<'static, Self>) {
        super::Content::desc(super::receivers::WidgetInitImpl {
            init: &mut init,
            getter: |x| x,
            _marker: std::marker::PhantomData,
        });
        init.watch(|this| {
            this.initialized = true;
        });
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
