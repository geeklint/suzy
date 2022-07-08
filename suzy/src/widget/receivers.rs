/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use crate::{
    graphics::{DrawContext, Graphic},
    platform::RenderPlatform,
    pointer::PointerEvent,
    watch,
};

use super::{Desc, Ephemeral, Widget, WidgetGraphic, WidgetRect};

macro_rules! impl_empty {
    ($T:ident; $P:ident; watch) => {
        fn watch<F>(&mut self, _func: F)
        where
            F: Fn(&mut T, &mut WidgetRect) + 'static {}
    };
    ($T:ident; $P:ident; child) => {
        fn child<F, Child>(&mut self, _map_fn: F)
        where
            F: FnOnce(&mut $T) -> &mut Widget<Child>,
            Child: super::Content<$P> {}
    };
    ($T:ident; $P:ident; graphic) => {
        fn graphic<F, Gr>(&mut self, _map_fn: F)
        where
            F: FnOnce(&mut $T) -> &mut Gr,
            Gr: WidgetGraphic<$P>,
            $P: RenderPlatform, {}
    };
    ($T:ident; $P:ident; iter_children) => {
        fn iter_children<F, Child>(&mut self, _iter_fn: F)
        where
            F: for<'iter_children> Fn(
                &'iter_children mut $T,
            ) -> Box<
                dyn 'iter_children + Iterator<Item = &'iter_children mut Ephemeral<Child>>,
            >,
            Child: super::Content<$P> {}
    };
    ($T:ident; $P:ident; $($method:ident)*) => {
        $(
            impl_empty!{ $T; $P; $method }
        )*
    }
}

pub(super) struct WidgetInitImpl<'a, Init, Getter, Base, Leaf>
where
    Base: ?Sized,
    Leaf: ?Sized,
    Getter: 'static + Clone + Fn(&mut Base) -> (&mut Leaf, &mut WidgetRect),
{
    pub init: &'a mut Init,
    pub getter: Getter,
    pub _marker: std::marker::PhantomData<&'a mut Base>,
}

impl<'a, Init, Base, Leaf, Plat, Getter> Desc<Leaf, Plat>
    for WidgetInitImpl<'a, Init, Getter, Base, Leaf>
where
    Init: watch::WatcherInit<'static, Base>,
    Base: ?Sized,
    Leaf: ?Sized + super::Content<Plat>,
    Plat: 'static,
    Getter: 'static + Clone + Fn(&mut Base) -> (&mut Leaf, &mut WidgetRect),
{
    impl_empty! { Leaf; Plat; graphic }

    fn watch<F>(&mut self, func: F)
    where
        F: Fn(&mut Leaf, &mut WidgetRect) + 'static,
    {
        let getter = self.getter.clone();
        self.init.watch(move |base| {
            let (content, rect) = getter(base);
            (func)(content, rect);
        });
    }

    fn child<F, Child>(&mut self, map_fn: F)
    where
        F: 'static + Clone + FnOnce(&mut Leaf) -> &mut Widget<Child>,
        Child: super::Content<Plat>,
    {
        let getter = self.getter.clone();
        self.init.init_child(move |base| {
            (map_fn.clone())(getter(base).0)
                .internal
                .as_watcher::<Plat>()
        });
    }

    fn iter_children<F, Child>(&mut self, iter_fn: F)
    where
        F: 'static,
        F: for<'b> Fn(
            &'b mut Leaf,
        ) -> Box<
            dyn 'b + Iterator<Item = &'b mut Ephemeral<Child>>,
        >,
        Child: super::Content<Plat>,
    {
        use crate::watch::{DefaultOwner, WatchedMeta};
        let getter = self.getter.clone();
        let maybe_more = WatchedMeta::<'static, DefaultOwner>::new();
        self.init.watch_for_new_child(move |base| {
            maybe_more.watched_auto();
            let (content, _rect) = getter(base);
            let holder =
                iter_fn(content).filter_map(|e| e.uninit_holder()).next();
            if holder.is_some() {
                maybe_more.trigger_external();
            }
            holder
        });
    }

    fn bare_child<F, Child>(&mut self, getter: F)
    where
        Child: super::Content<Plat>,
        F: 'static + Clone + Fn(&mut Leaf) -> &mut Child,
    {
        let current_getter = self.getter.clone();
        super::Content::desc(WidgetInitImpl {
            init: self.init,
            getter: move |base| {
                let (base_content, rect) = current_getter(base);
                (getter(base_content), rect)
            },
            _marker: std::marker::PhantomData,
        });
    }
}

pub(super) struct DrawChildReceiver<
    'a,
    'b,
    T: ?Sized + super::Content<P>,
    P: RenderPlatform,
> {
    pub content: &'a mut T,
    pub ctx: &'a mut DrawContext<'b, P>,
}

impl<'a, 'b, T, P> Desc<T, P> for DrawChildReceiver<'a, 'b, T, P>
where
    T: ?Sized + super::Content<P>,
    P: RenderPlatform,
{
    impl_empty! { T; P; watch graphic }

    fn child<F, Child>(&mut self, map_fn: F)
    where
        F: FnOnce(&mut T) -> &mut Widget<Child>,
        Child: super::Content<P>,
    {
        Widget::draw(map_fn(self.content), self.ctx);
    }

    fn iter_children<F, Child>(&mut self, iter_fn: F)
    where
        F: for<'i> Fn(
            &'i mut T,
        ) -> Box<
            dyn 'i + Iterator<Item = &'i mut Ephemeral<Child>>,
        >,
        Child: super::Content<P>,
    {
        let Self { content, ctx } = self;
        for child in iter_fn(content) {
            child.access_mut(|widget| {
                Widget::draw(widget, ctx);
            });
        }
    }

    fn bare_child<F, Child>(&mut self, map_fn: F)
    where
        F: FnOnce(&mut T) -> &mut Child,
        Child: super::Content<P>,
    {
        Child::desc(DrawChildReceiver {
            content: map_fn(self.content),
            ctx: self.ctx,
        });
    }
}

pub(super) struct PointerEventChildReceiver<'a, 'b, 'c, T: ?Sized> {
    pub content: &'a mut T,
    pub event: &'a mut PointerEvent<'c>,
    pub handled: &'b mut bool,
}

impl<'a, 'b, 'c, T, P> Desc<T, P> for PointerEventChildReceiver<'a, 'b, 'c, T>
where
    T: ?Sized + super::Content<P>,
{
    impl_empty! { T; P; watch graphic }

    fn child<F, Child>(&mut self, map_fn: F)
    where
        F: FnOnce(&mut T) -> &mut Widget<Child>,
        Child: super::Content<P>,
    {
        if !*self.handled {
            *self.handled =
                Widget::pointer_event(map_fn(self.content), self.event);
        }
    }

    fn iter_children<F, Child>(&mut self, iter_fn: F)
    where
        F: for<'i> Fn(
            &'i mut T,
        ) -> Box<
            dyn 'i + Iterator<Item = &'i mut Ephemeral<Child>>,
        >,
        Child: super::Content<P>,
    {
        let Self {
            content,
            event,
            handled,
        } = self;
        for child in iter_fn(content) {
            if !**handled {
                **handled = child
                    .access_mut(|widget| Widget::pointer_event(widget, event));
            }
        }
    }

    fn bare_child<F, Child>(&mut self, map_fn: F)
    where
        F: FnOnce(&mut T) -> &mut Child,
        Child: super::Content<P>,
    {
        Child::desc(PointerEventChildReceiver {
            content: map_fn(self.content),
            event: self.event,
            handled: self.handled,
        });
    }
}

pub(super) struct DrawGraphicBeforeReceiver<'a, 'b, T, P>
where
    T: ?Sized,
    P: RenderPlatform,
{
    pub content: &'a mut T,
    pub ctx: &'a mut DrawContext<'b, P>,
}

impl<'a, 'b, T, P> Desc<T, P> for DrawGraphicBeforeReceiver<'a, 'b, T, P>
where
    T: ?Sized,
    P: RenderPlatform,
{
    impl_empty! { T; P; watch child iter_children }

    fn graphic<F, Gr>(&mut self, map_fn: F)
    where
        F: FnOnce(&mut T) -> &mut Gr,
        Gr: WidgetGraphic<P>,
    {
        map_fn(self.content).before_children().draw(self.ctx);
    }

    fn bare_child<F, Child>(&mut self, map_fn: F)
    where
        F: FnOnce(&mut T) -> &mut Child,
        Child: super::Content<P>,
    {
        Child::desc(DrawGraphicBeforeReceiver {
            content: map_fn(self.content),
            ctx: self.ctx,
        });
    }
}

pub(super) struct DrawGraphicUnorderedReceiver<'a, 'b, T, P>
where
    T: ?Sized,
    P: RenderPlatform,
{
    pub content: &'a mut T,
    pub ctx: &'a mut DrawContext<'b, P>,
    pub num_ordered: &'a mut u32,
}

impl<'a, 'b, T, P> Desc<T, P> for DrawGraphicUnorderedReceiver<'a, 'b, T, P>
where
    T: ?Sized,
    P: RenderPlatform,
{
    impl_empty! { T; P; watch child iter_children }

    fn graphic<F, Gr>(&mut self, map_fn: F)
    where
        F: FnOnce(&mut T) -> &mut Gr,
        Gr: WidgetGraphic<P>,
    {
        if Gr::ordered() {
            *self.num_ordered += 1;
        } else {
            map_fn(self.content).after_children().draw(self.ctx);
        }
    }

    fn bare_child<F, Child>(&mut self, map_fn: F)
    where
        F: FnOnce(&mut T) -> &mut Child,
        Child: super::Content<P>,
    {
        Child::desc(DrawGraphicUnorderedReceiver {
            content: map_fn(self.content),
            ctx: self.ctx,
            num_ordered: self.num_ordered,
        });
    }
}

pub(super) struct DrawGraphicOrderedReceiver<'a, 'b, T, P>
where
    T: ?Sized,
    P: RenderPlatform,
{
    pub content: &'a mut T,
    pub ctx: &'a mut DrawContext<'b, P>,
    pub target: u32,
    pub current: u32,
}

impl<'a, 'b, T, P> Desc<T, P> for DrawGraphicOrderedReceiver<'a, 'b, T, P>
where
    T: ?Sized,
    P: RenderPlatform,
{
    impl_empty! { T; P; watch child iter_children }

    fn graphic<F, Gr>(&mut self, map_fn: F)
    where
        F: FnOnce(&mut T) -> &mut Gr,
        Gr: WidgetGraphic<P>,
    {
        if Gr::ordered() {
            if self.current == self.target {
                map_fn(self.content).after_children().draw(self.ctx);
            }
            self.current += 1;
        }
    }

    fn bare_child<F, Child>(&mut self, map_fn: F)
    where
        F: FnOnce(&mut T) -> &mut Child,
        Child: super::Content<P>,
    {
        Child::desc(DrawGraphicOrderedReceiver {
            content: map_fn(self.content),
            ctx: self.ctx,
            target: self.target,
            current: self.current,
        });
    }
}
