/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

//! Types dealing with the dimensions of widgets.
//!
//! The coordinate system in Suzy sets the origin at the lower-left
//! of the screen, with positive X (+X) to the right, and +Y up.
//!
//! Types `Dim` and `Padding` represent single-dimension quantities of their
//! mesurements: Position+Size for `Dim`, and padding before and after for
//! `Padding`.
//!
//! `Rect` and `Padding2d` are traits which describe the 2-dimensional
//! versions of `Dim` and `Padding` respectively.
//!
//! As a trait, `Rect` is implemented by Widgets and most Graphics.
//!
//! This module provides the types `SimpleRect` and `SimplePadding2d`, which
//! are simple value types that implement their respective traits.

/// Representation of a single dimension of padding.
#[derive(Debug, Default, PartialEq, Copy, Clone)]
pub struct Padding {
    /// The amount of padding "before" the content in a dimension.
    pub before: f32,
    /// The amount of padding "after" the content in a dimension.
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

/// Representation of two dimensions of padding
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Padding2d {
    pub x: Padding,
    pub y: Padding,
}

impl Padding2d {
    /// Create a new 2D padding with the specified values.
    pub fn new(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self {
            x: Padding::new(left, right),
            y: Padding::new(bottom, top),
        }
    }

    /// Create a new 2D padding with zero amount
    pub fn zero() -> Self {
        Self::uniform(0.0)
    }

    /// Create a new 2D padding with the same amount on all edges
    pub fn uniform(value: f32) -> Self {
        Self::new(value, value, value, value)
    }

    /// Create a new 2D padding with the same amount on left and right, and
    /// zero on top and bottom
    pub fn pillarbox(value: f32) -> Self {
        Self::new(0.0, value, 0.0, value)
    }

    /// Create a new 2D padding with the same amount on top and bottom, and
    /// zero on left and right
    pub fn letterbox(value: f32) -> Self {
        Self::new(value, 0.0, value, 0.0)
    }

    /// Create a new 2D padding with the same amount on top as on bottom, and
    /// the same amount on left as on right
    pub fn windowbox(y: f32, x: f32) -> Self {
        Self::new(y, x, y, x)
    }

    pub fn top(&self) -> f32 {
        self.y.after
    }

    pub fn set_top(&mut self, amount: f32) {
        self.y.after = amount
    }

    pub fn right(&self) -> f32 {
        self.x.after
    }

    pub fn set_right(&mut self, amount: f32) {
        self.x.after = amount
    }

    pub fn bottom(&self) -> f32 {
        self.y.before
    }

    pub fn set_bottom(&mut self, amount: f32) {
        self.y.before = amount
    }

    pub fn left(&self) -> f32 {
        self.x.before
    }

    pub fn set_left(&mut self, amount: f32) {
        self.x.before = amount
    }
}

/// A struct representing span of a single dimension.
///
/// The authoritative representation of a Dim is based on three values:
/// a position, a length, and a pivot.  The position describes a point on the
/// the screen.  The pivot is a ratio, generally from 0 (start) to 1 (end),
/// which indicates how much of the length is distributed on each side of
/// the position.
///
/// For example, a pivot of 0.1 indicates that 10% of the length is 'before'
/// the position point, and 90% is after.  Changing the length will maintain
/// this distribution.
///
/// Methods like `set_start` and `set_end` update the pivot to 0 and 1
/// respectively.
///
/// # Examples
/// ```rust
/// use suzy::dims::Dim;
/// let assert_feq = |a: f32, b: f32| {
/// #     assert!(
/// #         a.is_finite(), "assert_feq: argument `a` was NaN or infinite");
/// #     assert!(
/// #         b.is_finite(), "assert_feq: argument `b` was NaN or infinite");
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
/// span.length = 13.0;
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
/// span.length = 7.0;
/// assert_feq(span.start(), 3.0);
/// assert_feq(span.end(), 10.0);
/// ```
#[derive(Copy, Clone, Debug)]
pub struct Dim {
    pub position: f32,
    pub pivot: f32,
    pub length: f32,
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
            position: 0.5 * length,
            length,
        }
    }

    /// Get the beginning of the span
    pub fn start(&self) -> f32 {
        let distance_before_pivot = self.pivot * self.length;
        self.position - distance_before_pivot
    }

    /// Set the position of the beginning of the span, and set it to grow
    /// from that point
    pub fn set_start(&mut self, value: f32) {
        self.pivot = 0.0;
        self.position = value;
    }

    /// Get the end of the span
    pub fn end(&self) -> f32 {
        let percent_after_pivot = 1.0 - self.pivot;
        let distance_after_pivot = percent_after_pivot * self.length;
        self.position + distance_after_pivot
    }

    /// Set the position of the end of the span, and set it to grow
    /// from that point
    pub fn set_end(&mut self, value: f32) {
        self.pivot = 1.0;
        self.position = value;
    }

    /// Get the center of the span
    pub fn center(&self) -> f32 {
        let half_length = 0.5 * self.length;
        let distance_before_pivot = self.pivot * self.length;
        let distance_to_pivot = half_length - distance_before_pivot;
        self.position + distance_to_pivot
    }

    /// Set the position of the center of the span, and set it to grow
    /// from that point
    pub fn set_center(&mut self, value: f32) {
        self.pivot = 0.5;
        self.position = value;
    }

    /// Calculate and set the length and position based on a start and end
    /// value.  Leaves the pivot unchanged.
    pub fn set_stretch(&mut self, start: f32, end: f32) {
        self.length = end - start;
        // version with an if-statement guarentees we set start and end exactly
        // when pivot == 0.0 and pivot == 1.0 respectively
        self.position = if self.pivot <= 0.5 {
            start + self.length * self.pivot
        } else {
            end - self.length * (1.0 - self.pivot)
        };
    }

    /// Calculate and set the length and position based on another dim and
    /// provided padding.  Leaves the pivot unchanged.
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

/// Methods associated with controlling the dimensions of a Rectangular
/// visual element.
pub trait Rect {
    /// Get the x dimension of the rectangle.
    fn x(&self) -> Dim;
    /// Get the y dimension of the rectangle.
    fn y(&self) -> Dim;

    /// Get the left edge of the rectangle
    fn left(&self) -> f32 {
        self.x().start()
    }

    /// Set the left edge of the rect and for it to grow to the right
    fn set_left(&mut self, value: f32);

    /// Get the right edge of the rectangle
    fn right(&self) -> f32 {
        self.x().end()
    }

    /// Set the right edge of the rect and for it to grow to the left
    fn set_right(&mut self, value: f32);

    /// Get the bottom edge of the rectangle
    fn bottom(&self) -> f32 {
        self.y().start()
    }

    /// Set the bottom edge of the rect and for it to grow upwards
    fn set_bottom(&mut self, value: f32);

    /// Get the top edge of the rectangle
    fn top(&self) -> f32 {
        self.y().end()
    }

    /// Set the top edge of the rect and for it to grow downwards
    fn set_top(&mut self, value: f32);

    /// Get the horizontal center of the rect
    fn center_x(&self) -> f32 {
        self.x().center()
    }

    /// Set the horizontal center of the rect and for it to grow evenly wider
    fn set_center_x(&mut self, value: f32);

    /// Get the vertical center of the rect
    fn center_y(&self) -> f32 {
        self.y().center()
    }

    /// Set the vertical center of the rect and for it to grow evenly taller
    fn set_center_y(&mut self, value: f32);

    /// Get the center of the rect
    fn center(&self) -> [f32; 2] {
        [self.x().center(), self.y().center()]
    }

    /// Set the center of the rect and for it to grow evenly outwards
    fn set_center(&mut self, value: [f32; 2]) {
        let [cx, cy] = value;
        self.set_center_x(cx);
        self.set_center_y(cy);
    }

    /// Get the width of the rectangle
    fn width(&self) -> f32 {
        self.x().length
    }

    /// Set the width of the rectangle
    fn set_width(&mut self, value: f32);

    /// Get the height of the rectangle
    fn height(&self) -> f32 {
        self.y().length
    }

    /// Set the height of the rectangle
    fn set_height(&mut self, value: f32);

    /// Set the pivot. A pivot of [0.5, 0.5] indicates that a rect will
    /// grow from it's center, whereas a pivot of [0, 0] indicates that a
    /// rect will grow from it's bottom left corner.
    fn pivot(&self) -> [f32; 2] {
        [self.x().pivot, self.y().pivot]
    }

    /// Set the pivot. A pivot of [0.5, 0.5] indicates that a rect will
    /// grow from it's center, whereas a pivot of [0, 0] indicates that a
    /// rect will grow from it's bottom left corner.
    fn set_pivot(&mut self, value: [f32; 2]);

    /// Get the global position of the pivot of the rectangle
    fn pivot_pos(&self) -> [f32; 2] {
        [self.x().position, self.y().position]
    }

    /// Set the global position of the pivot of the rectangle
    fn set_pivot_pos(&mut self, value: [f32; 2]);

    /// Get the area of the rectangle
    fn area(&self) -> f32 {
        self.width() * self.height()
    }

    /// Get the aspect ratio of this rectangle (width / height)
    /// Note: this may be a non-normal number
    fn aspect(&self) -> f32 {
        self.x().length / self.y().length
    }

    /// Check if a point is inside the rectangle
    fn contains(&self, point: [f32; 2]) -> bool {
        let [px, py] = point;
        self.x().contains(px) && self.y().contains(py)
    }

    /// Set the width and x-position based on a left and right
    /// value.  Leaves the x-pivot unchanged.
    fn set_horizontal_stretch(&mut self, left: f32, right: f32);

    /// Set the height and y-position based on a bottom and top
    /// value.  Leaves the y-pivot unchanged.
    fn set_vertical_stretch(&mut self, bottom: f32, top: f32);

    /// Shink one of the lengths of this rect so that the rect's aspect ratio
    /// becomes the one provided
    fn shrink_to_aspect(&mut self, aspect: f32) {
        let self_aspect = self.aspect();
        if self_aspect < aspect {
            // we are relatively taller
            let new = self.width() / aspect;
            self.set_height(new);
        } else if self_aspect > aspect {
            // we're relatively wider
            let new = self.height() * aspect;
            self.set_width(new);
        }
    }

    /// Expand one of the lengths of this rect so that the rect's aspect ratio
    /// becomes the one provided
    fn grow_to_aspect(&mut self, aspect: f32) {
        let self_aspect = self.aspect();
        if self_aspect < aspect {
            // we are relatively taller
            let new = self.height() * aspect;
            self.set_width(new);
        } else if self_aspect > aspect {
            // we're relatively wider
            let new = self.width() / aspect;
            self.set_height(new)
        }
    }

    /// Calculate the width and horizontal position of this rect based on
    /// another rect and some padding
    fn set_fill_width<R>(&mut self, other: &R, padding: Padding)
    where
        Self: Sized,
        R: Rect + ?Sized,
    {
        let left = other.left() + padding.before;
        let right = other.right() - padding.after;
        self.set_horizontal_stretch(left, right)
    }

    /// Calculate the height and vertical position of this rect based on
    /// another rect and some padding
    fn set_fill_height<R>(&mut self, other: &R, padding: Padding)
    where
        Self: Sized,
        R: Rect + ?Sized,
    {
        let bottom = other.bottom() + padding.before;
        let top = other.top() - padding.after;
        self.set_vertical_stretch(bottom, top)
    }

    /// Calculate the size and position of this rect based on another rect
    /// and some padding
    fn set_fill<R>(&mut self, other: &R, padding: &Padding2d)
    where
        Self: Sized,
        R: Rect + ?Sized,
    {
        self.set_fill_width(other, padding.x);
        self.set_fill_height(other, padding.y);
    }

    /// Check if another rect is completely contained within this one
    fn surrounds<R>(&self, other: &R) -> bool
    where
        Self: Sized,
        R: Rect + ?Sized,
    {
        self.x().surrounds(other.x()) && self.y().surrounds(other.y())
    }

    /// Check if this rect overlaps at all with another
    fn overlaps<R>(&self, other: &R) -> bool
    where
        Self: Sized,
        R: Rect + ?Sized,
    {
        self.x().overlaps(other.x()) && self.y().overlaps(other.y())
    }
}

macro_rules! proxy_rect_impl {
    ($shared:expr; $exclusive:expr) => {
        fn x(&self) -> crate::dims::Dim {
            $shared(self, |rect| rect.x())
        }

        fn y(&self) -> crate::dims::Dim {
            $shared(self, |rect| rect.y())
        }

        fn set_left(&mut self, value: f32) {
            $exclusive(self, |rect| rect.set_left(value))
        }

        fn set_right(&mut self, value: f32) {
            $exclusive(self, |rect| rect.set_right(value))
        }

        fn set_bottom(&mut self, value: f32) {
            $exclusive(self, |rect| rect.set_bottom(value))
        }

        fn set_top(&mut self, value: f32) {
            $exclusive(self, |rect| rect.set_top(value))
        }

        fn set_center_x(&mut self, value: f32) {
            $exclusive(self, |rect| rect.set_center_x(value))
        }

        fn set_center_y(&mut self, value: f32) {
            $exclusive(self, |rect| rect.set_center_y(value))
        }

        fn set_width(&mut self, value: f32) {
            $exclusive(self, |rect| rect.set_width(value))
        }

        fn set_height(&mut self, value: f32) {
            $exclusive(self, |rect| rect.set_height(value))
        }

        fn set_pivot(&mut self, value: [f32; 2]) {
            $exclusive(self, |rect| rect.set_pivot(value))
        }

        fn set_pivot_pos(&mut self, value: [f32; 2]) {
            $exclusive(self, |rect| rect.set_pivot_pos(value))
        }

        fn set_horizontal_stretch(&mut self, left: f32, right: f32) {
            $exclusive(self, |rect| rect.set_horizontal_stretch(left, right))
        }

        fn set_vertical_stretch(&mut self, bottom: f32, top: f32) {
            $exclusive(self, |rect| rect.set_vertical_stretch(bottom, top))
        }
    };
}

pub(crate) use proxy_rect_impl;

/// A struct representing a rectangular region
#[derive(Copy, Clone, Debug, Default)]
pub struct SimpleRect {
    x: Dim,
    y: Dim,
}

impl SimpleRect {
    /// Create a new SimpleRect with the specified dimensions.
    pub fn new(x: Dim, y: Dim) -> Self {
        Self { x, y }
    }

    /// Create a new SimpleRect with the specified sizes, positioned at the
    /// bottom left.
    pub fn with_size(width: f32, height: f32) -> Self {
        let xdim = Dim::with_length(width);
        let ydim = Dim::with_length(height);
        Self::new(xdim, ydim)
    }
}

impl Rect for SimpleRect {
    fn x(&self) -> Dim {
        self.x
    }
    fn y(&self) -> Dim {
        self.y
    }

    fn set_left(&mut self, value: f32) {
        self.x.set_start(value)
    }

    fn set_right(&mut self, value: f32) {
        self.x.set_end(value)
    }

    fn set_bottom(&mut self, value: f32) {
        self.y.set_start(value)
    }

    fn set_top(&mut self, value: f32) {
        self.y.set_end(value)
    }

    fn set_center_x(&mut self, value: f32) {
        self.x.set_center(value)
    }

    fn set_center_y(&mut self, value: f32) {
        self.y.set_center(value)
    }

    fn set_width(&mut self, value: f32) {
        self.x.length = value;
    }

    fn set_height(&mut self, value: f32) {
        self.y.length = value;
    }

    fn set_pivot(&mut self, value: [f32; 2]) {
        let [px, py] = value;
        self.x.pivot = px;
        self.y.pivot = py;
    }

    fn set_pivot_pos(&mut self, value: [f32; 2]) {
        let [px, py] = value;
        self.x.position = px;
        self.y.position = py;
    }

    fn set_horizontal_stretch(&mut self, left: f32, right: f32) {
        self.x.set_stretch(left, right)
    }

    fn set_vertical_stretch(&mut self, bottom: f32, top: f32) {
        self.y.set_stretch(bottom, top)
    }
}

impl<R: Rect> From<&R> for SimpleRect {
    fn from(rect: &R) -> Self {
        Self {
            x: rect.x(),
            y: rect.y(),
        }
    }
}
