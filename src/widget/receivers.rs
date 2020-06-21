use crate::graphics::{Graphic, DrawContext};
use crate::platform::{DefaultRenderPlatform, RenderPlatform};
use crate::pointer::PointerEvent;

use super::{Widget, WidgetContent};

pub trait WidgetChildReceiver<P = DefaultRenderPlatform>
where
    P: RenderPlatform,
{
    fn child<T: WidgetContent<P>>(&mut self, child: &Widget<T, P>);
}

pub trait WidgetMutChildReceiver<P = DefaultRenderPlatform>
where
    P: RenderPlatform,
{
    fn child<T: WidgetContent<P>>(&mut self, child: &mut Widget<T, P>);
}

pub trait WidgetGraphicReceiver<P = DefaultRenderPlatform>
where
    P: RenderPlatform,
{
    fn graphic<T: Graphic<P>>(&mut self, graphic: &T);
}

pub(super) struct DrawChildReceiver<'a, P: RenderPlatform> {
    pub ctx: &'a mut DrawContext<P>,
}

pub(super) struct PointerEventChildReceiver<'a, 'b, 'c> {
    pub event: &'a mut PointerEvent<'c>,
    pub handled: &'b mut bool,
}

pub(super) struct DrawGraphicReceiver<'a, P: RenderPlatform> {
    pub ctx: &'a mut DrawContext<P>,
}

impl<'a, P> WidgetChildReceiver<P> for DrawChildReceiver<'a, P>
where P: RenderPlatform
{
    fn child<T: WidgetContent<P>>(&mut self, child: &Widget<T, P>) {
        child.draw(self.ctx);
    }
}

impl<'a, 'b, 'c, P> WidgetMutChildReceiver<P>
for PointerEventChildReceiver<'a, 'b, 'c>
where
    P: RenderPlatform,
{
    fn child<T: WidgetContent<P>>(&mut self, child: &mut Widget<T, P>) {
        if !*self.handled {
            *self.handled = child.pointer_event(self.event);
        }
    }
}

impl<'a, P> WidgetGraphicReceiver<P> for DrawGraphicReceiver<'a, P>
where
    P: RenderPlatform,
{
    fn graphic<T: Graphic<P>>(&mut self, graphic: &T) {
        graphic.draw(self.ctx);
    }
}
