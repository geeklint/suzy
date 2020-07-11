use drying_paint::{WatcherMeta, WatcherInit};

use crate::platform::{DefaultRenderPlatform, RenderPlatform};

use super::{
    WidgetContent,
    WidgetId,
    WidgetInternal,
    WidgetRect,
};

/// This will get passed to a widget's initializer. It provides functions to
/// watch values for changes and run code when those values change
pub struct WidgetInit<'a, T, P = DefaultRenderPlatform>
where
    P: RenderPlatform,
    T: WidgetContent<P>,
{
    pub(super) watcher: &'a mut WatcherMeta<WidgetInternal<P, T>>,
}

impl<P, T> WidgetInit<'_, T, P>
where
    P: RenderPlatform,
    T: WidgetContent<P>,
{
    /// Get a value representing a unique id for the widget this WidgetInit
    /// was created for. This value may outlive the widget, and will never
    /// compare equal to a value returned by the id method of a Widget other
    /// than this one.
    pub fn widget_id(&self) -> WidgetId {
        WidgetId { id: self.watcher.id() }
    }

    /// Register a simple watch which will get re-run whenever a value it
    /// references changes.
    pub fn watch<F>(&mut self, func: F)
        where F: Fn(&mut T, &mut WidgetRect) + 'static
    {
        self.watcher.watch(move |wid_int| {
            (func)(&mut wid_int.content, &mut wid_int.rect);
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
