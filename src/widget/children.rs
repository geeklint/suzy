use crate::graphics;
use crate::pointer::PointerEvent;

use super::{Widget, WidgetProxy, WidgetProxyMut, WidgetContent};

pub enum WidgetChildren<'a, A,B,C,D>
where
    A: WidgetContent,
    B: WidgetContent,
    C: WidgetContent,
    D: WidgetContent,
{
    Zero,
    One(&'a Widget<A>),
    Two(&'a Widget<A>,&'a Widget<B>),
    Three(&'a Widget<A>,&'a Widget<B>,&'a Widget<C>),
    Four(&'a Widget<A>,&'a Widget<B>,&'a Widget<C>,&'a Widget<D>),
    Uniform(Vec<&'a Widget<A>>),
    Varied(Vec<WidgetProxy<'a>>),
}


pub enum WidgetChildrenMut<'a, A,B,C,D>
where
    A: WidgetContent,
    B: WidgetContent,
    C: WidgetContent,
    D: WidgetContent,
{
    Zero,
    One(&'a mut Widget<A>),
    Two(&'a mut Widget<A>,&'a mut Widget<B>),
    Three(&'a mut Widget<A>,&'a mut Widget<B>,&'a mut Widget<C>),
    Four(&'a mut Widget<A>,&'a mut Widget<B>,&'a mut Widget<C>,&'a mut Widget<D>),
    Uniform(Vec<&'a mut Widget<A>>),
    Varied(Vec<WidgetProxyMut<'a>>),
}

impl<'a, T: WidgetContent> From<&'a [Widget<T>]>
for WidgetChildren<'a, T,(),(),()>
{
    fn from(widgets: &'a [Widget<T>]) -> Self {
        WidgetChildren::Uniform(widgets.iter().collect())
    }
}

impl<'a, T: WidgetContent> From<&'a mut [Widget<T>]>
for WidgetChildrenMut<'a, T,(),(),()>
{
    fn from(widgets: &'a mut [Widget<T>]) -> Self {
        WidgetChildrenMut::Uniform(widgets.iter_mut().collect())
    }
}

impl<'a, A,B,C,D> WidgetChildren<'a, A,B,C,D>
where
    A: WidgetContent,
    B: WidgetContent,
    C: WidgetContent,
    D: WidgetContent,
{
    pub(super) fn draw(self, ctx: &mut graphics::DrawContext) {
        use WidgetChildren::*;
        match self {
            Zero => (),
            One(a) => {
                a.draw(ctx);
            },
            Two(a,b) => {
                a.draw(ctx);
                b.draw(ctx);
            },
            Three(a,b,c) => {
                a.draw(ctx);
                b.draw(ctx);
                c.draw(ctx);
            },
            Four(a,b,c,d) => {
                a.draw(ctx);
                b.draw(ctx);
                c.draw(ctx);
                d.draw(ctx);
            },
            Uniform(list) => {
                for widget in list.into_iter() {
                    widget.draw(ctx);
                }
            },
            Varied(list) => {
                for proxy in list.into_iter() {
                    proxy.anon.draw(ctx);
                }
            },
        }
    }
}

impl<'a, A,B,C,D> WidgetChildrenMut<'a, A,B,C,D>
where
    A: WidgetContent,
    B: WidgetContent,
    C: WidgetContent,
    D: WidgetContent,
{
    pub(super) fn pointer_event(self, event: &mut PointerEvent) -> bool {
        use WidgetChildrenMut::*;
        match self {
            Zero => false,
            One(a) => {
                a.pointer_event(event)
            },
            Two(a,b) => {
                a.pointer_event(event)
                || b.pointer_event(event)
            },
            Three(a,b,c) => {
                a.pointer_event(event)
                || b.pointer_event(event)
                || c.pointer_event(event)
            },
            Four(a,b,c,d) => {
                a.pointer_event(event)
                || b.pointer_event(event)
                || c.pointer_event(event)
                || d.pointer_event(event)
            },
            Uniform(list) => {
                list.into_iter().any(|widget| widget.pointer_event(event))
            },
            Varied(list) => {
                list.into_iter().any(|proxy| proxy.anon.pointer_event(event))
            },
        }
    }
}
