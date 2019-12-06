use drying_paint::Watched;

use crate::dims::{Rect, Dim};

/// A version of Rect where each dimension will trigger watching functions
#[derive(Default)]
pub struct WidgetRect {
    x: Watched<Dim>,
    y: Watched<Dim>,
}

impl<'a> Rect for WidgetRect {
    fn x(&self) -> Dim { *self.x }
    fn y(&self) -> Dim { *self.y }
    fn x_mut<F: FnOnce(&mut Dim)>(&mut self, f: F) { (f)( &mut self.x ) }
    fn y_mut<F: FnOnce(&mut Dim)>(&mut self, f: F) { (f)( &mut self.y ) }
}

impl<T: Rect> From<&T> for WidgetRect {
    fn from(rect: &T) -> Self {
        Self {
            x: Watched::new(rect.x()),
            y: Watched::new(rect.y()),
        }
    }
}
