use drying_paint::{WatcherMeta, WatcherInit};

use crate::platform::RenderPlatform;

use super::{
    WidgetContent,
    WidgetId,
    WidgetInternal,
    WidgetView,
};

/// This will get passed to a widget's initializer. It provides functions to
/// watch values for changes and run code when those values change
pub struct WidgetInit<'a, P, T>
where
    P: RenderPlatform,
    T: WidgetContent<P>,
{
    pub(super) watcher: &'a mut WatcherMeta<WidgetInternal<P, T>>,
}

impl<P, T> WidgetInit<'_, P, T>
where
    P: RenderPlatform,
    T: WidgetContent<P>,
{
    /// Register a simple watch which will get re-run whenever a value it
    /// references changes.
    pub fn watch<F>(&mut self, func: F)
        where F: Fn(&mut WidgetView<'_, P, T>) + 'static
    {
        let id = WidgetId { id: self.watcher.id() };
        self.watcher.watch(move |wid_int| {
            let mut view = WidgetView {
                source: wid_int,
                id: id.clone(),
            };
            (func)(&mut view);
        });
    }
}

impl<P, T> WatcherInit for WidgetInternal<P, T>
where
    P: RenderPlatform,
    T: WidgetContent<P>,
{
    fn init(watcher: &mut WatcherMeta<Self>) {
        WidgetContent::init(&mut WidgetInit { watcher });
    }
}
