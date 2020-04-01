use crate::graphics::DrawContext;
use crate::pointer::PointerEvent;

use super::{Widget, WidgetId};
use super::WidgetContent;


pub(crate) trait AnonWidget {
    fn id(&self) -> WidgetId;
    fn draw(&self, ctx: &mut DrawContext);
    fn pointer_event(&mut self, event: &mut PointerEvent) -> bool;
}

impl<T: WidgetContent> AnonWidget for Widget<T> {
    fn id(&self) -> WidgetId {
        Widget::id(self)
    }

    fn draw(&self, ctx: &mut DrawContext) {
        Widget::draw(self, ctx);
    }

    fn pointer_event(&mut self, event: &mut PointerEvent) -> bool {
        Widget::pointer_event(self, event)
    }
}

/// A proxy to a widget with an unspecified underlying data type.
#[derive(Copy, Clone)]
pub struct WidgetProxy<'a> {
    pub(crate) anon: &'a dyn AnonWidget,
}

/// A mutable proxy to a widget with an unspecified underlying data type.
pub struct WidgetProxyMut<'a> {
    pub(crate) anon: &'a mut dyn AnonWidget,
}

pub struct OwnedWidgetProxy {
    pub(crate) anon: Box<dyn AnonWidget>,
}

impl<T: WidgetContent> From<Widget<T>> for OwnedWidgetProxy {
    fn from(concrete: Widget<T>) -> OwnedWidgetProxy {
        OwnedWidgetProxy { anon: Box::new(concrete) }
    }
}

impl<'a> From<&'a OwnedWidgetProxy> for WidgetProxy<'a> {
    fn from(owned: &OwnedWidgetProxy) -> WidgetProxy {
        WidgetProxy { anon: &*owned.anon }
    }
}

impl<'a> From<&'a mut OwnedWidgetProxy> for WidgetProxyMut<'a> {
    fn from(owned: &mut OwnedWidgetProxy) -> WidgetProxyMut {
        WidgetProxyMut { anon: &mut *owned.anon }
    }
}
