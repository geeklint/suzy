use super::{Widget, WidgetContent};

pub trait NewWidget {
    type Content: WidgetContent;

    fn as_widget(&self) -> &Widget<Self::Content>;
    fn as_widget_mut(&mut self) -> &mut Widget<Self::Content>;
}

impl<T: WidgetContent> NewWidget for Widget<T> {
    type Content = T;
    fn as_widget(&self) -> &Widget<Self::Content> { self }
    fn as_widget_mut(&mut self) -> &mut Widget<Self::Content> { self }
}
