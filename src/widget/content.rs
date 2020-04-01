use crate::graphics;
use crate::pointer::PointerEvent;

use super::children;
use super::{Widget, WidgetInit};

/// This trait should be implemented for the types you provide as data for
/// Widget implementations.
pub trait WidgetContent: Sized {
    /// This method provides a convient place to register functions which
    /// watch values and update parts of your widget when they change
    fn init(init: &mut WidgetInit<Self>);

    type ChildA: WidgetContent;
    type ChildB: WidgetContent;
    type ChildC: WidgetContent;
    type ChildD: WidgetContent;

    /// Custom widgets must define a way to iterate over their children
    /// if they want those children to be visible
    fn children(&self)
        -> WidgetChildren<Self::ChildA,Self::ChildB,Self::ChildC,Self::ChildD>
    ;

    /// Custom widgets must define a way to iterate over their children
    /// if they want those children to be visible
    fn children_mut(&mut self)
        -> WidgetChildrenMut<
            Self::ChildA,Self::ChildB,Self::ChildC,Self::ChildD>
    ;

    type Graphic: graphics::Graphic + ?Sized;
    type GraphicAfter: graphics::Graphic + ?Sized;

    fn graphic(&self) -> &Self::Graphic;
    fn graphic_after(&self) -> &Self::GraphicAfter;

    fn pointer_event(
        this: &mut WidgetView<'_, Self>,
        event: &mut PointerEvent,
    ) -> bool {
        let _unused = (this, event);
        false
    }
}

impl WidgetContent for () {
    fn init(_init: &mut WidgetInit<Self>) {}

    type ChildA = ();
    type ChildB = ();
    type ChildC = ();
    type ChildD = ();

    fn children(&self) -> WidgetChildren<(),(),(),()> {
        WidgetChildren::Zero
    }

    fn children_mut(&mut self) -> WidgetChildrenMut<(),(),(),()> {
        WidgetChildrenMut::Zero
    }

    type Graphic = ();
    type GraphicAfter = ();

    fn graphic(&self) -> &() { &() }
    fn graphic_after(&self) -> &() { &() }
}
