use crate::graphics;

use super::{Widget, WidgetProxy, WidgetProxyMut, WidgetData};

pub enum WidgetChildren<'a, A,B,C,D>
where A: WidgetData, B: WidgetData, C: WidgetData, D: WidgetData
{
    Zero,
    One(&'a Widget<A>),
    Two(&'a Widget<A>,&'a Widget<B>),
    Three(&'a Widget<A>,&'a Widget<B>,&'a Widget<C>),
    Four(&'a Widget<A>,&'a Widget<B>,&'a Widget<C>,&'a Widget<D>),
    Varied(Vec<WidgetProxy<'a>>),
}


pub enum WidgetChildrenMut<'a, A,B,C,D>
where A: WidgetData, B: WidgetData, C: WidgetData, D: WidgetData
{
    Zero,
    One(&'a mut Widget<A>),
    Two(&'a mut Widget<A>,&'a mut Widget<B>),
    Three(&'a mut Widget<A>,&'a mut Widget<B>,&'a mut Widget<C>),
    Four(&'a mut Widget<A>,&'a mut Widget<B>,&'a mut Widget<C>,&'a mut Widget<D>),
    Varied(Vec<WidgetProxyMut<'a>>),
}

impl<'a, A,B,C,D> WidgetChildren<'a, A,B,C,D>
where A: WidgetData, B: WidgetData, C: WidgetData, D: WidgetData
{
    pub(super) fn draw(self, renderer: &mut graphics::CanvasRenderer) {
        use WidgetChildren::*;
        match self {
            Zero => (),
            One(a) => a.draw(renderer),
            Two(a,b) => { a.draw(renderer); b.draw(renderer) },
            Three(a,b,c) => {
                a.draw(renderer);
                b.draw(renderer);
                c.draw(renderer);
            },
            Four(a,b,c,d) => {
                a.draw(renderer);
                b.draw(renderer);
                c.draw(renderer);
                d.draw(renderer);
            },
            Varied(list) => {
                for proxy in list.into_iter() {
                    proxy.anon.draw(renderer);
                }
            },
        }
    }
}
