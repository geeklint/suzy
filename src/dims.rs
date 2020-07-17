/// Representation of a single dimension of padding.
#[derive(Debug, Default, PartialEq, Copy, Clone)]
pub struct Padding {
    pub before: f32,
    pub after: f32,
}

impl Padding {
    /// Create a new padding struct with the provided amounts
    pub fn new(before: f32, after: f32) -> Padding {
        Padding { before, after }
    }

    /// Create a new padding struct with zero amount
    pub fn zero() -> Padding {
        Padding::new(0.0, 0.0)
    }

    /// Create a padding struct with the same amount before and after
    pub fn uniform(value: f32) -> Padding {
        Padding::new(value, value)
    }
}

pub trait Padding2d {
    /// Get the padding on the left and right
    fn x(&self) -> Padding;

    /// Get the padding on the bottom and top
    fn y(&self) -> Padding;

    /// Get mutable reference to padding on left and right
    fn x_mut(&mut self) -> &mut Padding;

    /// Get mutable reference to padding on bottom and top
    fn y_mut(&mut self) -> &mut Padding;

    /// Get the amount of top padding
    fn top(&self) -> f32 { self.y().after }

    /// Set the amount of top padding
    fn set_top(&mut self, amount: f32) { self.y_mut().after = amount }

    /// Get the amount of right padding
    fn right(&self) -> f32 { self.x().after }

    /// Set the amount of right padding
    fn set_right(&mut self, amount: f32) { self.x_mut().after = amount }

    /// Get the amount of bottom padding
    fn bottom(&self) -> f32 { self.y().before }

    /// Set the amount of bottom padding
    fn set_bottom(&mut self, amount: f32) { self.y_mut().before = amount }

    /// Get the amount of left padding
    fn left(&self) -> f32 { self.x().before }

    /// Set the amount of left padding
    fn set_left(&mut self, amount: f32) { self.x_mut().before = amount }
}

pub trait Padding2dNew
where Self: Padding2d + Sized
{
    /// Create a new 2D padding with the provided amounts
    fn new(top: f32, right: f32, bottom: f32, left: f32) -> Self;

    /// Create a new 2D padding with zero amount
    fn zero() -> Self {
        Self::uniform(0.0)
    }

    /// Create a new 2D padding with the same amount on all edges
    fn uniform(value: f32) -> Self {
        Self::new(value, value, value, value)
    }

    /// Create a new 2D padding with the same amount on left and right, and
    /// zero on top and bottom
    fn pillarbox(value: f32) -> Self {
        Self::new(0.0, value, 0.0, value)
    }

    /// Create a new 2D padding with the same amount on top and bottom, and
    /// zero on left and right
    fn letterbox(value: f32) -> Self {
        Self::new(value, 0.0, value, 0.0)
    }

    /// Create a new 2D padding with the same amount on top as on bottom, and
    /// the same amount on left as on right
    fn windowbox(y: f32, x: f32) -> Self {
        Self::new(y, x, y, x)
    }
}

/// Representation of two dimensions of padding
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct SimplePadding2d {
    x: Padding,
    y: Padding,
}

impl Padding2dNew for SimplePadding2d {
    fn new(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self {
            x: Padding::new(left, right),
            y: Padding::new(bottom, top),
        }
    }
}

impl Padding2d for SimplePadding2d {
    fn x(&self) -> Padding { self.x }
    fn y(&self) -> Padding { self.y }
    fn x_mut(&mut self) -> &mut Padding { &mut self.x }
    fn y_mut(&mut self) -> &mut Padding { &mut self.y }
}

impl<P: Padding2d> From<&P> for SimplePadding2d {
    fn from(padding: &P) -> Self {
        Self { x: padding.x(), y: padding.y() }
    }
}

/// A struct representing span of a single dimension.
///
/// The authoritative representation of a Dim is based on a "pivot" which is
/// a ratio generally from 0 (start) to 1 (end) which indicates the point
/// around which, when changing the length of the span, it will grow from.
/// 
/// # Examples
/// ```rust
/// use suzy::dims::Dim;
/// let assert_feq = |a: f32, b: f32| {
///     assert!(
///         a.is_finite(), "assert_feq: argument `a` was NaN or infinite");
///     assert!(
///         b.is_finite(), "assert_feq: argument `b` was NaN or infinite");
///     assert!(
///         (a-b).abs() < 0.0001,
///         "assert_feq: greater than threshold: {} != {}", a, b);
/// };
/// let mut span = Dim::with_length(5.0);
/// span.set_start(10.0);
/// assert_feq(span.start(), 10.0);
/// assert_feq(span.end(), 15.0);
/// // since we called set_start, changing the length will not effect the
/// // start position (the span grows outward from start)
/// span.set_length(13.0);
/// assert_feq(span.start(), 10.0);
/// assert_feq(span.end(), 23.0);
/// ```
///
/// ```rust
/// use suzy::dims::Dim;
/// # let assert_feq = |a: f32, b: f32| {
/// #     assert!(
/// #         a.is_finite(), "assert_feq: argument `a` was NaN or infinite");
/// #     assert!(
/// #         b.is_finite(), "assert_feq: argument `b` was NaN or infinite");
/// #     assert!(
/// #         (a-b).abs() < 0.0001,
/// #         "assert_feq: greater than threshold: {} != {}", a, b);
/// # };
/// let mut span = Dim::with_length(5.0);
/// span.set_end(10.0);
/// assert_feq(span.start(), 5.0);
/// assert_feq(span.end(), 10.0);
/// // since we called set_end, changing the length will not effect the
/// // end position (the span grows backward from end)
/// span.set_length(7.0);
/// assert_feq(span.start(), 3.0);
/// assert_feq(span.end(), 10.0);
/// ```
#[derive(Copy, Clone, Debug)]
pub struct Dim {
    pivot: f32,
    pos: f32,
    length: f32,
}

impl Default for Dim {
    fn default() -> Self {
        Dim::with_length(100.0)
    }
}

impl Dim {
    /// Create a Dim with a set length that starts at zero
    pub fn with_length(length: f32) -> Self {
        Dim {
            pivot: 0.5,
            pos: 0.5 * length,
            length,
        }
    }

    /// Get the pivot. A pivot of 0 indicates that the changing the length
    /// of the span will keep the start fixed and move the end. Likewise, a
    /// pivot of 1 will keep the end of the span fixed while adjusting the
    /// start to fit the length. Default: 0.5
    pub fn pivot(&self) -> f32 { self.pivot }

    /// Set the pivot. A pivot of 0 indicates that the changing the length
    /// of the span will keep the start fixed and move the end. Likewise, a
    /// pivot of 1 will keep the end of the span fixed while adjusting the
    /// start to fit the length.
    pub fn set_pivot(&mut self, value: f32) { self.pivot = value }

    /// Get the length of the span
    pub fn length(&self) -> f32 { self.length }

    /// Grow or shrink the span
    pub fn set_length(&mut self, value: f32) { self.length = value }

    /// Get the global position of the pivot of the span
    pub fn pivot_pos(&self) -> f32 { self.pos }

    /// Set the position of the pivot of the wid
    pub fn set_pivot_pos(&mut self, value: f32) { self.pos = value }

    /// Get the beginning of the span
    pub fn start(&self) -> f32 {
        let distance_before_pivot = self.pivot * self.length;
        self.pos - distance_before_pivot
    }

    /// Set the position of the beginning of the span, and set it to grow
    /// from that point
    pub fn set_start(&mut self, value: f32) {
        self.pivot = 0.0;
        self.pos = value;
    }

    /// Get the end of the span
    pub fn end(&self) -> f32 {
        let percent_after_pivot = 1.0 - self.pivot;
        let distance_after_pivot = percent_after_pivot * self.length;
        self.pos + distance_after_pivot
    }

    /// Set the position of the end of the span, and set it to grow
    /// from that point
    pub fn set_end(&mut self, value: f32) {
        self.pivot = 1.0;
        self.pos = value;
    }

    /// Get the center of the span
    pub fn center(&self) -> f32 {
        let half_length = 0.5 * self.length;
        let distance_before_pivot = self.pivot * self.length;
        let distance_to_pivot = distance_before_pivot - half_length;
        self.pos + distance_to_pivot
    }

    /// Set the position of the center of the span, and set it to grow
    /// from that point
    pub fn set_center(&mut self, value: f32) {
        self.pivot = 0.5;
        self.pos = value;
    }

    /// Calculate and set the length and position based on a start and end
    /// value. Set to grow from center.
    pub fn set_stretch(&mut self, start: f32, end: f32) {
        self.length = end - start;
        self.pivot = 0.5;
        self.pos = 0.5 * (start + end);
    }

    /// Calculate and set the length and position based on another dim and
    /// provided padding. Set to grow from center.
    pub fn set_fill(&mut self, other: Dim, padding: Padding) {
        let start = other.start() + padding.before;
        let end = other.end() - padding.after;
        self.set_stretch(start, end);
    }

    /// Check if the given value is within the span
    pub fn contains(&self, value: f32) -> bool {
        (self.start() <= value) && (self.end() >= value)
    }

    /// Check if another span is completely contained within this one
    pub fn surrounds(&self, other: Dim) -> bool {
        (self.start() <= other.start()) && (self.end() >= other.end())
    }

    /// Check if this span overlaps another one
    pub fn overlaps(&self, other: Dim) -> bool {
        other.start() < self.end() || other.end() > self.start()
    }
}

pub trait Rect {
    fn x(&self) -> Dim;
    fn y(&self) -> Dim;

    fn x_mut<F, R>(&mut self, f: F) -> R
        where F: FnOnce(&mut Dim) -> R;
    fn y_mut<F, R>(&mut self, f: F) -> R
        where F: FnOnce(&mut Dim) -> R;
    
    /// Get the left edge of the rectangle
    fn left(&self) -> f32 { self.x().start() }

    /// Set the left edge of the rect and for it to grow to the right
    fn set_left(&mut self, value: f32) { self.x_mut(|x| x.set_start(value)) }

    /// Get the right edge of the rectangle
    fn right(&self) -> f32 { self.x().end() }

    /// Set the right edge of the rect and for it to grow to the left
    fn set_right(&mut self, value: f32) { self.x_mut(|x| x.set_end(value)) }

    /// Get the bottom edge of the rectangle
    fn bottom(&self) -> f32 { self.y().start() }

    /// Set the bottom edge of the rect and for it to grow upwards
    fn set_bottom(&mut self, value: f32) {
        self.y_mut(|y| y.set_start(value));
    }

    /// Get the top edge of the rectangle
    fn top(&self) -> f32 { self.y().end() }

    /// Set the top edge of the rect and for it to grow downwards
    fn set_top(&mut self, value: f32) { self.y_mut(|y| y.set_end(value)) }

    /// Get the horizontal center of the rect
    fn center_x(&self) -> f32 { self.x().center() }

    /// Set the horizontal center of the rect and for it to grow evenly wider
    fn set_center_x(&mut self, value: f32) {
        self.x_mut(|x| x.set_center(value));
    }

    /// Get the vertical center of the rect
    fn center_y(&self) -> f32 { self.y().center() }

    /// Set the vertical center of the rect and for it to grow evenly taller
    fn set_center_y(&mut self, value: f32) {
        self.y_mut(|y| y.set_center(value));
    }

    /// Get the center of the rect
    fn center(&self) -> (f32, f32) { (self.x().center(), self.y().center()) }

    /// Set the center of the rect and for it to grow evenly outwards
    fn set_center(&mut self, value: (f32, f32)) {
        self.x_mut(|x| x.set_center(value.0));
        self.y_mut(|y| y.set_center(value.1));
    }

    /// Get the width of the rectangle
    fn width(&self) -> f32 { self.x().length }

    /// Set the width of the rectangle
    fn set_width(&mut self, value: f32) { self.x_mut(|x| x.length = value) }

    /// Get the height of the rectangle
    fn height(&self) -> f32 { self.y().length }

    /// Set the height of the rectangle
    fn set_height(&mut self, value: f32) { self.y_mut(|y| y.length = value) }

    /// Set the pivot. A pivot of (0.5, 0.5) indicates that a rect will
    /// grow from it's center, whereas a pivot of (0, 0) indicates that a
    /// rect will grow from it's bottom left corner.
    fn pivot(&self) -> (f32, f32) { (self.x().pivot(), self.y().pivot() ) }

    /// Set the pivot. A pivot of (0.5, 0.5) indicates that a rect will
    /// grow from it's center, whereas a pivot of (0, 0) indicates that a
    /// rect will grow from it's bottom left corner.
    fn set_pivot(&mut self, value: (f32, f32)) {
        self.x_mut(|x| x.set_pivot(value.0));
        self.y_mut(|y| y.set_pivot(value.1));
    }

    /// Get the global position of the pivot of the rectangle
    fn pivot_pos(&self) -> (f32, f32) {
        (self.x().pivot_pos(), self.y().pivot_pos())
    }

    /// Set the global position of the pivot of the rectangle
    fn set_pivot_pos(&mut self, value: (f32, f32)) {
        self.x_mut(|x| x.set_pivot_pos(value.0));
        self.y_mut(|y| y.set_pivot_pos(value.1));
    }

    /// Get the area of the rectangle
    fn area(&self) -> f32 { self.x().length * self.y().length }

    /// Get the aspect ratio of this rectangle (width / height)
    /// Note: this may be a non-normal number
    fn aspect(&self) -> f32 { self.x().length / self.y().length }

    /// Check if a point is inside the rectangle
    fn contains(&self, point: (f32, f32)) -> bool {
        self.x().contains(point.0) && self.y().contains(point.1)
    }

    /// Calculate the width and horizontal position of this rect based on
    /// another rect and some padding
    fn set_fill_width<R>(&mut self, other: &R, padding: Padding)
    where
        R: Rect + ?Sized,
    {
        self.x_mut(|x| x.set_fill(other.x(), padding));
    }

    /// Calculate the height and vertical position of this rect based on
    /// another rect and some padding
    fn set_fill_height<R>(&mut self, other: &R, padding: Padding)
    where
        R: Rect + ?Sized,
    {
        self.y_mut(|y| y.set_fill(other.y(), padding));
    }

    /// Calculate the size and position of this rect based on another rect
    /// and some padding
    fn set_fill<R, P>(&mut self, other: &R, padding: &P)
    where
        R: Rect + ?Sized,
        P: Padding2d + ?Sized,
    {
        self.x_mut(|x| x.set_fill(other.x(), padding.x()));
        self.y_mut(|y| y.set_fill(other.y(), padding.y()));
    }

    /// Shink one of the lengths of this rect so that the rect's aspect ratio
    /// becomes the one provided
    fn set_fit_aspect(&mut self, aspect: f32) {
        let self_aspect = self.aspect();
        if self_aspect < aspect {  // we are relatively taller
            let new = self.x().length / aspect;
            self.set_height(new);
        } else if self_aspect > aspect {  // we're relatively wider
            let new = self.y().length * aspect;
            self.set_width(new);
        }
    }

    /// Expand one of the lengths of this rect so that the rect's aspect ratio
    /// becomes the one provided
    fn set_fill_aspect(&mut self, aspect: f32) {
        let self_aspect = self.aspect();
        if self_aspect < aspect {  // we are relatively taller
            let new = self.y().length * aspect;
            self.set_width(new);
        } else if self_aspect > aspect {  // we're relatively wider
            let new = self.x().length / aspect;
            self.set_height(new)
        }
    }

    /// Check if another rect is completely contained within this one
    fn surrounds<R: Rect + ?Sized>(&self, other: &R) -> bool {
        self.x().surrounds(other.x()) && self.y().surrounds(other.y())
    }

    /// Check if this rect overlaps at all with another
    fn overlaps<R: Rect + ?Sized>(&self, other: &R) -> bool {
        self.x().overlaps(other.x()) && self.y().overlaps(other.y())
    }
}

/// A struct representing a rectangular region
#[derive(Copy, Clone, Debug, Default)]
pub struct SimpleRect {
    x: Dim,
    y: Dim,
}

impl SimpleRect {
    pub fn new(x: Dim, y: Dim) -> Self {
        Self { x, y }
    }
}

impl<'a> Rect for SimpleRect {
    fn x(&self) -> Dim { self.x }
    fn y(&self) -> Dim { self.y }

    fn x_mut<F, R>(&mut self, f: F) -> R
        where F: FnOnce(&mut Dim) -> R
    {
        (f)( &mut self.x )
    }

    fn y_mut<F, R>(&mut self, f: F) -> R
        where F: FnOnce(&mut Dim) -> R
    {
        (f)( &mut self.y )
    }
}

impl<R: Rect> From<&R> for SimpleRect {
    fn from(rect: &R) -> Self {
        Self { x: rect.x(), y: rect.y() }
    }
}

/// A rectangular region which will ignore changes to size
#[derive(Copy, Clone)]
pub struct FixedSizeRect {
    x: Dim,
    y: Dim,
}

impl FixedSizeRect {
    pub fn new(x: Dim, y: Dim) -> Self {
        Self { x, y }
    }

    pub fn default_pos(width: f32, height: f32) -> Self {
        let x = Dim::with_length(width);
        let y = Dim::with_length(height);
        Self { x, y }
    }
}

impl<'a> Rect for FixedSizeRect {
    fn x(&self) -> Dim { self.x }
    fn y(&self) -> Dim { self.y }

    fn x_mut<F, R>(&mut self, f: F) -> R
        where F: FnOnce(&mut Dim) -> R
    {
        let width = self.x.length;
        let res = (f)(&mut self.x);
        self.x.length = width;
        res
    }

    fn y_mut<F, R>(&mut self, f: F) -> R
        where F: FnOnce(&mut Dim) -> R
    {
        let height = self.y.length;
        let res = (f)(&mut self.y);
        self.y.length = height;
        res
    }
}

impl From<SimpleRect> for FixedSizeRect {
    fn from(rect: SimpleRect) -> Self {
        Self { x: rect.x, y: rect.y }
    }
}

pub trait DynRect {
    fn x(&self) -> Dim;
    fn y(&self) -> Dim;
    fn x_mut<'a>(&mut self, f: Box<dyn FnOnce(&mut Dim) + 'a>);
    fn y_mut<'a>(&mut self, f: Box<dyn FnOnce(&mut Dim) + 'a>);
 
    fn set_left(&mut self, value: f32);
    fn set_right(&mut self, value: f32);
    fn set_bottom(&mut self, value: f32);
    fn set_top(&mut self, value: f32);
    fn set_center_x(&mut self, value: f32);
    fn set_center_y(&mut self, value: f32);
    fn set_center(&mut self, value: (f32, f32));
    fn set_width(&mut self, value: f32);
    fn set_height(&mut self, value: f32);
    fn set_pivot(&mut self, value: (f32, f32));
    fn set_pivot_pos(&mut self, value: (f32, f32));
    fn set_fit_aspect(&mut self, aspect: f32);
    fn set_fill_aspect(&mut self, aspect: f32);
}

impl<T: Rect> DynRect for T {
    fn x(&self) -> Dim { Rect::x(self) }
    fn y(&self) -> Dim { Rect::y(self) }
    fn x_mut<'a>(&mut self, f: Box<dyn FnOnce(&mut Dim) + 'a>) {
        Rect::x_mut(self, |dim| f(dim))
    }
    fn y_mut<'a>(&mut self, f: Box<dyn FnOnce(&mut Dim) + 'a>) {
        Rect::y_mut(self, |dim| f(dim))
    }
 
    fn set_left(&mut self, value: f32) { Rect::set_left(self, value) }
    fn set_right(&mut self, value: f32) { Rect::set_right(self, value) }
    fn set_bottom(&mut self, value: f32) { Rect::set_bottom(self, value) }
    fn set_top(&mut self, value: f32) { Rect::set_top(self, value) }
    fn set_center_x(&mut self, value: f32) { Rect::set_center_x(self, value) }
    fn set_center_y(&mut self, value: f32) { Rect::set_center_y(self, value) }
    fn set_center(&mut self, value: (f32, f32)) { Rect::set_center(self, value) }
    fn set_width(&mut self, value: f32) { Rect::set_width(self, value) }
    fn set_height(&mut self, value: f32) { Rect::set_height(self, value) }
    fn set_pivot(&mut self, value: (f32, f32)) { Rect::set_pivot(self, value) }
    fn set_pivot_pos(&mut self, value: (f32, f32)) { Rect::set_pivot_pos(self, value) }
    fn set_fit_aspect(&mut self, aspect: f32) { Rect::set_fit_aspect(self, aspect) }
    fn set_fill_aspect(&mut self, aspect: f32) { Rect::set_fill_aspect(self, aspect) }
}

impl Rect for dyn DynRect {
    fn x(&self) -> Dim { (*self).x() }
    fn y(&self) -> Dim { (*self).y() }

    fn x_mut<F, R>(&mut self, f: F) -> R
        where F: FnOnce(&mut Dim) -> R
    {
        let res = None;
        (*self).x_mut(Box::new(|dim| {
            res = Some(f(dim));
        }));
        res.expect(
            "DynRect implementation did not call the closure passed to x_mut"
        )
    }

    fn y_mut<F, R>(&mut self, f: F) -> R
        where F: FnOnce(&mut Dim) -> R
    {
        let res = None;
        (*self).y_mut(Box::new(|dim| {
            res = Some(f(dim));
        }));
        res.expect(
            "DynRect implementation did not call the closure passed to y_mut"
        )
    }
 
    fn set_left(&mut self, value: f32) { (*self).set_left(value) }
    fn set_right(&mut self, value: f32) { (*self).set_right(value) }
    fn set_bottom(&mut self, value: f32) { (*self).set_bottom(value) }
    fn set_top(&mut self, value: f32) { (*self).set_top(value) }
    fn set_center_x(&mut self, value: f32) { (*self).set_center_x(value) }
    fn set_center_y(&mut self, value: f32) { (*self).set_center_y(value) }
    fn set_center(&mut self, value: (f32, f32)) { (*self).set_center(value) }
    fn set_width(&mut self, value: f32) { (*self).set_width(value) }
    fn set_height(&mut self, value: f32) { (*self).set_height(value) }
    fn set_pivot(&mut self, value: (f32, f32)) { (*self).set_pivot(value) }
    fn set_pivot_pos(&mut self, value: (f32, f32)) { (*self).set_pivot_pos(value) }
    fn set_fit_aspect(&mut self, aspect: f32) { (*self).set_fit_aspect(aspect) }
    fn set_fill_aspect(&mut self, aspect: f32) { (*self).set_fill_aspect(aspect) }
}
