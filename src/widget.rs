use std::cell::{Ref, RefMut};

use drying_paint::{Watched, Watcher, WatcherMeta, WatcherInit};

use crate::dims::{Rect, Dim};
use crate::graphics::{Canvas, CanvasRenderer};
use crate::interaction::{InteractionReceiver, Touch};


#[derive(Default)]
pub struct WidgetRect {
    x: Watched<Dim>,
    y: Watched<Dim>,
}

impl<'a> Rect for WidgetRect {
    fn x(&self) -> Dim { *self.x }
    fn y(&self) -> Dim { *self.y }
    fn x_mut<F: FnOnce(&mut Dim)>(&mut self, f: F) { (f)( &mut self.x ) }
    fn y_mut<F: FnOnce(&mut Dim)>(&mut self, f: F) { (f)( &mut self.y ) }
}

pub struct WidgetInit<'a, T: WidgetData> {
    watcher: &'a mut WatcherMeta<WidgetInternal<T>>,
}

impl<T: WidgetData + 'static> WidgetInit<'_, T> {
    pub fn watch<F>(&mut self, func: F)
        where F: Fn(&mut T) + 'static
    {
        self.watcher.watch(move |wid_int| (func)(&mut wid_int.data));
    }

    pub fn watch_rect<F>(&mut self, func: F)
        where F: Fn(&mut T, &mut WidgetRect) + 'static
    {
        self.watcher.watch(move |wid_int| {
            (func)(&mut wid_int.data, &mut wid_int.rect);
        });
    }

    pub fn watch_draw<F>(&mut self, func: F)
        where F: Fn(&mut T, &mut Canvas) + 'static
    {
        self.watcher.watch(move |wid_int| {
            (func)(&mut wid_int.data, &mut wid_int.canvas);
        });
    }
}

pub trait WidgetData: Sized + InteractionReceiver {
    fn init(init: &mut WidgetInit<Self>);

    fn children<'a>(&'a self) -> Vec<WidgetProxy<'a>>;

    fn children_mut<'a>(&'a mut self) -> Vec<WidgetProxyMut<'a>>;
}

#[derive(Default)]
struct WidgetInternal<T: WidgetData> {
    rect: WidgetRect,
    canvas: Canvas,
    data: T,
}

impl<T: WidgetData> WatcherInit for WidgetInternal<T> {
    fn init(watcher: &mut WatcherMeta<Self>) {
        WidgetData::init(&mut WidgetInit { watcher });
    }
}

#[derive(Default)]
pub struct Widget<T: WidgetData> {
    watcher: Watcher<WidgetInternal<T>>,
}

impl<T: WidgetData> Widget<T> {
    fn internal(&self) -> Ref<WidgetInternal<T>> { self.watcher.data() }
    fn internal_mut(&mut self) -> RefMut<WidgetInternal<T>> {
        self.watcher.data_mut()
    }
    
    pub fn proxy(&self) -> WidgetProxy {
        WidgetProxy { anon: self }
    }

    pub fn proxy_mut(&mut self) -> WidgetProxyMut {
        WidgetProxyMut { anon: self }
    }
}

impl<'a, T: WidgetData> Rect for Widget<T> {
    fn x(&self) -> Dim { *self.internal().rect.x }
    fn y(&self) -> Dim { *self.internal().rect.y }

    fn x_mut<F: FnOnce(&mut Dim)>(&mut self, f: F) {
        (f)( &mut self.internal_mut().rect.x )
    }

    fn y_mut<F: FnOnce(&mut Dim)>(&mut self, f: F) {
        (f)( &mut self.internal_mut().rect.y )
    }
}

impl<T: WidgetData> InteractionReceiver for Widget<T> {
    fn on_touch_down(&mut self, touch: Touch) -> bool {
        let mut wid_int = self.internal_mut();
        InteractionReceiver::on_touch_down(&mut wid_int.data, touch)
    }

    fn on_touch_move(&mut self, touch: Touch) -> bool {
        let mut wid_int = self.internal_mut();
        InteractionReceiver::on_touch_move(&mut wid_int.data, touch)
    }

    fn on_touch_up(&mut self, touch: Touch) -> bool {
        let mut wid_int = self.internal_mut();
        InteractionReceiver::on_touch_down(&mut wid_int.data, touch)
    }
}


trait AnonWidget: InteractionReceiver {
    fn canvas(&self) -> Ref<Canvas>;
}

impl<T: WidgetData> AnonWidget for Widget<T> {
    fn canvas(&self) -> Ref<Canvas> {
        Ref::map(self.internal(), |i| &i.canvas)
    }
}

#[derive(Copy, Clone)]
pub struct WidgetProxy<'a> {
    anon: &'a dyn AnonWidget,
}

pub struct WidgetProxyMut<'a> {
    anon: &'a mut dyn AnonWidget,
}
