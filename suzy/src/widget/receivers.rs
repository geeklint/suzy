/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::graphics::{Graphic, DrawContext};
use crate::platform::{DefaultRenderPlatform, RenderPlatform};
use crate::pointer::PointerEvent;

use super::{
    Widget,
    WidgetGraphic,
    WidgetContent,
    AnonWidget,
    FindWidgetCallback,
};

pub trait WidgetChildReceiver<P = DefaultRenderPlatform>
where
    P: RenderPlatform,
{
    fn child<T: WidgetContent<P>>(&mut self, child: &mut Widget<T, P>);
    fn anon_child(&mut self, child: &mut dyn AnonWidget<P>);
}

pub trait WidgetGraphicReceiver<P = DefaultRenderPlatform>
where
    P: RenderPlatform,
{
    fn graphic<T: WidgetGraphic<P>>(&mut self, graphic: &mut T);
}

pub(super) struct DrawChildReceiver<'a, 'b, P: RenderPlatform> {
    pub ctx: &'a mut DrawContext<'b, P>,
}

impl<'a, 'b, P> WidgetChildReceiver<P> for DrawChildReceiver<'a, 'b, P>
where P: RenderPlatform
{
    fn child<T: WidgetContent<P>>(&mut self, child: &mut Widget<T, P>) {
        Widget::draw(child, self.ctx);
    }

    fn anon_child(&mut self, child: &mut dyn AnonWidget<P>) {
        child.draw(self.ctx);
    }
}

pub(super) struct PointerEventChildReceiver<'a, 'b, 'c> {
    pub event: &'a mut PointerEvent<'c>,
    pub handled: &'b mut bool,
}

impl<'a, 'b, 'c, P> WidgetChildReceiver<P>
for PointerEventChildReceiver<'a, 'b, 'c>
where
    P: RenderPlatform,
{
    fn child<T: WidgetContent<P>>(&mut self, child: &mut Widget<T, P>) {
        if !*self.handled {
            *self.handled = Widget::pointer_event(child, self.event);
        }
    }

    fn anon_child(&mut self, child: &mut dyn AnonWidget<P>) {
        if !*self.handled {
            *self.handled = child.pointer_event(self.event);
        }
    }
}

pub(super) struct DrawGraphicBeforeReceiver<'a, 'b, P: RenderPlatform> {
    pub ctx: &'a mut DrawContext<'b, P>,
}

impl<'a, 'b, P> WidgetGraphicReceiver<P> for DrawGraphicBeforeReceiver<'a, 'b, P>
where
    P: RenderPlatform,
{
    fn graphic<T: WidgetGraphic<P>>(&mut self, graphic: &mut T) {
        graphic.before_children().draw(self.ctx);
    }
}

pub(super) struct DrawGraphicAfterReceiver<'a, 'b, P: RenderPlatform> {
    pub ctx: &'a mut DrawContext<'b, P>,
}

impl<'a, 'b, P> WidgetGraphicReceiver<P> for DrawGraphicAfterReceiver<'a, 'b, P>
where
    P: RenderPlatform,
{
    fn graphic<T: WidgetGraphic<P>>(&mut self, graphic: &mut T) {
        graphic.after_children().draw(self.ctx);
    }
}

pub(super) struct FindWidgetReceiver<'a, 'b, P> {
    pub id: &'a super::WidgetId,
    pub func: &'a mut FindWidgetCallback<'b, P>,
}

impl<'a, 'b, P> WidgetChildReceiver<P> for FindWidgetReceiver<'a, 'b, P>
where
    P: RenderPlatform,
{
    fn child<T: WidgetContent<P>>(&mut self, child: &mut Widget<T, P>) {
        Widget::find_widget(child, self.id, self.func);
    }

    fn anon_child(&mut self, child: &mut dyn AnonWidget<P>) {
        child.find_widget(self.id, self.func);
    }
}
