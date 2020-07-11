use crate::platform::{DefaultRenderPlatform, RenderPlatform};
use crate::pointer::PointerEvent;

use super::{
    WidgetChildReceiver,
    WidgetGraphicReceiver,
    WidgetInit,
    WidgetMutChildReceiver,
    WidgetView,
};

/// This trait should be implemented for the types you provide as data for
/// Widget implementations.
pub trait WidgetContent<P = DefaultRenderPlatform>
where
    P: RenderPlatform,
    Self: Sized + 'static,
{
    /// This method provides a convient place to register functions which
    /// watch values and update parts of your widget when they change
    fn init(init: &mut WidgetInit<Self, P>);

    /// Custom widgets must define a way to iterate over their children
    /// if they want those children to be visible
    fn children<R: WidgetChildReceiver<P>>(&self, receiver: R);

    /// Custom widgets must define a way to iterate over their children
    /// if they want those children to be visible
    fn children_mut<R: WidgetMutChildReceiver<P>>(&mut self, receiver: R);

    fn graphics<R: WidgetGraphicReceiver<P>>(&mut self, receiver: R);

    fn pointer_event(
        this: &mut WidgetView<'_, P, Self>,
        event: &mut PointerEvent,
    ) -> bool {
        let _unused = (this, event);
        false
    }
}

impl<P: RenderPlatform> WidgetContent<P> for () {
    fn init(_init: &mut WidgetInit<Self, P>) {}
    fn children<R: WidgetChildReceiver<P>>(&self, _receiver: R) {}
    fn children_mut<R: WidgetMutChildReceiver<P>>(&mut self, _receiver: R) {}
    fn graphics<R: WidgetGraphicReceiver<P>>(&mut self, _receiver: R) {}
}
