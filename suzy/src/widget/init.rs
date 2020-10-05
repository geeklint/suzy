/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use drying_paint::{WatcherMeta, WatcherInit};

use crate::platform::{DefaultRenderPlatform, RenderPlatform};

use super::{
    WidgetContent,
    WidgetId,
    WidgetInternal,
    WidgetRect,
};

/// Instances of this trait are provided to
/// [`WidgetContent::init`](trait.WidgetContent.html#tymethod.init).
///
/// It's primary use is to provide the `watch` method, which custom widgets
/// use to submit watch closures.
pub trait WidgetInit<T, P = DefaultRenderPlatform>
where
    P: RenderPlatform + ?Sized,
    T: WidgetContent<P> + ?Sized,
{
    /// Get a value representing a unique id for the widget this WidgetInit
    /// was created for. This value may outlive the widget, and will never
    /// compare equal to a value returned by the id method of a Widget other
    /// than this one.
    fn widget_id(&self) -> WidgetId;

    /// Register a watch function associated with this widget.  See the
    /// [watch](../watch/index.html) module for more information.
    fn watch<F>(&mut self, func: F)
    where
        F: Fn(&mut T, &mut WidgetRect) + 'static
    ;

    #[doc(hidden)]
    fn init_child_inline<F, C>(&mut self, getter: F)
    where
        C: WidgetContent<P>,
        F: 'static + Clone + Fn(&mut T) -> &mut C,
    ;
}

struct WidgetInitImpl<'a, 'b, O, T, G, P>
where
    P: RenderPlatform + ?Sized,
    G: 'static + Clone + Fn(&mut O) -> &mut T,
    O: WidgetContent<P>,
    T: WidgetContent<P>,
{
    watcher: &'a mut WatcherMeta<'b, WidgetInternal<P, O>>,
    getter: G,
}

impl<O, T, G, P> WidgetInit<T, P> for WidgetInitImpl<'_, '_, O, T, G, P>
where
    P: RenderPlatform + ?Sized,
    G: 'static + Clone + Fn(&mut O) -> &mut T,
    O: WidgetContent<P>,
    T: WidgetContent<P>,
{
    fn widget_id(&self) -> WidgetId {
        WidgetId { id: self.watcher.id() }
    }

    fn watch<F>(&mut self, func: F)
        where F: Fn(&mut T, &mut WidgetRect) + 'static
    {
        let getter = self.getter.clone();
        self.watcher.watch(move |wid_int| {
            let content = getter(&mut wid_int.content);
            (func)(content, &mut wid_int.rect);
        });
    }

    fn init_child_inline<F, C>(&mut self, getter: F)
    where
        C: WidgetContent<P>,
        F: 'static + Clone + Fn(&mut T) -> &mut C,
    {
        let current_getter = self.getter.clone();
        WidgetContent::init(WidgetInitImpl {
            watcher: self.watcher,
            getter: move |base| getter(current_getter(base)),
        });
    }
}

impl<P, T> WatcherInit for WidgetInternal<P, T>
where
    P: RenderPlatform + ?Sized,
    T: WidgetContent<P>,
{
    fn init(watcher: &mut WatcherMeta<Self>) {
        WidgetContent::init(WidgetInitImpl {
            watcher,
            getter: |x| x,
        });
    }
}
