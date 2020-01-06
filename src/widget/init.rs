use drying_paint::{WatcherMeta};

use super::{WidgetContent, WidgetRect, WidgetInternal};

/// This will get passed to a widget's initializer. It provides functions to
/// watch values for changes and run code when those values change
pub struct WidgetInit<'a, T: WidgetContent> {
    pub(super) watcher: &'a mut WatcherMeta<WidgetInternal<T>>,
}

impl<T: WidgetContent + 'static> WidgetInit<'_, T> {
    /// Register a simple watch which will get re-run whenever a value it
    /// references changes.
    pub fn watch<F>(&mut self, func: F)
        where F: Fn(&mut T, &WidgetRect) + 'static
    {
        self.watcher.watch(move |wid_int| {
            (func)(&mut wid_int.data, &wid_int.rect);
        });
    }
}
