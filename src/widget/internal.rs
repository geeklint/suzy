use std::ops::{Deref, DerefMut};

use crate::dims::{Dim, Rect};
use crate::platform::RenderPlatform;

use super::{WidgetId, WidgetContent, WidgetRect};


#[derive(Default)]
pub(super) struct WidgetInternal<P, T>
where
    P: RenderPlatform,
    T: WidgetContent<P>,
{
    pub(super) rect: WidgetRect,
    pub(super) content: T,
    pub(super) _platform: std::marker::PhantomData<P>,
}

pub struct WidgetView<'a, P, T>
where
    P: RenderPlatform,
    T: WidgetContent<P>,
{
    pub(super) source: &'a mut WidgetInternal<P, T>,
    pub(super) id: WidgetId,
}

impl<P, T> WidgetView<'_, P, T>
where
    P: RenderPlatform,
    T: WidgetContent<P>,
{
    pub fn id(&self) -> WidgetId {
        self.id.clone()
    }
}

impl<P, T> Rect for WidgetView<'_, P, T>
where
    P: RenderPlatform,
    T: WidgetContent<P>,
{
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

impl<P, T> Deref for WidgetView<'_, P, T>
where
    P: RenderPlatform,
    T: WidgetContent<P>,
{
    type Target = T;

    fn deref(&self) -> &T { &self.source.content }
}

impl<P, T> DerefMut for WidgetView<'_, P, T>
where
    P: RenderPlatform,
    T: WidgetContent<P>,
{
    fn deref_mut(&mut self) -> &mut T { &mut self.source.content }
}
