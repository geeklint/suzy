/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::math::{
    Dim,
    Rect,
};
use crate::platform::{DefaultRenderPlatform, RenderPlatform};

use super::{Widget, WidgetId};
use super::WidgetContent;

mod private {
    use crate::math::DynRect;
    use crate::platform::RenderPlatform;
    use crate::graphics::DrawContext;
    use crate::pointer::PointerEvent;
    use super::{WidgetId, WidgetContent};
    use super::super::FindWidgetCallback;

    pub trait Widget<P: RenderPlatform>: DynRect {
        fn id(&self) -> WidgetId;
        fn draw(&mut self, ctx: &mut DrawContext<P>);
        fn pointer_event(&mut self, event: &mut PointerEvent) -> bool;
        fn pointer_event_self(&mut self, event: &mut PointerEvent) -> bool;
        fn find_widget(
            &mut self,
            id: &WidgetId,
            func: &mut FindWidgetCallback<P>,
        );
        fn as_any(self: Box<Self>) -> Box<dyn std::any::Any>;
        fn as_any_ref(&self) -> &dyn std::any::Any;
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    }

    impl<P, T> Widget<P> for super::Widget<T, P>
    where
        P: RenderPlatform,
        T: WidgetContent<P>,
    {
        fn id(&self) -> WidgetId {
            super::Widget::id(self)
        }

        fn draw(&mut self, ctx: &mut DrawContext<P>) {
            super::Widget::draw(self, ctx);
        }

        fn pointer_event(&mut self, event: &mut PointerEvent) -> bool {
            super::Widget::pointer_event(self, event)
        }

        fn pointer_event_self(&mut self, event: &mut PointerEvent) -> bool {
            super::Widget::pointer_event_self(self, event)
        }

        fn find_widget(
            &mut self,
            id: &WidgetId,
            func: &mut FindWidgetCallback<P>,
        ) {
            super::Widget::find_widget(self, id, func);
        }

        fn as_any(self: Box<Self>) -> Box<dyn std::any::Any> { self }
        fn as_any_ref(&self) -> &dyn std::any::Any { self }
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
    }
}


/// A trait which represents a Widget with an unknown content type.
///
/// This can be used for the same patterns trait-objects usually are, e.g.
/// a heterogeneous collection of Widgets.
pub trait AnonWidget<P: RenderPlatform = DefaultRenderPlatform>
    : private::Widget<P>
{
    fn id(&self) -> WidgetId { private::Widget::id(self) }
}

impl<P, T> AnonWidget<P> for Widget<T, P>
where
    P: RenderPlatform,
    T: WidgetContent<P>,
{
}

impl<P: RenderPlatform> dyn AnonWidget<P> {
    pub fn downcast_widget<T>(self: Box<Self>) -> Option<Widget<T, P>>
    where
        T: WidgetContent<P>,
    {
        self.as_any().downcast().ok().map(|w| *w)
    }

    pub fn downcast_widget_ref<T>(&self) -> Option<&Widget<T, P>>
    where
        T: WidgetContent<P>,
    {
        self.as_any_ref().downcast_ref()
    }

    pub fn downcast_widget_mut<T>(&mut self) -> Option<&mut Widget<T, P>>
    where
        T: WidgetContent<P>,
    {
        self.as_any_mut().downcast_mut()
    }
}

impl<P: RenderPlatform> Rect for dyn AnonWidget<P> {
    fn x(&self) -> Dim { (*self).x() }
    fn y(&self) -> Dim { (*self).y() }

    fn x_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Dim) -> R
    {
        let mut res = None;
        (*self).x_mut(Box::new(|dim| {
            res = Some(f(dim));
        }));
        res.expect(
            "DynRect implementation did not call the closure passed to x_mut"
        )
    }
    fn y_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Dim) -> R
    {
        let mut res = None;
        (*self).y_mut(Box::new(|dim| {
            res = Some(f(dim));
        }));
        res.expect(
            "DynRect implementation did not call the closure passed to y_mut"
        )
    }

    fn set_left(&mut self, value: f32) { (*self).set_left(value) }
    fn set_right(&mut self, value: f32) { (*self).set_right(value) }
    fn set_bottom(&mut self, value: f32) { (*self).set_bottom(value) }
    fn set_top(&mut self, value: f32) { (*self).set_top(value) }
    fn set_center_x(&mut self, value: f32) { (*self).set_center_x(value) }
    fn set_center_y(&mut self, value: f32) { (*self).set_center_y(value) }
    fn set_center(&mut self, value: (f32, f32)) { (*self).set_center(value) }
    fn set_width(&mut self, value: f32) { (*self).set_width(value) }
    fn set_height(&mut self, value: f32) { (*self).set_height(value) }
    fn set_pivot(&mut self, value: (f32, f32)) { (*self).set_pivot(value) }
    fn set_pivot_pos(&mut self, value: (f32, f32)) { (*self).set_pivot_pos(value) }
    fn shrink_to_aspect(&mut self, aspect: f32) { (*self).shrink_to_aspect(aspect) }
    fn grow_to_aspect(&mut self, aspect: f32) { (*self).grow_to_aspect(aspect) }
}
