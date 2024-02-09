/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

//! This module contains all the types associated with defining custom
//! widgets.

use std::ops::{Deref, DerefMut};

use crate::{
    adapter::Adaptable, dims::Rect, graphics::DrawContext,
    platform::RenderPlatform, pointer::PointerEvent,
};

mod anon;
mod content;
mod coroutine;
mod desc;
mod ephemeral;
mod graphic;
mod internal;
pub mod layout;
mod receivers;
mod rect;
mod unique_handle;

use internal::WidgetInternal;
use receivers::{
    DrawChildReceiver, DrawGraphicBeforeReceiver, DrawGraphicOrderedReceiver,
    DrawGraphicUnorderedReceiver, PointerEventChildReceiver,
};

pub use anon::AnonWidget;
pub use content::Content;
#[cfg(feature = "quickstart")]
pub use content::RunAsApp;
pub use coroutine::Coroutine;
pub use desc::Desc;
pub use ephemeral::Ephemeral;
pub use graphic::WidgetGraphic;
pub use rect::WidgetRect;
pub use unique_handle::{UniqueHandle, UniqueHandleId};

/// A basic structure to wrap some data and turn it into a widget.
pub struct Widget<T>
where
    T: ?Sized,
{
    internal: WidgetInternal<T>,
}

impl<T, Data> Adaptable<Data> for Widget<T>
where
    T: Adaptable<Data>,
{
    fn adapt(&mut self, data: &Data) {
        self.internal.content.adapt(data);
    }

    fn from(data: &Data) -> Self {
        Widget::create_from(data)
    }
}

impl<T> Default for Widget<T>
where
    T: Default,
{
    fn default() -> Self {
        Self {
            internal: WidgetInternal::default(),
        }
    }
}

impl<T> Deref for Widget<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.internal.content
    }
}

impl<T> DerefMut for Widget<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.internal.content
    }
}

// Constructors
impl<T> Widget<T> {
    pub fn new(content: T) -> Self {
        Widget {
            internal: WidgetInternal {
                rect: WidgetRect::default(),
                content,
            },
        }
    }

    pub fn new_with_rect<R>(content: T, rect: &R) -> Self
    where
        R: ?Sized + Rect,
    {
        Widget {
            internal: WidgetInternal {
                rect: WidgetRect::from_rect(rect),
                content,
            },
        }
    }

    /// Create a Widget with a specified initial position and size
    pub fn default_with_rect<R>(rect: &R) -> Self
    where
        T: Default,
        R: ?Sized + Rect,
    {
        Widget {
            internal: WidgetInternal {
                rect: WidgetRect::from_rect(rect),
                content: T::default(),
            },
        }
    }

    /// Create a new widget, populating it's content using the adaptable
    /// trait.
    pub fn create_from<Data>(data: &Data) -> Self
    where
        T: Adaptable<Data>,
        Data: ?Sized,
    {
        Widget {
            internal: WidgetInternal {
                rect: WidgetRect::default(),
                content: Adaptable::from(data),
            },
        }
    }

    /// Create a new widget, populating it's content using the adaptable
    /// trait and with a specific initial position and size
    pub fn create_with_rect<Data, R>(data: &Data, rect: &R) -> Self
    where
        T: Adaptable<Data>,
        Data: ?Sized,
        R: ?Sized + Rect,
    {
        Widget {
            internal: WidgetInternal {
                rect: WidgetRect::from_rect(rect),
                content: Adaptable::from(data),
            },
        }
    }
}

impl<T: ?Sized> Widget<T> {
    pub(crate) fn init<P>(
        path: std::rc::Weak<std::cell::RefCell<Self>>,
        app: &mut crate::app::App<P>,
    ) where
        T: Content<P::Renderer>,
        P: crate::platform::Platform,
    {
        let crate::app::App {
            watch_ctx, state, ..
        } = app;
        T::desc(receivers::WidgetInitImpl {
            watch_ctx,
            state,
            path,
        })
    }

    pub(crate) fn draw<P>(this: &mut Self, ctx: &mut DrawContext<'_, P>)
    where
        T: Content<P>,
        P: RenderPlatform,
    {
        let wid_int = &mut this.internal;
        let content = &mut wid_int.content;
        T::desc(DrawGraphicBeforeReceiver { content, ctx });
        T::desc(DrawChildReceiver { content, ctx });
        let mut num_ordered = 0;
        T::desc(DrawGraphicUnorderedReceiver {
            content,
            ctx,
            num_ordered: &mut num_ordered,
        });
        for target in (0..num_ordered).rev() {
            T::desc(DrawGraphicOrderedReceiver {
                content,
                ctx,
                target,
                current: 0,
            });
        }
    }

    pub(crate) fn pointer_event<P>(
        this: &mut Self,
        event: &mut PointerEvent<'_>,
    ) -> bool
    where
        T: Content<P>,
    {
        let wid_int = &mut this.internal;
        let content = &mut wid_int.content;
        let rect = &wid_int.rect;
        T::pointer_event_before(content, rect, event)
            || {
                let mut handled_by_child = false;
                T::desc(PointerEventChildReceiver {
                    content,
                    event,
                    handled: &mut handled_by_child,
                });
                handled_by_child
            }
            || T::pointer_event(content, rect, event)
    }

    pub(crate) fn pointer_event_self<P>(
        this: &mut Self,
        event: &mut PointerEvent<'_>,
    ) -> bool
    where
        T: Content<P>,
    {
        let wid_int = &mut this.internal;
        let content = &mut wid_int.content;
        let rect = &wid_int.rect;
        T::pointer_event(content, rect, event)
    }

    fn proxy_rect<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&WidgetRect) -> R,
    {
        f(&self.internal.rect)
    }

    fn proxy_rect_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut WidgetRect) -> R,
    {
        f(&mut self.internal.rect)
    }
}

impl<T> Rect for Widget<T>
where
    T: ?Sized,
{
    crate::dims::proxy_rect_impl! {
        Self::proxy_rect; Self::proxy_rect_mut
    }
}
