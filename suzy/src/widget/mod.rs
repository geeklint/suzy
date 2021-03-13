/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This module contains all the types associated with defining custom
//! widgets.

use std::ops::{Deref, DerefMut};

pub use drying_paint::Watched;
use drying_paint::{Watcher, WatcherId};

use crate::adapter::Adaptable;
use crate::dims::{Dim, Rect};
use crate::graphics::DrawContext;
use crate::platform::{DefaultRenderPlatform, RenderPlatform};
use crate::pointer::PointerEvent;

mod anon;
mod content;
mod graphic;
mod init;
mod internal;
pub mod layout;
mod receivers;
mod rect;

use internal::WidgetInternal;
use receivers::{
    DrawChildReceiver, DrawGraphicBeforeReceiver, DrawGraphicOrderedReceiver,
    DrawGraphicUnorderedReceiver, FindWidgetReceiver,
    PointerEventChildReceiver,
};

pub use anon::AnonWidget;
pub use content::WidgetContent;
pub use graphic::WidgetGraphic;
pub use init::WidgetInit;
pub use internal::WidgetExtra;
pub use receivers::{WidgetChildReceiver, WidgetGraphicReceiver};
pub use rect::WidgetRect;

pub(crate) type FindWidgetCallback<'a, P> =
    Option<Box<dyn FnOnce(&mut dyn AnonWidget<P>) + 'a>>;

/// A basic structure to wrap some data and turn it into a widget.
pub struct Widget<T, P = DefaultRenderPlatform>
where
    P: RenderPlatform + ?Sized,
    T: WidgetContent<P> + ?Sized,
{
    watcher: Watcher<WidgetInternal<P, T>>,
}

impl<T, P, Data> Adaptable<Data> for Widget<T, P>
where
    P: RenderPlatform,
    T: WidgetContent<P> + Adaptable<Data>,
{
    fn adapt(&mut self, data: &Data) {
        self.watcher.data_mut().content.adapt(data);
    }

    fn from(data: &Data) -> Self {
        Widget::create_from(data)
    }
}

impl<T, P> Default for Widget<T, P>
where
    P: RenderPlatform,
    T: WidgetContent<P> + Default + ?Sized,
{
    fn default() -> Self {
        Self {
            watcher: Default::default(),
        }
    }
}

impl<T, P> Deref for Widget<T, P>
where
    P: RenderPlatform + ?Sized,
    T: WidgetContent<P>,
{
    type Target = T;
    fn deref(&self) -> &T {
        &self.watcher.data().content
    }
}

impl<T, P> DerefMut for Widget<T, P>
where
    P: RenderPlatform + ?Sized,
    T: WidgetContent<P>,
{
    fn deref_mut(&mut self) -> &mut T {
        &mut self.watcher.data_mut().content
    }
}

impl<P, T> Widget<T, P>
where
    P: RenderPlatform + ?Sized,
    T: WidgetContent<P>,
{
    /// Create a new widget, populating it's content using the adaptable
    /// trait.
    pub fn create_from<Data>(data: &Data) -> Self
    where
        T: Adaptable<Data>,
    {
        Widget {
            watcher: Watcher::create(WidgetInternal {
                rect: WidgetRect::default(),
                content: Adaptable::from(data),
                _platform: Default::default(),
            }),
        }
    }

    /// Create a new widget, populating it's content using the adaptable
    /// trait and with a specific initial position and size
    pub fn create_with_rect<Data, R>(data: &Data, rect: &R) -> Self
    where
        T: Adaptable<Data>,
        R: Rect,
    {
        Widget {
            watcher: Watcher::create(WidgetInternal {
                rect: WidgetRect::external_from(rect),
                content: Adaptable::from(data),
                _platform: Default::default(),
            }),
        }
    }

    /// Get a value representing a unique id for this Widget.  This value may
    /// outlive the widget, and will never compare equal to a value returned
    /// by the id method of a Widget other than this one.
    pub fn id(this: &Self) -> WidgetId {
        WidgetId {
            id: this.watcher.id(),
        }
    }

    pub(crate) fn draw(this: &mut Self, ctx: &mut DrawContext<P>) {
        let wid_int = this.watcher.data_mut();
        let content = &mut wid_int.content;
        content.graphics(DrawGraphicBeforeReceiver { ctx });
        content.children(DrawChildReceiver { ctx });
        let mut num_ordered = 0;
        content.graphics(DrawGraphicUnorderedReceiver {
            ctx,
            num_ordered: &mut num_ordered,
        });
        for target in (0..num_ordered).rev() {
            content.graphics(DrawGraphicOrderedReceiver {
                ctx,
                target,
                current: 0,
            });
        }
    }

    pub(crate) fn find_widget(
        this: &mut Self,
        id: &WidgetId,
        func: &mut FindWidgetCallback<P>,
    ) {
        if let Some(f) = func.take() {
            if Widget::id(this) == *id {
                f(this);
            } else {
                *func = Some(f);
                let content: &mut T = this;
                content.children(FindWidgetReceiver { id, func });
            }
        }
    }

    pub(crate) fn pointer_event(
        this: &mut Self,
        event: &mut PointerEvent,
    ) -> bool {
        let id = Widget::id(this);
        let wid_int = this.watcher.data_mut();
        let mut extra = WidgetExtra {
            id,
            rect: &mut wid_int.rect,
        };
        T::pointer_event_before(&mut wid_int.content, &mut extra, event)
            || {
                let mut handled_by_child = false;
                wid_int.content.children(PointerEventChildReceiver {
                    event,
                    handled: &mut handled_by_child,
                });
                handled_by_child
            }
            || T::pointer_event(&mut wid_int.content, &mut extra, event)
    }

    pub(crate) fn pointer_event_self(
        this: &mut Self,
        event: &mut PointerEvent,
    ) -> bool {
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
    /// Create a Widget with a specified initial position and size
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
    P: RenderPlatform + ?Sized,
    T: WidgetContent<P>,
{
    fn x(&self) -> Dim {
        self.watcher.data().rect.x()
    }
    fn y(&self) -> Dim {
        self.watcher.data().rect.y()
    }

    fn x_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Dim) -> R,
    {
        self.watcher.data_mut().rect.external_x_mut(f)
    }

    fn y_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Dim) -> R,
    {
        self.watcher.data_mut().rect.external_y_mut(f)
    }
}

/// A unique id for a widget
///
/// This value may outlive the widget, and will never compare equal to a
/// value obtained from a different widget.
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
