/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use crate::dims::Rect;

use super::{Content, Widget};

mod private {
    use super::Content;
    use crate::graphics::DrawContext;
    use crate::platform::RenderPlatform;
    use crate::pointer::PointerEvent;

    pub trait Widget<P> {
        fn draw(&mut self, ctx: &mut DrawContext<'_, P>)
        where
            P: RenderPlatform;
        fn pointer_event(&mut self, event: &mut PointerEvent<'_>) -> bool;
        fn pointer_event_self(&mut self, event: &mut PointerEvent<'_>)
            -> bool;
        fn as_any(self: Box<Self>) -> Box<dyn std::any::Any>;
        fn as_any_ref(&self) -> &dyn std::any::Any;
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    }

    impl<P, T> Widget<P> for super::Widget<T>
    where
        Self: 'static,
        T: Content<P>,
    {
        fn draw(&mut self, ctx: &mut DrawContext<'_, P>)
        where
            P: RenderPlatform,
        {
            super::Widget::draw(self, ctx);
        }

        fn pointer_event(&mut self, event: &mut PointerEvent<'_>) -> bool {
            super::Widget::pointer_event(self, event)
        }

        fn pointer_event_self(
            &mut self,
            event: &mut PointerEvent<'_>,
        ) -> bool {
            super::Widget::pointer_event_self(self, event)
        }

        fn as_any(self: Box<Self>) -> Box<dyn std::any::Any> {
            self
        }
        fn as_any_ref(&self) -> &dyn std::any::Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
    }
}

with_default_render_platform! {
    /// A trait which represents a Widget with an unknown content type.
    ///
    /// This can be used for the same patterns trait-objects usually are, e.g.
    /// a heterogeneous collection of Widgets.
    pub trait AnonWidget<P>
    where
        Self: Rect + private::Widget<P>,
    {
    }
}

impl<P, T> AnonWidget<P> for Widget<T>
where
    Self: 'static,
    T: Content<P>,
{
}

impl<P: 'static> dyn AnonWidget<P> {
    /// Returns the widget if its content is of type `T`.
    pub fn downcast_widget<T>(self: Box<Self>) -> Option<Widget<T>>
    where
        T: Content<P>,
    {
        self.as_any().downcast().ok().map(|w| *w)
    }

    /// Returns a reference to the widget if its content is of type `T`.
    pub fn downcast_widget_ref<T>(&self) -> Option<&Widget<T>>
    where
        T: Content<P>,
    {
        self.as_any_ref().downcast_ref()
    }

    /// Returns a mutable reference to the widget if its content is of
    /// type `T`.
    pub fn downcast_widget_mut<T>(&mut self) -> Option<&mut Widget<T>>
    where
        T: Content<P>,
    {
        self.as_any_mut().downcast_mut()
    }
}
