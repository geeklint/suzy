
use super::children;
use super::{WidgetInit};

/// This trait should be implemented for the types you provide as data for
/// Widget implementations.
pub trait WidgetData: Sized {
    /// This method provides a convient place to register functions which
    /// watch values and update parts of your widget when they change
    fn init(init: &mut WidgetInit<Self>);

    type ChildA: WidgetData;
    type ChildB: WidgetData;
    type ChildC: WidgetData;
    type ChildD: WidgetData;

    /// Custom widgets must define a way to iterate over their children
    /// if they want those children to be visible
    fn children(&self)
        -> children::WidgetChildren<
            Self::ChildA,Self::ChildB,Self::ChildC,Self::ChildD>
    ;

    /// Custom widgets must define a way to iterate over their children
    /// if they want those children to be visible
    fn children_mut(&mut self)
        -> children::WidgetChildrenMut<
            Self::ChildA,Self::ChildB,Self::ChildC,Self::ChildD>
    ;
}

impl WidgetData for () {
    fn init(_init: &mut WidgetInit<Self>) {}

    type ChildA = ();
    type ChildB = ();
    type ChildC = ();
    type ChildD = ();

    fn children(&self) -> children::WidgetChildren<(),(),(),()> {
        children::WidgetChildren::Zero
    }

    fn children_mut(&mut self) -> children::WidgetChildrenMut<(),(),(),()> {
        children::WidgetChildrenMut::Zero
    }
}
