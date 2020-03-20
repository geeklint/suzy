use std::cell::{Ref, RefMut};

use drying_paint::{Watcher, WatcherMeta, WatcherInit, WatcherId};
pub use drying_paint::Watched;

use crate::dims::{Rect, Dim};
use crate::graphics::{Graphic, DrawContext};
use crate::pointer::PointerEvent;

pub mod children;
mod content;
mod init;
mod rect;
pub use content::WidgetContent;
pub use init::WidgetInit;
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

impl<T: WidgetContent> WatcherInit for WidgetInternal<T> {
    fn init(watcher: &mut WatcherMeta<Self>) {
        WidgetContent::init(&mut WidgetInit { watcher });
    }
}

impl<T: WidgetContent + 'static> Widget<T> {
    pub fn id(&self) -> WidgetId {
        WidgetId {
            id: self.watcher.id(),
        }
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

impl<T> Widget<T>
where T: WidgetContent + Default
{
    pub fn default_with_rect<R: Rect>(rect: &R) -> Self {
        let rect = rect.into();
        Widget {
            watcher: Watcher::create(WidgetInternal {
                rect,
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
        self.internal_mut().rect.x_mut(f)
    }

    fn y_mut<F, R>(&mut self, f: F) -> R
        where F: FnOnce(&mut Dim) -> R
    {
        self.internal_mut().rect.y_mut(f)
    }
}

#[derive(PartialEq, Eq)]
pub struct WidgetId {
    id: WatcherId,
}

impl<T: WidgetContent + 'static> From<&Widget<T>> for WidgetId {
    fn from(widget: &Widget<T>) -> Self {
        widget.id()
    }
}

pub trait NewWidget {
    type Content: WidgetContent;

    fn as_widget(&self) -> &Widget<Self::Content>;
    fn as_widget_mut(&mut self) -> &mut Widget<Self::Content>;
}

impl<T: WidgetContent> NewWidget for Widget<T> {
    type Content = T;
    fn as_widget(&self) -> &Widget<Self::Content> { self }
    fn as_widget_mut(&mut self) -> &mut Widget<Self::Content> { self }
}

trait AnonWidget {
    fn draw(&self, ctx: &mut DrawContext);
    fn pointer_event(&mut self, event: &mut PointerEvent) -> bool;
}

impl<T: WidgetContent> AnonWidget for Widget<T> {
    fn draw(&self, ctx: &mut DrawContext) {
        Widget::draw(self, ctx);
    }

    fn pointer_event(&mut self, event: &mut PointerEvent) -> bool {
        Widget::pointer_event(self, event)
    }
}

/// A proxy to a widget with an unspecified underlying data type.
#[derive(Copy, Clone)]
pub struct WidgetProxy<'a> {
    anon: &'a dyn AnonWidget,
}

/// A mutable proxy to a widget with an unspecified underlying data type.
pub struct WidgetProxyMut<'a> {
    anon: &'a mut dyn AnonWidget,
}

pub struct OwnedWidgetProxy {
    anon: Box<dyn AnonWidget>,
}

impl<T: WidgetContent + 'static> From<Widget<T>> for OwnedWidgetProxy {
    fn from(concrete: Widget<T>) -> OwnedWidgetProxy {
        OwnedWidgetProxy { anon: Box::new(concrete) }
    }
}

impl<'a> From<&'a OwnedWidgetProxy> for WidgetProxy<'a> {
    fn from(owned: &OwnedWidgetProxy) -> WidgetProxy {
        WidgetProxy { anon: &*owned.anon }
    }
}

impl<'a> From<&'a mut OwnedWidgetProxy> for WidgetProxyMut<'a> {
    fn from(owned: &mut OwnedWidgetProxy) -> WidgetProxyMut {
        WidgetProxyMut { anon: &mut *owned.anon }
    }
}
