/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

//! This module contains all the types associated with defining custom
//! widgets.

use std::ops::{Deref, DerefMut};

use crate::{
    adapter::Adaptable,
    dims::{Dim, Rect},
    graphics::DrawContext,
    platform::{DefaultRenderPlatform, RenderPlatform},
    pointer::PointerEvent,
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
pub use content::{Content, RunAsApp};
pub use coroutine::Coroutine;
pub use desc::Desc;
pub use ephemeral::Ephemeral;
pub use graphic::WidgetGraphic;
pub use internal::WidgetExtra;
pub use rect::WidgetRect;
pub use unique_handle::{UniqueHandle, UniqueHandleId};

/// A basic structure to wrap some data and turn it into a widget.
pub struct Widget<T, P = DefaultRenderPlatform>
where
    T: ?Sized,
{
    internal: WidgetInternal<P, T>,
}

impl<T, P, Data> Adaptable<Data> for Widget<T, P>
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

impl<T, P> Default for Widget<T, P>
where
    T: Default,
{
    fn default() -> Self {
        Self {
            internal: WidgetInternal::default(),
        }
    }
}

impl<T, P> Deref for Widget<T, P>
where
    P: 'static,
    T: Content<P>,
{
    type Target = T;
    fn deref(&self) -> &T {
        &self.internal.content
    }
}

impl<T, P> DerefMut for Widget<T, P>
where
    P: 'static,
    T: Content<P>,
{
    fn deref_mut(&mut self) -> &mut T {
        &mut self.internal.content
    }
}

// Constructors
impl<T, P> Widget<T, P> {
    pub fn new(content: T) -> Self {
        Widget {
            internal: WidgetInternal {
                rect: WidgetRect::default(),
                content,
                _platform: std::marker::PhantomData,
            },
        }
    }

    pub fn new_with_rect<R>(content: T, rect: &R) -> Self
    where
        R: ?Sized + Rect,
    {
        Widget {
            internal: WidgetInternal {
                rect: WidgetRect::external_from(rect),
                content,
                _platform: std::marker::PhantomData,
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
                rect: WidgetRect::external_from(rect),
                content: T::default(),
                _platform: std::marker::PhantomData,
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
                _platform: std::marker::PhantomData,
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
                rect: WidgetRect::external_from(rect),
                content: Adaptable::from(data),
                _platform: std::marker::PhantomData,
            },
        }
    }
}

impl<P, T> Widget<T, P>
where
    P: 'static,
    T: Content<P>,
{
    pub(crate) fn pointer_event(
        this: &mut Self,
        event: &mut PointerEvent,
    ) -> bool {
        let wid_int = &mut this.internal;
        let mut extra = WidgetExtra {
            rect: &mut wid_int.rect,
        };
        let content = &mut wid_int.content;
        T::pointer_event_before(content, &mut extra, event)
            || {
                let mut handled_by_child = false;
                T::desc(PointerEventChildReceiver {
                    content,
                    event,
                    handled: &mut handled_by_child,
                });
                handled_by_child
            }
            || T::pointer_event(content, &mut extra, event)
    }

    pub(crate) fn pointer_event_self(
        this: &mut Self,
        event: &mut PointerEvent,
    ) -> bool {
        let wid_int = &mut this.internal;
        let mut extra = WidgetExtra {
            rect: &mut wid_int.rect,
        };
        T::pointer_event(&mut wid_int.content, &mut extra, event)
    }

    pub(crate) fn into_root(self) -> RootWidget<Self> {
        RootWidget { widget: self }
    }
}

impl<P, T> Widget<T, P>
where
    P: RenderPlatform,
    T: Content<P>,
{
    pub(crate) fn draw(this: &mut Self, ctx: &mut DrawContext<P>) {
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
}

impl<P, T> Rect for Widget<T, P>
where
    T: ?Sized,
{
    fn x(&self) -> Dim {
        self.internal.rect.x()
    }
    fn y(&self) -> Dim {
        self.internal.rect.y()
    }

    fn x_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Dim) -> R,
    {
        self.internal.rect.external_x_mut(f)
    }

    fn y_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Dim) -> R,
    {
        self.internal.rect.external_y_mut(f)
    }
}

pub(crate) struct RootWidget<W: ?Sized> {
    pub widget: W,
}

impl<T, P> crate::watch::Watcher<'static> for RootWidget<Widget<T, P>>
where
    P: 'static,
    T: Content<P>,
{
    fn init(mut init: impl crate::watch::WatcherInit<'static, Self>) {
        init.init_child(|this| &mut this.widget.internal);
    }
}
