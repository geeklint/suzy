use drying_paint::{WatcherMeta, WatcherInit};

use super::{
    WidgetContent,
    WidgetId,
    WidgetInternal,
    WidgetView,
};

/// This will get passed to a widget's initializer. It provides functions to
/// watch values for changes and run code when those values change
pub struct WidgetInit<'a, T: WidgetContent> {
    pub(super) watcher: &'a mut WatcherMeta<WidgetInternal<T>>,
}

impl<T: WidgetContent> WidgetInit<'_, T> {
    /// Register a simple watch which will get re-run whenever a value it
    /// references changes.
    pub fn watch<F>(&mut self, func: F)
        where F: Fn(&mut WidgetView<'_, T>) + 'static
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

impl<T: WidgetContent> WatcherInit for WidgetInternal<T> {
    fn init(watcher: &mut WatcherMeta<Self>) {
        WidgetContent::init(&mut WidgetInit { watcher });
    }
}
