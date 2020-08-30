/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ops::{
    Deref,
    DerefMut,
};

use drying_paint::{Watcher, WatcherId};
pub use drying_paint::Watched;

use crate::dims::{Rect, Dim};
use crate::graphics::DrawContext;
use crate::platform::{DefaultRenderPlatform, RenderPlatform};
use crate::pointer::PointerEvent;

mod anon;
mod content;
mod graphic;
mod init;
mod internal;
mod newwidget;
mod receivers;
mod rect;

use anon::AnonWidget;
use internal::WidgetInternal;
use rect::WidgetRect;
use receivers::{
    DrawChildReceiver,
    PointerEventChildReceiver,
    DrawGraphicBeforeReceiver,
    DrawGraphicAfterReceiver,
    FindWidgetReceiver,
};

pub use anon::{
    OwnedWidgetProxy,
    WidgetProxy,
    WidgetProxyMut,
};
pub use content::WidgetContent;
pub use graphic::WidgetGraphic;
pub use init::WidgetInit;
pub use internal::WidgetExtra;
pub use newwidget::NewWidget;
pub use receivers::{
    WidgetChildReceiver,
    WidgetMutChildReceiver,
    WidgetGraphicReceiver,
};


/// A basic structure to wrap some data and turn it into a widget.
pub struct Widget<T, P = DefaultRenderPlatform>
where
    P: RenderPlatform,
    T: WidgetContent<P> + ?Sized,
{
    watcher: Watcher<WidgetInternal<P, T>>,
}

impl<T, P> Default for Widget<T, P>
where
    P: RenderPlatform,
    T: WidgetContent<P> + Default + ?Sized,
{
    fn default() -> Self {
        Self { watcher: Default::default() }
    }
}

impl<T, P> Deref for Widget<T, P>
where
    P: RenderPlatform,
    T: WidgetContent<P>,
{
    type Target = T;
    fn deref(&self) -> &T {
        &self.watcher.data().content
    }
}

impl<T, P> DerefMut for Widget<T, P>
where
    P: RenderPlatform,
    T: WidgetContent<P>,
{
    fn deref_mut(&mut self) -> &mut T {
        &mut self.watcher.data_mut().content
    }
}

impl<P, T> Widget<T, P>
where
    P: RenderPlatform,
    T: WidgetContent<P>,
{
    pub fn id(this: &Self) -> WidgetId {
        WidgetId {
            id: this.watcher.id(),
        }
    }

    /// Get an anonymous reference to this widget. This is required by
    /// WidgetContent::children(), for example.
    pub fn proxy(this: &Self) -> WidgetProxy<P> {
        WidgetProxy { anon: this }
    }

    /// Get an mutable anonymous reference to this widget. This is required
    /// by WidgetContent::children_mut(), for example.
    pub fn proxy_mut(this: &mut Self) -> WidgetProxyMut<P> {
        WidgetProxyMut { anon: this }
    }

    pub(crate) fn draw(this: &mut Self, ctx: &mut DrawContext<P>) {
        let wid_int = this.watcher.data_mut();
        let content = &mut wid_int.content;
        content.graphics(DrawGraphicBeforeReceiver { ctx });
        content.children_mut(DrawChildReceiver { ctx });
        content.graphics(DrawGraphicAfterReceiver { ctx });
    }

    pub(crate) fn find_widget<F>(this: &mut Self, id: WidgetId, func: F)
    where
        F: FnOnce(&mut dyn AnonWidget<P>)
    {
        Self::find_widget_internal(this, &id, &mut Some(func));
    }

    fn find_widget_internal(
        this: &mut Self,
        id: &WidgetId,
        func: &mut Option<impl FnOnce(&mut dyn AnonWidget<P>)>,
    ) {
        if let Some(f) = func.take() {
            if Widget::id(this) == *id {
                f(this);
            } else {
                *func = Some(f);
                let content: &mut T = this;
                content.children_mut(FindWidgetReceiver { id, func });
            }
        }
    }

    pub(crate) fn pointer_event(this: &mut Self, event: &mut PointerEvent)
        -> bool
    {
        let mut handled_by_child = false;
        {
            let content: &mut T = this;
            content.children_mut(PointerEventChildReceiver {
                event,
                handled: &mut handled_by_child,
            });
        }
        handled_by_child || Self::pointer_event_self(this, event)
    }

    pub(crate) fn pointer_event_self(this: &mut Self, event: &mut PointerEvent)
        -> bool
    {
        let id = Widget::id(this);
        let wid_int = this.watcher.data_mut();
        let mut extra = WidgetExtra {
            id,
            rect: &mut wid_int.rect,
        };
        T::pointer_event(&mut wid_int.content, &mut extra, event)
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
                rect: WidgetRect::external_from(rect),
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
    fn x(&self) -> Dim { self.watcher.data().rect.x() }
    fn y(&self) -> Dim { self.watcher.data().rect.y() }

    fn x_mut<F, R>(&mut self, f: F) -> R
        where F: FnOnce(&mut Dim) -> R
    {
        self.watcher.data_mut().rect.external_x_mut(f)
    }

    fn y_mut<F, R>(&mut self, f: F) -> R
        where F: FnOnce(&mut Dim) -> R
    {
        self.watcher.data_mut().rect.external_y_mut(f)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
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

impl From<&mut WidgetExtra<'_>> for WidgetId {
    fn from(extra: &mut WidgetExtra) -> Self {
        extra.id()
    }
}
