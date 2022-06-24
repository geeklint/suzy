/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use drying_paint::{WatcherInit, WatcherMeta};

use crate::platform::DefaultRenderPlatform;

use super::{layout, WidgetInternal, WidgetRect};

/// Instances of this trait are provided to
/// [`widget::Content::init`](trait.widget::Content.html#tymethod.init).
///
/// It's primary use is to provide the `watch` method, which custom widgets
/// use to submit watch closures.
pub trait WidgetInit<T, P = DefaultRenderPlatform>
where
    T: super::Content<P> + ?Sized,
{
    /// Register a watch function associated with this widget.  See the
    /// [watch](../watch/index.html) module for more information.
    fn watch<F>(&mut self, func: F)
    where
        F: Fn(&mut T, &mut WidgetRect) + 'static;

    /// Create a layout group which a provides a shorthand for organizing
    /// widgets in common configurations.
    fn create_layout_group(&mut self) -> layout::LayoutTypes<Self, T, P> {
        layout::LayoutTypes::new(self)
    }

    /// Register a coroutine with a factory function which creates the Future
    /// from the arguments provided to Coroutine::start.
    fn register_coroutine<Get, Args, Fac, Fut>(
        &mut self,
        coroutine: Get,
        factory: Fac,
    ) where
        Get: 'static + Fn(&mut T) -> &mut super::Coroutine<Args>,
        Fut: 'static + std::future::Future<Output = ()>,
        Fac: 'static + Fn(Args) -> Fut,
    {
        super::Coroutine::register(coroutine, self, factory);
    }

    #[doc(hidden)]
    fn init_child_inline<F, C>(&mut self, getter: F)
    where
        C: super::Content<P>,
        F: 'static + Clone + Fn(&mut T) -> &mut C;
}

struct WidgetInitImpl<'a, 'b, O, T, G, P>
where
    G: 'static + Clone + Fn(&mut O) -> &mut T,
    O: super::Content<P>,
    T: super::Content<P>,
{
    watcher: &'a mut WatcherMeta<'b, WidgetInternal<P, O>>,
    getter: G,
}

impl<O, T, G, P> WidgetInit<T, P> for WidgetInitImpl<'_, '_, O, T, G, P>
where
    G: 'static + Clone + Fn(&mut O) -> &mut T,
    O: super::Content<P>,
    T: super::Content<P>,
    WidgetInternal<P, O>: 'static,
{
    fn watch<F>(&mut self, func: F)
    where
        F: Fn(&mut T, &mut WidgetRect) + 'static,
    {
        let getter = self.getter.clone();
        self.watcher.watch(move |wid_int| {
            let content = getter(&mut wid_int.content);
            (func)(content, &mut wid_int.rect);
        });
    }

    fn init_child_inline<F, C>(&mut self, getter: F)
    where
        C: super::Content<P>,
        F: 'static + Clone + Fn(&mut T) -> &mut C,
    {
        let current_getter = self.getter.clone();
        super::Content::init(WidgetInitImpl {
            watcher: self.watcher,
            getter: move |base| getter(current_getter(base)),
        });
    }
}

impl<P, T> WatcherInit for WidgetInternal<P, T>
where
    T: super::Content<P>,
    Self: 'static,
{
    fn init(watcher: &mut WatcherMeta<Self>) {
        super::Content::init(WidgetInitImpl {
            watcher,
            getter: |x| x,
        });
    }
}
