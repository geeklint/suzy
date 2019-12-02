use crate::interaction::{InteractionReceiver};

use super::{WidgetInit, WidgetProxy, WidgetProxyMut};

/// This trait should be implemented for the types you provide as data for
/// Widget implementations.
pub trait WidgetData: Sized + InteractionReceiver {
    /// This method provides a convient place to register functions which
    /// watch values and update parts of your widget when they change
    fn init(init: &mut WidgetInit<Self>);

    /// Custom widgets must define a way to iterate over their children
    /// if they want those children to be visible
    fn children<'a>(&'a self) -> Vec<WidgetProxy<'a>>;

    /// Custom widgets must define a way to iterate over their children
    /// if they want those children to be visible
    fn children_mut<'a>(&'a mut self) -> Vec<WidgetProxyMut<'a>>;
}
