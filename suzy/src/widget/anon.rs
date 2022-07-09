/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use crate::dims::{Dim, Rect};

use super::{Content, Widget};

mod private {
    use super::Content;
    use crate::dims::DynRect;
    use crate::graphics::DrawContext;
    use crate::platform::RenderPlatform;
    use crate::pointer::PointerEvent;

    pub trait Widget<P>: DynRect {
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

/// A trait which represents a Widget with an unknown content type.
///
/// This can be used for the same patterns trait-objects usually are, e.g.
/// a heterogeneous collection of Widgets.
#[cfg(feature = "platform_opengl")]
pub trait AnonWidget<P = crate::platforms::DefaultRenderPlatform>:
    private::Widget<P>
{
}

/// A trait which represents a Widget with an unknown content type.
///
/// This can be used for the same patterns trait-objects usually are, e.g.
/// a heterogeneous collection of Widgets.
#[cfg(not(feature = "platform_opengl"))]
pub trait AnonWidget<P>: private::Widget<P> {}

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

impl<P> Rect for dyn AnonWidget<P> {
    fn x(&self) -> Dim {
        (*self).x()
    }
    fn y(&self) -> Dim {
        (*self).y()
    }

    fn x_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Dim) -> R,
    {
        let mut res = None;
        (*self).x_mut(Box::new(|dim| {
            res = Some(f(dim));
        }));
        res.expect(
            "DynRect implementation did not call the closure passed to x_mut",
        )
    }
    fn y_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Dim) -> R,
    {
        let mut res = None;
        (*self).y_mut(Box::new(|dim| {
            res = Some(f(dim));
        }));
        res.expect(
            "DynRect implementation did not call the closure passed to y_mut",
        )
    }

    fn set_left(&mut self, value: f32) {
        (*self).set_left(value)
    }
    fn set_right(&mut self, value: f32) {
        (*self).set_right(value)
    }
    fn set_bottom(&mut self, value: f32) {
        (*self).set_bottom(value)
    }
    fn set_top(&mut self, value: f32) {
        (*self).set_top(value)
    }
    fn set_center_x(&mut self, value: f32) {
        (*self).set_center_x(value)
    }
    fn set_center_y(&mut self, value: f32) {
        (*self).set_center_y(value)
    }
    fn set_center(&mut self, value: (f32, f32)) {
        (*self).set_center(value)
    }
    fn set_width(&mut self, value: f32) {
        (*self).set_width(value)
    }
    fn set_height(&mut self, value: f32) {
        (*self).set_height(value)
    }
    fn set_pivot(&mut self, value: (f32, f32)) {
        (*self).set_pivot(value)
    }
    fn set_pivot_pos(&mut self, value: (f32, f32)) {
        (*self).set_pivot_pos(value)
    }
    fn shrink_to_aspect(&mut self, aspect: f32) {
        (*self).shrink_to_aspect(aspect)
    }
    fn grow_to_aspect(&mut self, aspect: f32) {
        (*self).grow_to_aspect(aspect)
    }
}
