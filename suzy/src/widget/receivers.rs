/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use crate::graphics::{DrawContext, Graphic};
use crate::platform::{DefaultRenderPlatform, RenderPlatform};
use crate::pointer::PointerEvent;

use super::{AnonWidget, Widget, WidgetGraphic};

/// An internal iterator style receiver.  Types of this trait are passed to
/// [`widget::Content::children`](crate::widget::Content::children).
pub trait WidgetDescReceiver<T, P = DefaultRenderPlatform>
where
    T: ?Sized,
{
    /// Receive a child.
    fn child<F, Child>(&mut self, map_fn: F)
    where
        F: FnOnce(&mut T) -> &mut Widget<Child, P>,
        Child: super::Content<P>;

    fn iter_children<F, Child>(&mut self, iter_fn: F)
    where
        F: for<'a> FnOnce(
            &'a mut T,
        ) -> Box<
            dyn 'a + Iterator<Item = &'a mut Widget<Child, P>>,
        >,
        Child: super::Content<P>;

    /// Receive a child with an anonymous type.
    fn anon_child<F>(&mut self, map_fn: F)
    where
        F: FnOnce(&mut T) -> &mut dyn AnonWidget<P>;

    fn bare_child<F, Child>(&mut self, map_fn: F)
    where
        F: FnOnce(&mut T) -> &mut Child,
        Child: super::Content<P>;

    /// Receive a graphic.
    fn graphic<F, Gr>(&mut self, map_fn: F)
    where
        P: RenderPlatform,
        F: FnOnce(&mut T) -> &mut Gr,
        Gr: WidgetGraphic<P>;
}

macro_rules! impl_empty {
    ($T:ident; $P:ident; child) => {
        fn child<F, Child>(&mut self, _map_fn: F)
        where
            F: FnOnce(&mut $T) -> &mut Widget<Child, $P>,
            Child: super::Content<$P> {}
    };
    ($T:ident; $P:ident; iter_children) => {
        fn iter_children<F, Child>(&mut self, _iter_fn: F)
        where
            F: for<'iter_children> FnOnce(
                &'iter_children mut $T,
            ) -> Box<
                dyn 'iter_children + Iterator<Item = &'iter_children mut Widget<Child, $P>>,
            >,
            Child: super::Content<$P> {}
    };
    ($T:ident; $P:ident; anon_child) => {
        fn anon_child<F>(&mut self, _map_fn: F)
        where
            F: FnOnce(&mut $T) -> &mut dyn AnonWidget<$P> {}
    };

    ($T:ident; $P:ident; graphic) => {
        fn graphic<F, Gr>(&mut self, _map_fn: F)
        where
            F: FnOnce(&mut $T) -> &mut Gr,
            Gr: WidgetGraphic<$P>,
            P: RenderPlatform, {}
    };
    ($T:ident; $P:ident; $($method:ident)*) => {
        $(
            impl_empty!{ $T; $P; $method }
        )*
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

impl<'a, 'b, T, P> WidgetDescReceiver<T, P> for DrawChildReceiver<'a, 'b, T, P>
where
    T: ?Sized + super::Content<P>,
    P: RenderPlatform,
{
    impl_empty! { T; P; graphic }

    fn child<F, Child>(&mut self, map_fn: F)
    where
        F: FnOnce(&mut T) -> &mut Widget<Child, P>,
        Child: super::Content<P>,
    {
        Widget::draw(map_fn(self.content), self.ctx);
    }

    fn iter_children<F, Child>(&mut self, iter_fn: F)
    where
        F: for<'i> FnOnce(
            &'i mut T,
        ) -> Box<
            dyn 'i + Iterator<Item = &'i mut Widget<Child, P>>,
        >,
        Child: super::Content<P>,
    {
        for child in iter_fn(self.content) {
            Widget::draw(child, self.ctx);
        }
    }

    fn anon_child<F>(&mut self, map_fn: F)
    where
        F: FnOnce(&mut T) -> &mut dyn AnonWidget<P>,
    {
        map_fn(self.content).draw(self.ctx);
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

impl<'a, 'b, 'c, T, P> WidgetDescReceiver<T, P>
    for PointerEventChildReceiver<'a, 'b, 'c, T>
where
    T: ?Sized + super::Content<P>,
    P: 'static,
{
    impl_empty! { T; P; graphic }

    fn child<F, Child>(&mut self, map_fn: F)
    where
        F: FnOnce(&mut T) -> &mut Widget<Child, P>,
        Child: super::Content<P>,
    {
        if !*self.handled {
            *self.handled =
                Widget::pointer_event(map_fn(self.content), self.event);
        }
    }

    fn iter_children<F, Child>(&mut self, iter_fn: F)
    where
        F: for<'i> FnOnce(
            &'i mut T,
        ) -> Box<
            dyn 'i + Iterator<Item = &'i mut Widget<Child, P>>,
        >,
        Child: super::Content<P>,
    {
        for child in iter_fn(self.content) {
            if !*self.handled {
                *self.handled = Widget::pointer_event(child, self.event);
            }
        }
    }

    fn anon_child<F>(&mut self, map_fn: F)
    where
        F: FnOnce(&mut T) -> &mut dyn AnonWidget<P>,
    {
        if !*self.handled {
            *self.handled = map_fn(self.content).pointer_event(self.event);
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
    P: ?Sized + RenderPlatform,
{
    pub content: &'a mut T,
    pub ctx: &'a mut DrawContext<'b, P>,
}

impl<'a, 'b, T, P> WidgetDescReceiver<T, P>
    for DrawGraphicBeforeReceiver<'a, 'b, T, P>
where
    P: RenderPlatform,
{
    impl_empty! { T; P; child iter_children anon_child }

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
    P: RenderPlatform,
{
    pub content: &'a mut T,
    pub ctx: &'a mut DrawContext<'b, P>,
    pub num_ordered: &'a mut u32,
}

impl<'a, 'b, T, P> WidgetDescReceiver<T, P>
    for DrawGraphicUnorderedReceiver<'a, 'b, T, P>
where
    P: RenderPlatform,
{
    impl_empty! { T; P; child iter_children anon_child }

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
    P: RenderPlatform,
{
    pub content: &'a mut T,
    pub ctx: &'a mut DrawContext<'b, P>,
    pub target: u32,
    pub current: u32,
}

impl<'a, 'b, T, P> WidgetDescReceiver<T, P>
    for DrawGraphicOrderedReceiver<'a, 'b, T, P>
where
    P: RenderPlatform,
{
    impl_empty! { T; P; child iter_children anon_child }

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
