use drying_paint::{WatcherMeta};

use crate::graphics::{Canvas};

use super::{WidgetData, WidgetRect, WidgetInternal};

/// This will get passed to a widget's initializer. It provides functions to
/// watch values for changes and run code when those values change
pub struct WidgetInit<'a, T: WidgetData> {
    pub(super) watcher: &'a mut WatcherMeta<WidgetInternal<T>>,
}

impl<T: WidgetData + 'static> WidgetInit<'_, T> {
    /// Register a simple watch which will get re-run whenever a value it
    /// references changes.
    pub fn watch<F>(&mut self, func: F)
        where F: Fn(&mut T, &WidgetRect) + 'static
    {
        self.watcher.watch(move |wid_int| {
            (func)(&mut wid_int.data, &wid_int.rect);
        });
    }

    /// Register a function which draws to the widget's canvas and updates
    /// whenever referenced values change.
    pub fn draw<F>(self, func: F)
        where F: Fn(&mut T, &WidgetRect, &mut Canvas) + 'static
    {
        self.watcher.watch(move |wid_int| {
            (func)(&mut wid_int.data, &wid_int.rect, &mut wid_int.canvas);
        });
    }

    /// Register a function which draws to the widget's canvas that renders
    /// on-top of it's children.
    pub fn draw_after<F>(self, func: F)
        where F: Fn(&mut T, &WidgetRect, &mut Canvas) + 'static
    {
        self.watcher.watch(move |wid_int| {
            (func)(
                &mut wid_int.data, &wid_int.rect, &mut wid_int.canvas_after);
        });
    }

    /// Register functions which draw to the widget's canvas before and after
    /// the children respectively.
    pub fn draw_before_and_after<F, G>(
        self, before: F, after: G)
        where F: Fn(&mut T, &WidgetRect, &mut Canvas) + 'static,
            G: Fn(&mut T, &WidgetRect, &mut Canvas) + 'static
    {
        self.watcher.watch(move |wid_int| {
            (before)(
                &mut wid_int.data, &wid_int.rect, &mut wid_int.canvas);
        });
        self.watcher.watch(move |wid_int| {
            (after)(
                &mut wid_int.data, &wid_int.rect, &mut wid_int.canvas_after);
        });
    }
}
