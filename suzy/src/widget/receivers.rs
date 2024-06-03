/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use crate::{
    app::{self, AppState},
    graphics::{DrawContext, Graphic},
    platform::RenderPlatform,
    pointer::PointerEvent,
    watch::{self, DefaultOwner, WatchArg, WatchName, WatchedMeta},
};

use super::{Desc, Ephemeral, Widget, WidgetGraphic, WidgetRect};

macro_rules! impl_empty {
    ($T:ident; $P:ident; watch) => {
        fn watch<F>(&mut self, _func: F)
        where
            F: 'static + Fn(&mut $T, &WidgetRect)  {}

        fn watch_explicit<F>(&mut self, _func: F)
        where
            F: 'static + Fn(&mut $T, &WidgetRect, &AppState, WatchArg<'_, 'static, DefaultOwner>){}
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
        fn iter_children_explicit<F, Child>(&mut self, _iter_fn: F)
        where
            F: for<'iter_children> Fn(
                &'iter_children mut $T,
                Option<WatchArg<'_, 'static, DefaultOwner>>,
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

pub(super) trait Holder<O: ?Sized>: Clone {
    type Content: ?Sized;

    fn get_mut<F>(&self, owner: &mut O, f: F)
    where
        F: FnOnce(&mut Self::Content, &mut WidgetRect);
}

impl<T: ?Sized, O: ?Sized> Holder<O> for Weak<RefCell<Widget<T>>> {
    type Content = T;

    fn get_mut<F>(&self, _owner: &mut O, f: F)
    where
        F: FnOnce(&mut Self::Content, &mut WidgetRect),
    {
        if let Some(strong) = self.upgrade() {
            let mut widget = strong.borrow_mut();
            let internal = &mut widget.internal;
            f(&mut internal.content, &mut internal.rect);
        }
    }
}

#[derive(Clone)]
struct MapHolder<Base, MapFn> {
    base: Base,
    map: MapFn,
}

impl<Base, MapFn, Leaf, Owner> Holder<Owner> for MapHolder<Base, MapFn>
where
    Base: Holder<Owner>,
    MapFn: Clone
        + for<'a> Fn(
            &'a mut Base::Content,
            &'a mut WidgetRect,
        ) -> (&'a mut Leaf, &'a mut WidgetRect),
    Leaf: ?Sized,
{
    type Content = Leaf;

    fn get_mut<F>(&self, owner: &mut Owner, f: F)
    where
        F: FnOnce(&mut Self::Content, &mut WidgetRect),
    {
        let map = &self.map;
        self.base.get_mut(owner, |base_content, base_rect| {
            let (leaf_content, leaf_rect) = map(base_content, base_rect);
            f(leaf_content, leaf_rect)
        });
    }
}

fn fix_closure<Branch, Leaf, F>(f: F) -> F
where
    Branch: ?Sized,
    Leaf: ?Sized,
    F: Clone
        + for<'a> Fn(
            &'a mut Branch,
            &'a mut WidgetRect,
        ) -> (&'a mut Leaf, &'a mut WidgetRect),
{
    f
}

pub(super) struct WidgetInitImpl<'a, Path> {
    pub watch_ctx: &'a mut watch::WatchContext<'static, watch::DefaultOwner>,
    pub state: &'a Rc<app::AppState>,
    pub path: Path,
}

impl<'a, Path> WidgetInitImpl<'a, Path> {
    #[track_caller]
    fn iter_children_raw<Leaf, Plat, F, Child>(&mut self, iter_fn: F)
    where
        F: 'static,
        F: for<'i> Fn(
            &'i mut Leaf,
            WatchArg<'_, 'static, DefaultOwner>,
        ) -> Box<
            dyn 'i + Iterator<Item = &'i mut Ephemeral<Child>>,
        >,
        Child: super::Content<Plat>,
        Leaf: ?Sized + super::Content<Plat>,
        Path: 'static + Holder<DefaultOwner, Content = Leaf>,
    {
        let watch_name = WatchName::from_caller();
        let maybe_more = WatchedMeta::<'static, DefaultOwner>::new();
        let current_path = self.path.clone();
        let state = Rc::clone(self.state);
        self.watch_ctx
            .add_watch_raw(watch_name, move |mut raw_arg| {
                let (owner, arg) = raw_arg.as_owner_and_arg();
                maybe_more.watched(arg);
                let mut holder = None;
                current_path.get_mut(owner, |content, _rect| {
                    holder =
                        iter_fn(content, arg).find_map(|e| e.uninit_holder());
                });
                if let Some(widget) = holder {
                    maybe_more.trigger_external();
                    widget.init(raw_arg.context(), &state);
                }
            });
    }
}

impl<'a, Leaf, Plat, Path> Desc<Leaf, Plat> for WidgetInitImpl<'a, Path>
where
    Leaf: ?Sized + super::Content<Plat>,
    Path: 'static + Holder<DefaultOwner, Content = Leaf>,
{
    impl_empty! { Leaf; Plat; graphic }

    #[track_caller]
    fn watch<F>(&mut self, func: F)
    where
        F: 'static + Fn(&mut Leaf, &WidgetRect),
    {
        let state = Rc::clone(self.state);
        self.watch_explicit(move |leaf, rect, _state, arg| {
            arg.use_as_current(|| {
                AppState::use_as_current(Rc::clone(&state), || {
                    func(leaf, rect);
                });
            });
        });
    }

    #[track_caller]
    fn watch_explicit<F>(&mut self, func: F)
    where
        F: 'static
            + Fn(
                &mut Leaf,
                &WidgetRect,
                &AppState,
                WatchArg<'_, 'static, DefaultOwner>,
            ),
    {
        let watch_name = WatchName::from_caller();
        let current_path = self.path.clone();
        let state = Rc::clone(self.state);
        self.watch_ctx
            .add_watch_raw(watch_name, move |mut raw_arg| {
                let (owner, arg) = raw_arg.as_owner_and_arg();
                current_path.get_mut(owner, |leaf, rect| {
                    func(leaf, rect, &state, arg)
                });
            });
    }

    fn child<F, Child>(&mut self, map_fn: F)
    where
        F: 'static + Clone + Fn(&mut Leaf) -> &mut Widget<Child>,
        Child: super::Content<Plat>,
    {
        Child::desc(WidgetInitImpl {
            watch_ctx: self.watch_ctx,
            state: self.state,
            path: MapHolder {
                base: self.path.clone(),
                map: fix_closure(move |content, _rect| {
                    let widget = map_fn(content);
                    (&mut widget.internal.content, &mut widget.internal.rect)
                }),
            },
        })
    }

    #[track_caller]
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
        self.iter_children_raw(move |leaf, arg| {
            let iter_fn = &iter_fn;
            arg.use_as_current(move || iter_fn(leaf))
        });
    }

    #[track_caller]
    fn iter_children_explicit<F, Child>(&mut self, iter_fn: F)
    where
        F: 'static,
        F: for<'b> Fn(
            &'b mut Leaf,
            Option<WatchArg<'_, 'static, DefaultOwner>>,
        ) -> Box<
            dyn 'b + Iterator<Item = &'b mut Ephemeral<Child>>,
        >,
        Child: super::Content<Plat>,
    {
        self.iter_children_raw(move |leaf, arg| iter_fn(leaf, Some(arg)));
    }

    fn bare_child<F, Child>(&mut self, getter: F)
    where
        Child: super::Content<Plat>,
        F: 'static + Clone + Fn(&mut Leaf) -> &mut Child,
    {
        Child::desc(WidgetInitImpl {
            watch_ctx: self.watch_ctx,
            state: self.state,
            path: MapHolder {
                base: self.path.clone(),
                map: fix_closure(move |content, rect| (getter(content), rect)),
            },
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

    fn iter_children_explicit<F, Child>(&mut self, iter_fn: F)
    where
        F: 'static,
        F: for<'i> Fn(
            &'i mut T,
            Option<WatchArg<'_, 'static, DefaultOwner>>,
        ) -> Box<
            dyn 'i + Iterator<Item = &'i mut Ephemeral<Child>>,
        >,
        Child: super::Content<P>,
    {
        let Self { content, ctx } = self;
        for child in iter_fn(content, None) {
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
        F: 'static,
        F: for<'i> Fn(
            &'i mut T,
        ) -> Box<
            dyn 'i + Iterator<Item = &'i mut Ephemeral<Child>>,
        >,
        Child: super::Content<P>,
    {
        self.iter_children_explicit(move |leaf, _arg| iter_fn(leaf));
    }

    fn iter_children_explicit<F, Child>(&mut self, iter_fn: F)
    where
        F: for<'i> Fn(
            &'i mut T,
            Option<WatchArg<'_, 'static, DefaultOwner>>,
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
        for child in iter_fn(content, None) {
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
