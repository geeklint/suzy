use std::cell::{Ref, RefMut};

use drying_paint::{Watcher, WatcherMeta, WatcherInit, WatcherId};

use crate::dims::{Rect, Dim};
use crate::graphics::{Graphic, DrawContext};

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

    pub(crate) fn draw(&self, ctx: &mut DrawContext) {
        let wid_int = self.internal();
        let data = &wid_int.data;
        data.graphic().draw(ctx);
        data.children().draw(ctx);
        data.graphic_after().draw(ctx);
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

    fn x_mut<F: FnOnce(&mut Dim)>(&mut self, f: F) {
        self.internal_mut().rect.x_mut(f)
    }

    fn y_mut<F: FnOnce(&mut Dim)>(&mut self, f: F) {
        self.internal_mut().rect.y_mut(f)
    }
}

#[derive(PartialEq, Eq)]
pub struct WidgetId {
    id: WatcherId,
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

/*
impl<T: WidgetContent> InteractionReceiver for Widget<T> {
    fn on_touch_down(&mut self, touch: Touch) -> bool {
        let mut wid_int = self.internal_mut();
        for child in wid_int.data.children_mut().into_iter() {
            if InteractionReceiver::on_touch_down(child.anon, touch) {
                return true;
            }
        }
        InteractionReceiver::on_touch_down(&mut wid_int.data, touch)
    }

    fn on_touch_move(&mut self, touch: Touch) -> bool {
        let mut wid_int = self.internal_mut();
        for child in wid_int.data.children_mut().into_iter() {
            if InteractionReceiver::on_touch_move(child.anon, touch) {
                return true;
            }
        }
        InteractionReceiver::on_touch_move(&mut wid_int.data, touch)
    }

    fn on_touch_up(&mut self, touch: Touch) -> bool {
        let mut wid_int = self.internal_mut();
        for child in wid_int.data.children_mut().into_iter() {
            if InteractionReceiver::on_touch_down(child.anon, touch) {
                return true;
            }
        }
        InteractionReceiver::on_touch_down(&mut wid_int.data, touch)
    }
}
*/


trait AnonWidget {
    fn draw(&self, ctx: &mut DrawContext);
}

impl<T: WidgetContent> AnonWidget for Widget<T> {
    fn draw(&self, ctx: &mut DrawContext) {
        Widget::draw(self, ctx);
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
