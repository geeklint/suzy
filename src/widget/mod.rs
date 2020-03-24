use std::cell::{Ref, RefMut};

use drying_paint::{Watcher, WatcherMeta, WatcherInit, WatcherId};
pub use drying_paint::Watched;

use crate::dims::{Rect, Dim};
use crate::graphics::{Graphic, DrawContext};
use crate::pointer::PointerEvent;

mod anon;
pub mod children;
mod content;
mod init;
mod newwidget;
mod rect;

pub use anon::{OwnedWidgetProxy, WidgetProxy, WidgetProxyMut};
pub use content::WidgetContent;
pub use init::WidgetInit;
pub use newwidget::NewWidget;
pub use rect::WidgetRect;

/// A basic structure to wrap some data and turn it into a widget.
#[derive(Default)]
pub struct Widget<T: WidgetContent> {
    watcher: Watcher<WidgetInternal<T>>,
}

#[derive(Default)]
struct WidgetInternal<T: WidgetContent> {
    rect: WidgetRect,
    data: T,
}

impl<T: WidgetContent + 'static> Widget<T> {
    pub fn id(&self) -> WidgetId {
        WidgetId {
            id: self.watcher.id(),
        }
    }

    /// Get an anonymous reference to this widget. This is required by
    /// WidgetContent::children(), for example.
    pub fn proxy(&self) -> WidgetProxy {
        WidgetProxy { anon: self }
    }

    /// Get an mutable anonymous reference to this widget. This is required
    /// by WidgetContent::children_mut(), for example.
    pub fn proxy_mut(&mut self) -> WidgetProxyMut {
        WidgetProxyMut { anon: self }
    }
}

impl<T: WidgetContent> Widget<T> {
    fn internal(&self) -> Ref<WidgetInternal<T>> { self.watcher.data() }
    fn internal_mut(&mut self) -> RefMut<WidgetInternal<T>> {
        self.watcher.data_mut()
    }

    pub fn data(&self) -> Ref<T> { Ref::map(self.internal(), |w| &w.data) }
    pub fn data_mut(&mut self) -> RefMut<T> {
        RefMut::map(self.internal_mut(), |w| &mut w.data)
    }

    pub(crate) fn draw(&self, ctx: &mut DrawContext) {
        let wid_int = self.internal();
        let data = &wid_int.data;
        data.graphic().draw(ctx);
        data.children().draw(ctx);
        data.graphic_after().draw(ctx);
    }

    pub(crate) fn pointer_event(&mut self, event: &mut PointerEvent) -> bool {
        let handled_by_child = {
            let data = &mut self.internal_mut().data;
            data.children_mut().pointer_event(event)
        };
        handled_by_child || T::pointer_event(self, event)
    }
}

impl<T> Widget<T>
where T: WidgetContent + Default
{
    pub fn default_with_rect<R: Rect>(rect: &R) -> Self {
        Widget {
            watcher: Watcher::create(WidgetInternal {
                rect: WidgetRect::from(rect),
                data: Default::default(),
            }),
        }
    }
}

impl<T: WidgetContent> Rect for Widget<T> {
    fn x(&self) -> Dim { self.internal().rect.x() }
    fn y(&self) -> Dim { self.internal().rect.y() }

    fn x_mut<F, R>(&mut self, f: F) -> R
        where F: FnOnce(&mut Dim) -> R
    {
        self.internal_mut().rect.external_view().x_mut(f)
    }

    fn y_mut<F, R>(&mut self, f: F) -> R
        where F: FnOnce(&mut Dim) -> R
    {
        self.internal_mut().rect.external_view().y_mut(f)
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct WidgetId {
    id: WatcherId,
}

impl<T: WidgetContent + 'static> From<&Widget<T>> for WidgetId {
    fn from(widget: &Widget<T>) -> Self {
        widget.id()
    }
}
