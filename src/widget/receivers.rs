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
    fn graphic<T: Graphic<P>>(&mut self, graphic: &mut T);
}

pub(super) struct DrawChildReceiver<'a, 'b, P: RenderPlatform> {
    pub ctx: &'a mut DrawContext<'b, P>,
}

pub(super) struct PointerEventChildReceiver<'a, 'b, 'c> {
    pub event: &'a mut PointerEvent<'c>,
    pub handled: &'b mut bool,
}

pub(super) struct DrawGraphicReceiver<'a, 'b, P: RenderPlatform> {
    pub ctx: &'a mut DrawContext<'b, P>,
}

impl<'a, 'b, P> WidgetMutChildReceiver<P> for DrawChildReceiver<'a, 'b, P>
where P: RenderPlatform
{
    fn child<T: WidgetContent<P>>(&mut self, child: &mut Widget<T, P>) {
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

impl<'a, 'b, P> WidgetGraphicReceiver<P> for DrawGraphicReceiver<'a, 'b, P>
where
    P: RenderPlatform,
{
    fn graphic<T: Graphic<P>>(&mut self, graphic: &mut T) {
        graphic.draw(self.ctx);
    }
}
