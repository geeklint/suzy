use std::ops::{Deref, DerefMut};

use crate::dims::{Dim, Rect};

use super::{WidgetId, WidgetContent, WidgetRect};


#[derive(Default)]
pub(super) struct WidgetInternal<T: WidgetContent> {
    pub(super) rect: WidgetRect,
    pub(super) content: T,
}

pub struct WidgetView<'a, T: WidgetContent> {
    pub(super) source: &'a mut WidgetInternal<T>,
    pub(super) id: WidgetId,
}

impl<T: WidgetContent> WidgetView<'_, T> {
    pub fn id(&self) -> WidgetId {
        self.id.clone()
    }
}

impl<T: WidgetContent> Rect for WidgetView<'_, T> {
    fn x(&self) -> Dim { self.source.rect.x() }
    fn y(&self) -> Dim { self.source.rect.y() }

    fn x_mut<F, R>(&mut self, f: F) -> R
        where F: FnOnce(&mut Dim) -> R
    {
        self.source.rect.internal_x_mut(f)
    }

    fn y_mut<F, R>(&mut self, f: F) -> R
        where F: FnOnce(&mut Dim) -> R
    {
        self.source.rect.internal_y_mut(f)
    }
}

impl<T: WidgetContent> Deref for WidgetView<'_, T> {
    type Target = T;

    fn deref(&self) -> &T { &self.source.content }
}

impl<T: WidgetContent> DerefMut for WidgetView<'_, T> {
    fn deref_mut(&mut self) -> &mut T { &mut self.source.content }
}
