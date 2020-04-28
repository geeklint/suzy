use crate::graphics;
use crate::platform::RenderPlatform;
use crate::pointer::PointerEvent;

use super::{Widget, WidgetProxy, WidgetProxyMut, WidgetContent};

pub enum WidgetChildren<'a, P, A,B,C,D>
where
    P: RenderPlatform,
    A: WidgetContent<P>,
    B: WidgetContent<P>,
    C: WidgetContent<P>,
    D: WidgetContent<P>,
{
    Zero,
    One(&'a Widget<A,P>),
    Two(&'a Widget<A,P>,&'a Widget<B,P>),
    Three(&'a Widget<A,P>,&'a Widget<B,P>,&'a Widget<C,P>),
    Four(&'a Widget<A,P>,&'a Widget<B,P>,&'a Widget<C,P>,&'a Widget<D,P>),
    Uniform(Vec<&'a Widget<A,P>>),
    Varied(Vec<WidgetProxy<'a,P>>),
}


pub enum WidgetChildrenMut<'a,P, A,B,C,D>
where
    P: RenderPlatform,
    A: WidgetContent<P>,
    B: WidgetContent<P>,
    C: WidgetContent<P>,
    D: WidgetContent<P>,
{
    Zero,
    One(&'a mut Widget<A,P>),
    Two(&'a mut Widget<A,P>,&'a mut Widget<B,P>),
    Three(&'a mut Widget<A,P>,&'a mut Widget<B,P>,&'a mut Widget<C,P>),
    Four(
        &'a mut Widget<A,P>,
        &'a mut Widget<B,P>,
        &'a mut Widget<C,P>,
        &'a mut Widget<D,P>,
    ),
    Uniform(Vec<&'a mut Widget<A,P>>),
    Varied(Vec<WidgetProxyMut<'a,P>>),
}

impl<'a, P, T> From<&'a [Widget<T,P>]>
for WidgetChildren<'a, P, T,(),(),()>
where
    P: RenderPlatform,
    T: WidgetContent<P>
{
    fn from(widgets: &'a [Widget<T,P>]) -> Self {
        WidgetChildren::Uniform(widgets.iter().collect())
    }
}

impl<'a, P, T> From<&'a mut [Widget<T,P>]>
for WidgetChildrenMut<'a, P, T,(),(),()>
where
    P: RenderPlatform,
    T: WidgetContent<P>
{
    fn from(widgets: &'a mut [Widget<T, P>]) -> Self {
        WidgetChildrenMut::Uniform(widgets.iter_mut().collect())
    }
}

impl<'a, P, A,B,C,D> WidgetChildren<'a, P, A,B,C,D>
where
    P: RenderPlatform,
    A: WidgetContent<P>,
    B: WidgetContent<P>,
    C: WidgetContent<P>,
    D: WidgetContent<P>,
{
    pub(super) fn draw(self, ctx: &mut graphics::DrawContext<P>) {
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

impl<'a, P, A,B,C,D> WidgetChildrenMut<'a, P, A,B,C,D>
where
    P: RenderPlatform,
    A: WidgetContent<P>,
    B: WidgetContent<P>,
    C: WidgetContent<P>,
    D: WidgetContent<P>,
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
