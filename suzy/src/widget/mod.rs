/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

//! This module contains all the types associated with defining custom
//! widgets.

use std::ops::{Deref, DerefMut};

pub use drying_paint::Watched;
use drying_paint::Watcher;

use crate::adapter::Adaptable;
use crate::dims::{Dim, Rect};
use crate::graphics::DrawContext;
use crate::platform::{DefaultRenderPlatform, RenderPlatform};
use crate::pointer::PointerEvent;

mod anon;
mod content;
mod coroutine;
mod graphic;
mod init;
mod internal;
pub mod layout;
mod receivers;
mod rect;
mod unique_handle;

use internal::WidgetInternal;
use receivers::{
    DrawChildReceiver, DrawGraphicBeforeReceiver, DrawGraphicOrderedReceiver,
    DrawGraphicUnorderedReceiver,
    PointerEventChildReceiver,
};

pub use anon::AnonWidget;
pub use content::{Content, RunAsApp};
pub use coroutine::Coroutine;
pub use graphic::WidgetGraphic;
pub use init::WidgetInit;
pub use internal::WidgetExtra;
pub use receivers::{WidgetChildReceiver, WidgetGraphicReceiver};
pub use rect::WidgetRect;
pub use unique_handle::{UniqueHandle, UniqueHandleId};

/// A basic structure to wrap some data and turn it into a widget.
pub struct Widget<T, P = DefaultRenderPlatform>
where
    P: RenderPlatform + ?Sized,
    T: Content<P> + ?Sized,
{
    watcher: Watcher<WidgetInternal<P, T>>,
}

impl<T, P, Data> Adaptable<Data> for Widget<T, P>
where
    P: RenderPlatform,
    T: Content<P> + Adaptable<Data>,
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
    T: Content<P> + Default + ?Sized,
{
    fn default() -> Self {
        Self {
            watcher: Watcher::default(),
        }
    }
}

impl<T, P> Deref for Widget<T, P>
where
    P: RenderPlatform + ?Sized,
    T: Content<P>,
{
    type Target = T;
    fn deref(&self) -> &T {
        &self.watcher.data().content
    }
}

impl<T, P> DerefMut for Widget<T, P>
where
    P: RenderPlatform + ?Sized,
    T: Content<P>,
{
    fn deref_mut(&mut self) -> &mut T {
        &mut self.watcher.data_mut().content
    }
}

impl<P, T> Widget<T, P>
where
    P: RenderPlatform + ?Sized,
    T: Content<P>,
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
                _platform: std::marker::PhantomData,
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
                _platform: std::marker::PhantomData,
            }),
        }
    }

    pub(crate) fn draw(this: &mut Self, ctx: &mut DrawContext<P>) {
        let wid_int = this.watcher.data_mut();
        let content = &mut wid_int.content;
        T::graphics(DrawGraphicBeforeReceiver { content, ctx });
        T::children(DrawChildReceiver { content, ctx });
        let mut num_ordered = 0;
        T::graphics(DrawGraphicUnorderedReceiver {
            content,
            ctx,
            num_ordered: &mut num_ordered,
        });
        for target in (0..num_ordered).rev() {
            T::graphics(DrawGraphicOrderedReceiver {
                content,
                ctx,
                target,
                current: 0,
            });
        }
    }

    pub(crate) fn pointer_event(
        this: &mut Self,
        event: &mut PointerEvent,
    ) -> bool {
        let wid_int = this.watcher.data_mut();
        let mut extra = WidgetExtra {
            rect: &mut wid_int.rect,
        };
        let content = &mut wid_int.content;
        T::pointer_event_before(content, &mut extra, event)
            || {
                let mut handled_by_child = false;
                T::children(PointerEventChildReceiver {
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
        let wid_int = this.watcher.data_mut();
        let mut extra = WidgetExtra {
            rect: &mut wid_int.rect,
        };
        T::pointer_event(&mut wid_int.content, &mut extra, event)
    }
}

impl<P, T> Widget<T, P>
where
    P: RenderPlatform,
    T: Content<P> + Default,
{
    /// Create a Widget with a specified initial position and size
    pub fn default_with_rect<R: Rect>(rect: &R) -> Self {
        Widget {
            watcher: Watcher::create(WidgetInternal {
                rect: WidgetRect::external_from(rect),
                content: T::default(),
                _platform: std::marker::PhantomData,
            }),
        }
    }
}

impl<P, T> Rect for Widget<T, P>
where
    P: RenderPlatform + ?Sized,
    T: Content<P>,
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
