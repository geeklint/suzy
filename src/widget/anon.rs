use crate::graphics::DrawContext;
use crate::platform::{DefaultRenderPlatform, RenderPlatform};
use crate::pointer::PointerEvent;

use super::{Widget, WidgetId};
use super::WidgetContent;


pub(crate) trait AnonWidget<P: RenderPlatform> {
    fn id(&self) -> WidgetId;
    fn draw(&mut self, ctx: &mut DrawContext<P>);
    fn pointer_event(&mut self, event: &mut PointerEvent) -> bool;
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
