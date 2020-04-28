use crate::graphics;
use crate::platform::{DefaultRenderPlatform, RenderPlatform};
use crate::pointer::PointerEvent;

use super::children::{WidgetChildren, WidgetChildrenMut};
use super::{WidgetView, WidgetInit};

/// This trait should be implemented for the types you provide as data for
/// Widget implementations.
pub trait WidgetContent<P = DefaultRenderPlatform>
where
    P: RenderPlatform,
    Self: Sized + 'static,
{
    /// This method provides a convient place to register functions which
    /// watch values and update parts of your widget when they change
    fn init(init: &mut WidgetInit<P, Self>);

    type ChildA: WidgetContent<P>;
    type ChildB: WidgetContent<P>;
    type ChildC: WidgetContent<P>;
    type ChildD: WidgetContent<P>;

    /// Custom widgets must define a way to iterate over their children
    /// if they want those children to be visible
    fn children(&self) ->
        WidgetChildren<
            P, Self::ChildA,Self::ChildB,Self::ChildC,Self::ChildD>
    ;

    /// Custom widgets must define a way to iterate over their children
    /// if they want those children to be visible
    fn children_mut(&mut self)
        -> WidgetChildrenMut<
            P, Self::ChildA,Self::ChildB,Self::ChildC,Self::ChildD>
    ;

    type Graphic: graphics::Graphic<P> + ?Sized;
    type GraphicAfter: graphics::Graphic<P> + ?Sized;

    fn graphic(&self) -> &Self::Graphic;
    fn graphic_after(&self) -> &Self::GraphicAfter;

    fn pointer_event(
        this: &mut WidgetView<'_, P, Self>,
        event: &mut PointerEvent,
    ) -> bool {
        let _unused = (this, event);
        false
    }
}

impl<P: RenderPlatform> WidgetContent<P> for () {
    fn init(_init: &mut WidgetInit<P, Self>) {}

    type ChildA = ();
    type ChildB = ();
    type ChildC = ();
    type ChildD = ();

    fn children(&self) -> WidgetChildren<P, (),(),(),()> {
        WidgetChildren::Zero
    }

    fn children_mut(&mut self) -> WidgetChildrenMut<P, (),(),(),()> {
        WidgetChildrenMut::Zero
    }

    type Graphic = ();
    type GraphicAfter = ();

    fn graphic(&self) -> &() { &() }
    fn graphic_after(&self) -> &() { &() }
}
