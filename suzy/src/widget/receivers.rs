/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use crate::graphics::{DrawContext, Graphic};
use crate::platform::{DefaultRenderPlatform, RenderPlatform};
use crate::pointer::PointerEvent;

use super::{AnonWidget, Widget, WidgetContent, WidgetGraphic};

/// An internal iterator style receiver.  Types of this trait are passed to
/// [`WidgetContent::children`](trait.WidgetContent.html#tymethod.children).
pub trait WidgetChildReceiver<T, P = DefaultRenderPlatform>
where
    T: ?Sized,
    P: ?Sized + RenderPlatform,
{
    /// Receive a child.
    fn child<F, Child>(&mut self, map_fn: F)
    where
        F: FnOnce(&mut T) -> &mut Widget<Child, P>,
        Child: WidgetContent<P>;

    fn iter_children<F, Child>(&mut self, iter_fn: F)
    where
        F: for<'a> FnOnce(
            &'a mut T,
        ) -> Box<
            dyn 'a + Iterator<Item = &'a mut Widget<Child, P>>,
        >,
        Child: WidgetContent<P>;

    /// Receive a child with an anonymous type.
    fn anon_child<F>(&mut self, map_fn: F)
    where
        F: FnOnce(&mut T) -> &mut dyn AnonWidget<P>;

    fn recurse<F, Child>(&mut self, map_fn: F)
    where
        F: FnOnce(&mut T) -> &mut Child,
        Child: WidgetContent<P>;
}

/// An internal iterator style receiver.  Types of this trait are passed to
/// [`WidgetContent::graphics`](trait.WidgetContent.html#tymethod.graphics).
pub trait WidgetGraphicReceiver<P = DefaultRenderPlatform>
where
    P: RenderPlatform + ?Sized,
{
    /// Receive a graphic.
    fn graphic<'g, T: WidgetGraphic<'g, 'g, P>>(&mut self, graphic: &'g mut T);
}

pub(super) struct DrawChildReceiver<
    'a,
    'b,
    T: ?Sized + WidgetContent<P>,
    P: RenderPlatform + ?Sized,
> {
    pub content: &'a mut T,
    pub ctx: &'a mut DrawContext<'b, P>,
}

impl<'a, 'b, T, P> WidgetChildReceiver<T, P>
    for DrawChildReceiver<'a, 'b, T, P>
where
    T: ?Sized + WidgetContent<P>,
    P: RenderPlatform + ?Sized,
{
    fn child<F, Child>(&mut self, map_fn: F)
    where
        F: FnOnce(&mut T) -> &mut Widget<Child, P>,
        Child: WidgetContent<P>,
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
        Child: WidgetContent<P>,
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

    fn recurse<F, Child>(&mut self, map_fn: F)
    where
        F: FnOnce(&mut T) -> &mut Child,
        Child: WidgetContent<P>,
    {
        Child::children(DrawChildReceiver {
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

impl<'a, 'b, 'c, T, P> WidgetChildReceiver<T, P>
    for PointerEventChildReceiver<'a, 'b, 'c, T>
where
    T: ?Sized + WidgetContent<P>,
    P: RenderPlatform + ?Sized,
{
    fn child<F, Child>(&mut self, map_fn: F)
    where
        F: FnOnce(&mut T) -> &mut Widget<Child, P>,
        Child: WidgetContent<P>,
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
        Child: WidgetContent<P>,
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

    fn recurse<F, Child>(&mut self, map_fn: F)
    where
        F: FnOnce(&mut T) -> &mut Child,
        Child: WidgetContent<P>,
    {
        Child::children(PointerEventChildReceiver {
            content: map_fn(self.content),
            event: self.event,
            handled: self.handled,
        });
    }
}

pub(super) struct DrawGraphicBeforeReceiver<'a, 'b, P>
where
    P: RenderPlatform + ?Sized,
{
    pub ctx: &'a mut DrawContext<'b, P>,
}

impl<'a, 'b, P> WidgetGraphicReceiver<P>
    for DrawGraphicBeforeReceiver<'a, 'b, P>
where
    P: RenderPlatform + ?Sized,
{
    fn graphic<'g, T: WidgetGraphic<'g, 'g, P>>(
        &mut self,
        graphic: &'g mut T,
    ) {
        graphic.before_children().draw(self.ctx);
    }
}

pub(super) struct DrawGraphicUnorderedReceiver<'a, 'b, P>
where
    P: RenderPlatform + ?Sized,
{
    pub ctx: &'a mut DrawContext<'b, P>,
    pub num_ordered: &'a mut u32,
}

impl<'a, 'b, P> WidgetGraphicReceiver<P>
    for DrawGraphicUnorderedReceiver<'a, 'b, P>
where
    P: RenderPlatform + ?Sized,
{
    fn graphic<'g, T: WidgetGraphic<'g, 'g, P>>(
        &mut self,
        graphic: &'g mut T,
    ) {
        if T::ordered() {
            *self.num_ordered += 1;
        } else {
            graphic.after_children().draw(self.ctx);
        }
    }
}

pub(super) struct DrawGraphicOrderedReceiver<'a, 'b, P>
where
    P: RenderPlatform + ?Sized,
{
    pub ctx: &'a mut DrawContext<'b, P>,
    pub target: u32,
    pub current: u32,
}

impl<'a, 'b, P> WidgetGraphicReceiver<P>
    for DrawGraphicOrderedReceiver<'a, 'b, P>
where
    P: RenderPlatform + ?Sized,
{
    fn graphic<'g, T: WidgetGraphic<'g, 'g, P>>(
        &mut self,
        graphic: &'g mut T,
    ) {
        if T::ordered() {
            if self.current == self.target {
                graphic.after_children().draw(self.ctx);
            }
            self.current += 1;
        }
    }
}
