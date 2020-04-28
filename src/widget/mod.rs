use std::cell::{Ref, RefMut};

use drying_paint::{Watcher, WatcherId};
pub use drying_paint::Watched;

use crate::dims::{Rect, Dim};
use crate::graphics::{Graphic, DrawContext};
use crate::platform::{DefaultRenderPlatform, RenderPlatform};
use crate::pointer::PointerEvent;

mod anon;
mod children;
mod content;
mod init;
mod internal;
mod newwidget;
mod rect;

use internal::WidgetInternal;
use rect::WidgetRect;

pub use anon::{OwnedWidgetProxy, WidgetProxy, WidgetProxyMut};
pub use children::{WidgetChildren, WidgetChildrenMut};
pub use content::WidgetContent;
pub use init::WidgetInit;
pub use internal::WidgetView;
pub use newwidget::NewWidget;

/// A basic structure to wrap some data and turn it into a widget.
#[derive(Default)]
pub struct Widget<T, P = DefaultRenderPlatform>
where
    P: RenderPlatform,
    T: WidgetContent<P>,
{
    watcher: Watcher<WidgetInternal<P, T>>,
}

impl<P, T> Widget<T, P>
where
    P: RenderPlatform,
    T: WidgetContent<P>,
{
    pub fn id(&self) -> WidgetId {
        WidgetId {
            id: self.watcher.id(),
        }
    }

    /// Get an anonymous reference to this widget. This is required by
    /// WidgetContent::children(), for example.
    pub fn proxy(&self) -> WidgetProxy<P> {
        WidgetProxy { anon: self }
    }

    /// Get an mutable anonymous reference to this widget. This is required
    /// by WidgetContent::children_mut(), for example.
    pub fn proxy_mut(&mut self) -> WidgetProxyMut<P> {
        WidgetProxyMut { anon: self }
    }

    fn internal(&self) -> Ref<WidgetInternal<P, T>> { self.watcher.data() }
    fn internal_mut(&mut self) -> RefMut<WidgetInternal<P, T>> {
        self.watcher.data_mut()
    }

    pub fn content(&self) -> Ref<T> {
        Ref::map(self.internal(), |w| &w.content)
    }

    pub fn content_mut(&mut self) -> RefMut<T> {
        RefMut::map(self.internal_mut(), |w| &mut w.content)
    }

    pub(crate) fn draw(&self, ctx: &mut DrawContext<P>) {
        let wid_int = self.internal();
        let content = &wid_int.content;
        content.graphic().draw(ctx);
        content.children().draw(ctx);
        content.graphic_after().draw(ctx);
    }

    pub(crate) fn pointer_event(&mut self, event: &mut PointerEvent) -> bool {
        let handled_by_child = {
            let content = &mut self.internal_mut().content;
            content.children_mut().pointer_event(event)
        };
        handled_by_child || {
            let mut view = WidgetView {
                id: self.id(),
                source: &mut self.internal_mut(),
            };
            T::pointer_event(&mut view, event)
        }
    }
}

impl<P, T> Widget<T, P>
where
    P: RenderPlatform,
    T: WidgetContent<P> + Default,
{
    pub fn default_with_rect<R: Rect>(rect: &R) -> Self {
        Widget {
            watcher: Watcher::create(WidgetInternal {
                rect: WidgetRect::from(rect),
                content: Default::default(),
                _platform: Default::default(),
            }),
        }
    }
}

impl<P, T> Rect for Widget<T, P>
where
    P: RenderPlatform,
    T: WidgetContent<P>,
{
    fn x(&self) -> Dim { self.internal().rect.x() }
    fn y(&self) -> Dim { self.internal().rect.y() }

    fn x_mut<F, R>(&mut self, f: F) -> R
        where F: FnOnce(&mut Dim) -> R
    {
        self.internal_mut().rect.external_x_mut(f)
    }

    fn y_mut<F, R>(&mut self, f: F) -> R
        where F: FnOnce(&mut Dim) -> R
    {
        self.internal_mut().rect.external_y_mut(f)
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct WidgetId {
    id: WatcherId,
}

impl<P, T> From<&Widget<T, P>> for WidgetId
where
    P: RenderPlatform,
    T: WidgetContent<P>,
{
    fn from(widget: &Widget<T, P>) -> Self {
        widget.id()
    }
}
