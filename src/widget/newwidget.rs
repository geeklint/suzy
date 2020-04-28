use crate::platform::{DefaultRenderPlatform, RenderPlatform};

use super::{Widget, WidgetContent};

pub trait NewWidget<P: RenderPlatform = DefaultRenderPlatform> {
    type Content: WidgetContent<P>;

    fn as_widget(&self) -> &Widget<Self::Content, P>;
    fn as_widget_mut(&mut self) -> &mut Widget<Self::Content, P>;
}

impl<P, T> NewWidget<P> for Widget<T, P>
where
    P: RenderPlatform,
    T: WidgetContent<P>,
{
    type Content = T;
    fn as_widget(&self) -> &Widget<Self::Content, P> { self }
    fn as_widget_mut(&mut self) -> &mut Widget<Self::Content, P> { self }
}
