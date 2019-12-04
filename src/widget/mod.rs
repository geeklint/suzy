use std::cell::{Ref, RefMut};

use drying_paint::{Watcher, WatcherMeta, WatcherInit};

use crate::dims::{Rect, Dim};
use crate::graphics::{Canvas, CanvasRenderer};

pub mod children;
mod data;
mod init;
mod rect;
pub use data::WidgetData;
pub use init::WidgetInit;
pub use rect::WidgetRect;

/// A basic structure to wrap some data and turn it into a widget.
#[derive(Default)]
pub struct Widget<T: WidgetData> {
    watcher: Watcher<WidgetInternal<T>>,
}

#[derive(Default)]
struct WidgetInternal<T: WidgetData> {
    rect: WidgetRect,
    canvas: Canvas,
    canvas_after: Canvas,
    data: T,
}

impl<T: WidgetData> WatcherInit for WidgetInternal<T> {
    fn init(watcher: &mut WatcherMeta<Self>) {
        WidgetData::init(&mut WidgetInit { watcher });
    }
}

impl<T: WidgetData> Widget<T> {
    fn internal(&self) -> Ref<WidgetInternal<T>> { self.watcher.data() }
    fn internal_mut(&mut self) -> RefMut<WidgetInternal<T>> {
        self.watcher.data_mut()
    }

    fn draw(&self, renderer: &mut CanvasRenderer) {
        let wid_int = self.internal();
        renderer.draw(&wid_int.canvas);
        wid_int.data.children().draw(renderer);
        renderer.draw(&wid_int.canvas_after);
    }
    
    /// Get an anonymous reference to this widget. This is required by
    /// WidgetData::children(), for example.
    pub fn proxy(&self) -> WidgetProxy {
        WidgetProxy { anon: self }
    }

    /// Get an mutable anonymous reference to this widget. This is required
    /// by WidgetData::children_mut(), for example.
    pub fn proxy_mut(&mut self) -> WidgetProxyMut {
        WidgetProxyMut { anon: self }
    }
}

impl<T: WidgetData> Rect for Widget<T> {
    fn x(&self) -> Dim { self.internal().rect.x() }
    fn y(&self) -> Dim { self.internal().rect.y() }

    fn x_mut<F: FnOnce(&mut Dim)>(&mut self, f: F) {
        self.internal_mut().rect.x_mut(f)
    }

    fn y_mut<F: FnOnce(&mut Dim)>(&mut self, f: F) {
        self.internal_mut().rect.y_mut(f)
    }
}

/*
impl<T: WidgetData> InteractionReceiver for Widget<T> {
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
    fn draw(&self, renderer: &mut CanvasRenderer);
}

impl<T: WidgetData> AnonWidget for Widget<T> {
    fn draw(&self, renderer: &mut CanvasRenderer) {
        Widget::draw(self, renderer);
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
