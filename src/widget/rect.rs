use drying_paint::Watched;

use crate::dims::{Rect, Dim};

fn nan_dim() -> Dim {
    let mut dim = Dim::with_length(std::f32::NAN);
    dim.set_pivot(std::f32::NAN);
    dim
}

/// A version of a Dim with two perspectives, in "internal" view and an
/// "external" view.  Changes made to the external view override changes to
/// the internal view, but the internal view is used when attributes of
/// the external view are NaN
struct TwoSidedDim {
    external: Dim,
    internal: Dim,
}

impl TwoSidedDim {
    pub fn get(&self) -> Dim {
        let mut result = self.external;

        if result.pivot().is_nan() {
            result.set_pivot(self.internal.pivot());
        }
        if result.pivot_pos().is_nan() {
            result.set_pivot_pos(self.internal.pivot());
        }
        if result.length().is_nan() {
            result.set_length(self.internal.length());
        }
        result
    }

    pub fn mut_external<F, R>(&mut self, f: F) -> R
        where F: FnOnce(&mut Dim) -> R
    {
        let res = (f)( &mut self.external );
        self.internal = self.get();
        res
    }

    pub fn mut_internal<F, R>(&mut self, f: F) -> R
        where F: FnOnce(&mut Dim) -> R
    {
        let res = (f)( &mut self.internal );
        self.internal = self.get();
        res
    }
}

impl Default for TwoSidedDim {
    fn default() -> Self {
        TwoSidedDim {
            external: nan_dim(),
            internal: Default::default(),
        }
    }
}

impl From<Dim> for TwoSidedDim {
    fn from(dim: Dim) -> Self {
        TwoSidedDim {
            external: dim,
            internal: dim,
        }
    }
}

/// A version of Rect where each dimension will trigger watching functions
#[derive(Default)]
pub struct WidgetRect {
    x: Watched<TwoSidedDim>,
    y: Watched<TwoSidedDim>,
}

impl WidgetRect {
    pub fn x(&self) -> Dim { self.x.get() }
    pub fn y(&self) -> Dim { self.y.get() }

    pub fn external_view(&mut self) -> WidgetRectExternalView {
        WidgetRectExternalView(self)
    }

    pub fn internal_view(&mut self) -> WidgetRectExternalView {
        WidgetRectExternalView(self)
    }
}

pub struct WidgetRectExternalView<'a>(&'a mut WidgetRect);

impl Rect for WidgetRectExternalView<'_> {
    fn x(&self) -> Dim { self.0.x.get() }
    fn y(&self) -> Dim { self.0.y.get() }

    fn x_mut<F, R>(&mut self, f: F) -> R
        where F: FnOnce(&mut Dim) -> R
    {
        self.0.x.mut_external(f)
    }

    fn y_mut<F, R>(&mut self, f: F) -> R
        where F: FnOnce(&mut Dim) -> R
    {
        self.0.y.mut_external(f)
    }
}

pub struct WidgetRectInternalView<'a>(&'a mut WidgetRect);

impl Rect for WidgetRectInternalView<'_> {
    fn x(&self) -> Dim { self.0.x.get() }
    fn y(&self) -> Dim { self.0.y.get() }

    fn x_mut<F, R>(&mut self, f: F) -> R
        where F: FnOnce(&mut Dim) -> R
    {
        self.0.x.mut_internal(f)
    }

    fn y_mut<F, R>(&mut self, f: F) -> R
        where F: FnOnce(&mut Dim) -> R
    {
        self.0.y.mut_internal(f)
    }
}

impl<T: Rect> From<&T> for WidgetRect {
    fn from(rect: &T) -> Self {
        Self {
            x: Watched::new(rect.x().into()),
            y: Watched::new(rect.y().into()),
        }
    }
}
