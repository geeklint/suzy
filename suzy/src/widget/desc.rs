/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use crate::{
    app::AppState,
    platform::RenderPlatform,
    watch::{DefaultOwner, WatchArg},
};

use super::{layout, Ephemeral, Widget, WidgetGraphic, WidgetRect};

with_default_render_platform! {
    /// Instances of this trait are provided to
    /// [`widget::Content::desc`](crate::widget::Content::desc).
    ///
    /// It's primary use is to provide the `watch` method, which custom widgets
    /// use to submit watch closures.
    pub trait Desc<T, P>
    where
        T: ?Sized,
    {
        /// Register a watch function associated with this widget.  See the
        /// [watch](crate::watch) module for more information.
        fn watch<F>(&mut self, func: F)
        where
            F: 'static + Fn(&mut T, &WidgetRect);

        fn watch_explicit<F>(&mut self, func: F)
        where
            F: 'static
                + Fn(
                    &mut T,
                    &WidgetRect,
                    &AppState,
                    WatchArg<'_, 'static, DefaultOwner>,
                );

        /// Register a child of this widget.
        fn child<F, Child>(&mut self, map_fn: F)
        where
            F: 'static + Clone + Fn(&mut T) -> &mut Widget<Child>,
            Child: super::Content<P>;

        /// Register a graphic member of this widget.
        fn graphic<F, Gr>(&mut self, map_fn: F)
        where
            P: RenderPlatform,
            F: FnOnce(&mut T) -> &mut Gr,
            Gr: WidgetGraphic<P>;

        /// Create a layout group which a provides a shorthand for organizing
        /// widgets in common configurations.
        fn create_layout_group(&mut self) -> layout::LayoutTypes<'_, Self>
        where
            Self: Sized,
        {
            layout::LayoutTypes::new(self)
        }

        /// Register a coroutine with a factory function which creates the
        /// Future from the arguments provided to Coroutine::start.
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

        /// Register a variable number of children
        fn iter_children<F, Child>(&mut self, iter_fn: F)
        where
            F: 'static,
            F: for<'a> Fn(
                &'a mut T,
            ) -> Box<
                dyn 'a + Iterator<Item = &'a mut Ephemeral<Child>>,
            >,
            Child: super::Content<P>;

        /// Register a variable number of children
        fn iter_children_explicit<F, Child>(&mut self, iter_fn: F)
        where
            F: 'static,
            F: for<'a> Fn(
                &'a mut T,
                Option<WatchArg<'_, 'static, DefaultOwner>>,
            ) -> Box<
                dyn 'a + Iterator<Item = &'a mut Ephemeral<Child>>,
            >,
            Child: super::Content<P>;

        #[doc(hidden)]
        fn bare_child<F, Child>(&mut self, getter: F)
        where
            Child: super::Content<P>,
            F: 'static + Clone + Fn(&mut T) -> &mut Child;
    }
}
