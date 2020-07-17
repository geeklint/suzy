use crate::dims::{
    Dim,
    Rect,
};
use crate::graphics::DrawContext;
use crate::platform::{DefaultRenderPlatform, RenderPlatform};
use crate::pointer::PointerEvent;

use super::{Widget, WidgetId};
use super::WidgetContent;


pub(crate) trait AnonWidget<P: RenderPlatform>: crate::dims::DynRect {
    fn id(&self) -> WidgetId;
    fn draw(&mut self, ctx: &mut DrawContext<P>);
    fn pointer_event(&mut self, event: &mut PointerEvent) -> bool;
    fn pointer_event_self(&mut self, event: &mut PointerEvent) -> bool;
}

impl<P, T> AnonWidget<P> for Widget<T, P>
where
    P: RenderPlatform,
    T: WidgetContent<P>,
{
    fn id(&self) -> WidgetId {
        Widget::id(self)
    }

    fn draw(&mut self, ctx: &mut DrawContext<P>) {
        Widget::draw(self, ctx);
    }

    fn pointer_event(&mut self, event: &mut PointerEvent) -> bool {
        Widget::pointer_event(self, event)
    }

    fn pointer_event_self(&mut self, event: &mut PointerEvent) -> bool {
        Widget::pointer_event_self(self, event)
    }
}

/// A proxy to a widget with an unspecified underlying data type.
#[derive(Copy, Clone)]
pub struct WidgetProxy<'a, P: RenderPlatform = DefaultRenderPlatform> {
    pub(crate) anon: &'a dyn AnonWidget<P>,
}

/// A mutable proxy to a widget with an unspecified underlying data type.
pub struct WidgetProxyMut<'a, P: RenderPlatform = DefaultRenderPlatform> {
    pub(crate) anon: &'a mut dyn AnonWidget<P>,
}

pub struct OwnedWidgetProxy<P: RenderPlatform = DefaultRenderPlatform> {
    pub(crate) anon: Box<dyn AnonWidget<P>>,
}

impl<P, T> From<Widget<T, P>> for OwnedWidgetProxy<P>
where
    P: RenderPlatform,
    T: WidgetContent<P>,
{
    fn from(concrete: Widget<T, P>) -> OwnedWidgetProxy<P> {
        OwnedWidgetProxy { anon: Box::new(concrete) }
    }
}

impl<'a, P> From<&'a OwnedWidgetProxy<P>> for WidgetProxy<'a, P>
where
    P: RenderPlatform,
{
    fn from(owned: &'a OwnedWidgetProxy<P>) -> Self {
        WidgetProxy { anon: &*owned.anon }
    }
}

impl<'a, P> From<&'a mut OwnedWidgetProxy<P>> for WidgetProxyMut<'a, P>
where
    P: RenderPlatform,
{
    fn from(owned: &'a mut OwnedWidgetProxy<P>) -> Self {
        WidgetProxyMut { anon: &mut *owned.anon }
    }
}

impl<P: RenderPlatform> Rect for OwnedWidgetProxy<P> {
    fn x(&self) -> Dim { self.anon.x() }
    fn y(&self) -> Dim { self.anon.y() }

    fn x_mut<F, R>(&mut self, f: F) -> R
        where F: FnOnce(&mut Dim) -> R
    {
        let res = None;
        self.anon.x_mut(Box::new(|dim| {
            res = Some(f(dim));
        }));
        res.expect(
            "DynRect implementation did not call the closure passed to x_mut"
        )
    }

    fn y_mut<F, R>(&mut self, f: F) -> R
        where F: FnOnce(&mut Dim) -> R
    {
        let res = None;
        self.anon.y_mut(Box::new(|dim| {
            res = Some(f(dim));
        }));
        res.expect(
            "DynRect implementation did not call the closure passed to y_mut"
        )
    }

    fn set_left(&mut self, value: f32) { self.anon.set_left(value) }
    fn set_right(&mut self, value: f32) { self.anon.set_right(value) }
    fn set_bottom(&mut self, value: f32) { self.anon.set_bottom(value) }
    fn set_top(&mut self, value: f32) { self.anon.set_top(value) }
    fn set_center_x(&mut self, value: f32) { self.anon.set_center_x(value) }
    fn set_center_y(&mut self, value: f32) { self.anon.set_center_y(value) }
    fn set_center(&mut self, value: (f32, f32)) { self.anon.set_center(value) }
    fn set_width(&mut self, value: f32) { self.anon.set_width(value) }
    fn set_height(&mut self, value: f32) { self.anon.set_height(value) }
    fn set_pivot(&mut self, value: (f32, f32)) { self.anon.set_pivot(value) }
    fn set_pivot_pos(&mut self, value: (f32, f32)) { self.anon.set_pivot_pos(value) }
    fn set_fit_aspect(&mut self, aspect: f32) { self.anon.set_fit_aspect(aspect) }
    fn set_fill_aspect(&mut self, aspect: f32) { self.anon.set_fill_aspect(aspect) }
}
