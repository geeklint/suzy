/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

//! This module provides shorthand to create layout groups to organize
//! widgets in common configurations.
//!
//! Use the method `create_layout_group` provided by WidgetInit to get a
//! instance of `LayoutTypes`, which will allow you to create a variety of
//! different layout types.
//!
//! ```rust
//! use suzy::dims::Rect;
//! use suzy::widget;
//! use suzy::widgets::{Button, TextContent};
//!
//! struct CustomWidget {
//!     one: Button,
//!     two: Button,
//!     three: Button,
//! }
//!
//! impl widget::Content for CustomWidget {
//!     fn desc(mut desc: impl widget::Desc<Self>) {
//!         desc.create_layout_group()
//!             .stack_right()
//!             .start_at(|this| this.left())
//!             .spacing(|_| 10.0)
//!             .push_new_child(|this| &mut this.one)
//!             .push_new_child(|this| &mut this.two)
//!             .push_new_child(|this| &mut this.three)
//!         ;
//!     }
//! }

use std::rc::Rc;

use crate::{dims::Rect, watch::WatchedCell};

use super::{Widget, WidgetRect};

/// A combined reference to a widget's content and it's Rect, which is passed
/// to the closures used to customize layouts.
pub struct ContentWithRect<'a, T: ?Sized> {
    content: &'a T,
    rect: &'a WidgetRect,
}

impl<'a, T: ?Sized> std::ops::Deref for ContentWithRect<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.content
    }
}

impl<'a, T: ?Sized> Rect for ContentWithRect<'a, T> {
    fn x(&self) -> crate::dims::Dim {
        self.rect.x()
    }
    fn y(&self) -> crate::dims::Dim {
        self.rect.y()
    }

    fn x_mut<F, R>(&mut self, _: F) -> R
    where
        F: FnOnce(&mut crate::dims::Dim) -> R,
    {
        unreachable!("shouldn't ever have a &mut")
    }

    fn y_mut<F, R>(&mut self, _: F) -> R
    where
        F: FnOnce(&mut crate::dims::Dim) -> R,
    {
        unreachable!("shouldn't ever have a &mut")
    }
}

/// A trait representing the current values calculated by a layout.
pub trait Value<T: ?Sized, V = f32>: 'static + Clone {
    /// Get the current calculated value.
    fn value(&self, content: &mut T, rect: &WidgetRect) -> V;
}

impl<Content, V> Value<Content, V> for V
where
    V: 'static + Copy,
    Content: ?Sized,
{
    fn value(&self, _content: &mut Content, _rect: &WidgetRect) -> V {
        *self
    }
}

#[derive(Clone, Debug)]
struct ValueImpl<F> {
    f: F,
}

impl<F, V, Content> Value<Content, V> for ValueImpl<F>
where
    Content: ?Sized,
    F: 'static + Clone + Fn(&ContentWithRect<'_, Content>) -> V,
{
    fn value(&self, content: &mut Content, rect: &WidgetRect) -> V {
        (self.f)(&ContentWithRect { content, rect })
    }
}

/// The base type returned from WidgetInit::create_layout_group, used to
/// create a variety of types of layouts.
#[derive(Debug)]
pub struct LayoutTypes<'a, Desc> {
    desc: &'a mut Desc,
}

impl<'a, Desc> LayoutTypes<'a, Desc> {
    pub(super) fn new(desc: &'a mut Desc) -> Self {
        LayoutTypes { desc }
    }

    fn stack<Dir>(self) -> Stack<'a, Dir, Desc, f32, f32>
    where
        Dir: Default,
    {
        Stack {
            desc: self.desc,
            spacing: 0.0,
            cursor: 0.0,
            dir: Dir::default(),
        }
    }

    fn distribute<Dir>(self) -> Distribute<'a, Dir, Desc, (f32, f32), f32, f32>
    where
        Dir: Default,
    {
        Distribute {
            desc: self.desc,
            dir: Dir::default(),
            bounds: (0.0, 300.0),
            weight: 1.0,
            cursor: 0.0,
            total_weight: Rc::default(),
        }
    }

    /// Create a layout which arranges elements vertically, putting each
    /// element above the previous one.
    pub fn stack_up(self) -> Stack<'a, Up, Desc, f32, f32> {
        self.stack()
    }

    /// Create a layout which arranges elements vertically, putting each
    /// element below the previous one.
    pub fn stack_down(self) -> Stack<'a, Down, Desc, f32, f32> {
        self.stack()
    }

    /// Create a layout which arranges elements horizontally, putting each
    /// element to the left of the previous one.
    pub fn stack_left(self) -> Stack<'a, Left, Desc, f32, f32> {
        self.stack()
    }

    /// Create a layout which arranges elements horizontally, putting each
    /// element to the right of the previous one.
    pub fn stack_right(self) -> Stack<'a, Right, Desc, f32, f32> {
        self.stack()
    }

    pub fn distribute_up(
        self,
    ) -> Distribute<'a, Up, Desc, (f32, f32), f32, f32> {
        self.distribute()
    }

    pub fn distribute_down(
        self,
    ) -> Distribute<'a, Down, Desc, (f32, f32), f32, f32> {
        self.distribute()
    }

    pub fn distribute_left(
        self,
    ) -> Distribute<'a, Right, Desc, (f32, f32), f32, f32> {
        self.distribute()
    }

    pub fn distribute_right(
        self,
    ) -> Distribute<'a, Right, Desc, (f32, f32), f32, f32> {
        self.distribute()
    }
}

/// A trait representing the direction items can be layed out.
pub trait Direction: 'static + Copy {
    /// The sign of the direction.
    fn sign(value: f32) -> f32;

    /// Given two bounds, return which one of them is the starting point.
    fn bounds_origin(bounds: (f32, f32)) -> f32;

    /// Set the start of the rect, as understood by this direction.
    fn set_start<R: Rect>(rect: &mut R, value: f32);

    /// Set the size of the rect along this direction's axis.
    fn set_size<R: Rect>(rect: &mut R, value: f32);

    /// Get the end value of the rect, as understood by this direction.
    fn get_end<R: Rect>(rect: &R) -> f32;
}

/// A implementation of a layout direction where each element is positioned
/// above the previous.
#[derive(Clone, Copy, Debug, Default)]
pub struct Up;

impl Direction for Up {
    fn sign(value: f32) -> f32 {
        value
    }

    fn bounds_origin((a, b): (f32, f32)) -> f32 {
        a.min(b)
    }

    fn set_start<R: Rect>(rect: &mut R, value: f32) {
        rect.set_bottom(value);
    }

    fn set_size<R: Rect>(rect: &mut R, value: f32) {
        rect.set_height(value);
    }

    fn get_end<R: Rect>(rect: &R) -> f32 {
        rect.top()
    }
}

/// A implementation of a layout direction where each element is positioned
/// below the previous.
#[derive(Clone, Copy, Debug, Default)]
pub struct Down;

impl Direction for Down {
    fn sign(value: f32) -> f32 {
        -value
    }

    fn bounds_origin((a, b): (f32, f32)) -> f32 {
        a.max(b)
    }

    fn set_start<R: Rect>(rect: &mut R, value: f32) {
        rect.set_top(value);
    }

    fn set_size<R: Rect>(rect: &mut R, value: f32) {
        rect.set_height(value);
    }

    fn get_end<R: Rect>(rect: &R) -> f32 {
        rect.bottom()
    }
}

/// A implementation of a layout direction where each element is positioned
/// to the left of the previous.
#[derive(Clone, Copy, Debug, Default)]
pub struct Left;

impl Direction for Left {
    fn sign(value: f32) -> f32 {
        -value
    }

    fn bounds_origin((a, b): (f32, f32)) -> f32 {
        a.max(b)
    }

    fn set_start<R: Rect>(rect: &mut R, value: f32) {
        rect.set_right(value);
    }

    fn set_size<R: Rect>(rect: &mut R, value: f32) {
        rect.set_width(value);
    }

    fn get_end<R: Rect>(rect: &R) -> f32 {
        rect.left()
    }
}

/// A implementation of a layout direction where each element is positioned
/// to the right of the previous.
#[derive(Clone, Copy, Debug, Default)]
pub struct Right;

impl Direction for Right {
    fn sign(value: f32) -> f32 {
        value
    }

    fn bounds_origin((a, b): (f32, f32)) -> f32 {
        a.min(b)
    }

    fn set_start<R: Rect>(rect: &mut R, value: f32) {
        rect.set_left(value);
    }

    fn set_size<R: Rect>(rect: &mut R, value: f32) {
        rect.set_width(value);
    }

    fn get_end<R: Rect>(rect: &R) -> f32 {
        rect.right()
    }
}

/// A layout group which organizes elements linearly from a starting point.
///
/// This layout does not control the size of elements.
#[derive(Debug)]
pub struct Stack<'a, Dir, Desc, Spacing, Cursor> {
    desc: &'a mut Desc,
    spacing: Spacing,
    cursor: Cursor,
    dir: Dir,
}

impl<'a, Dir, Desc, Spacing, Cursor> Stack<'a, Dir, Desc, Spacing, Cursor> {
    /// Specify the position the layout stack should start from.
    #[must_use]
    pub fn start_at<Content, Platform, F>(
        self,
        value: F,
    ) -> Stack<'a, Dir, Desc, Spacing, impl Value<Content>>
    where
        Desc: super::Desc<Content, Platform>,
        Content: ?Sized,
        F: 'static + Clone + Fn(&ContentWithRect<'_, Content>) -> f32,
    {
        Stack {
            spacing: self.spacing,
            desc: self.desc,
            cursor: ValueImpl { f: value },
            dir: self.dir,
        }
    }

    /// Specify the spacing between elements in the layout group.
    #[must_use]
    pub fn spacing<Content, F>(
        self,
        value: F,
    ) -> Stack<'a, Dir, Desc, impl Value<Content>, Cursor>
    where
        Content: ?Sized,
        F: 'static + Clone + Fn(&ContentWithRect<'_, Content>) -> f32,
    {
        Stack {
            spacing: ValueImpl { f: value },
            desc: self.desc,
            cursor: self.cursor,
            dir: self.dir,
        }
    }

    /// Declare a new child and add it to the layout group.
    ///
    /// Avoids duplication between `widget::Desc::child` and a layout group's `push`.
    pub fn push_new_child<F, Child, Content, Platform>(
        self,
        child_fn: F,
    ) -> Stack<'a, Dir, Desc, Spacing, impl Value<Content>>
    where
        Dir: Direction,
        Desc: super::Desc<Content, Platform>,
        Spacing: Clone + Value<Content>,
        Cursor: Value<Content>,
        F: 'static + Clone + Fn(&mut Content) -> &mut Widget<Child>,
        Child: super::Content<Platform>,
        Content: ?Sized,
    {
        self.desc.child(child_fn.clone());
        self.push(child_fn)
    }

    /// Add a rectangle to the layout group.
    pub fn push<F, R, Content, Platform>(
        self,
        item: F,
    ) -> Stack<'a, Dir, Desc, Spacing, impl Value<Content>>
    where
        Dir: Direction,
        Desc: super::Desc<Content, Platform>,
        Spacing: Clone + Value<Content>,
        Cursor: Value<Content>,
        F: 'static + Clone + for<'b> Fn(&'b mut Content) -> &'b mut R,
        R: Rect,
        Content: ?Sized,
    {
        let Self {
            spacing,
            desc,
            cursor,
            dir,
        } = self;
        let spacing_clone = spacing.clone();
        let item_clone = item.clone();
        desc.watch(move |content, rect| {
            let start = cursor.value(content, rect);
            let item_rect = item(content);
            Dir::set_start(item_rect, start);
        });

        #[derive(Clone)]
        struct NewCursor<Spacing, Dir, End> {
            spacing: Spacing,
            _dir: Dir,
            end: End,
        }
        impl<Content, Spacing, Dir, End> Value<Content>
            for NewCursor<Spacing, Dir, End>
        where
            Content: ?Sized,
            Spacing: Value<Content>,
            Dir: Direction,
            End: 'static + Clone + Fn(&mut Content) -> f32,
        {
            fn value(&self, content: &mut Content, rect: &WidgetRect) -> f32 {
                let spacing = self.spacing.value(content, rect);
                let spacing = Dir::sign(spacing);
                (self.end)(content) + spacing
            }
        }

        fn end<Content, F>(f: F) -> F
        where
            Content: ?Sized,
            F: 'static + Clone + Fn(&mut Content) -> f32,
        {
            f
        }

        Stack {
            spacing,
            desc,
            cursor: NewCursor {
                spacing: spacing_clone,
                _dir: dir,
                end: end::<Content, _>(move |content| {
                    let rect = item_clone(content);
                    Dir::get_end(rect)
                }),
            },
            dir,
        }
    }
}

pub struct Distribute<'a, Dir, Desc, Bounds, Weight, Cursor> {
    desc: &'a mut Desc,
    dir: Dir,
    bounds: Bounds,
    weight: Weight,
    cursor: Cursor,
    total_weight: Rc<WatchedCell<f32>>,
}

impl<'a, Dir, Desc, Bounds, Weight, Cursor>
    Distribute<'a, Dir, Desc, Bounds, Weight, Cursor>
{
    #[must_use]
    pub fn between<Content, Platform, F>(
        self,
        value: F,
    ) -> Distribute<
        'a,
        Dir,
        Desc,
        impl Value<Content, (f32, f32)>,
        Weight,
        Cursor,
    >
    where
        Desc: super::Desc<Content, Platform>,
        Content: ?Sized,
        F: 'static + Clone + Fn(&ContentWithRect<'_, Content>) -> (f32, f32),
    {
        Distribute {
            desc: self.desc,
            dir: self.dir,
            bounds: ValueImpl { f: value },
            weight: self.weight,
            cursor: self.cursor,
            total_weight: self.total_weight,
        }
    }

    #[must_use]
    pub fn weight<Content, F>(
        self,
        value: F,
    ) -> Distribute<'a, Dir, Desc, Bounds, impl Value<Content>, Cursor>
    where
        Content: ?Sized,
        F: 'static + Clone + Fn(&ContentWithRect<'_, Content>) -> f32,
    {
        Distribute {
            desc: self.desc,
            dir: self.dir,
            bounds: self.bounds,
            weight: ValueImpl { f: value },
            cursor: self.cursor,
            total_weight: self.total_weight,
        }
    }

    pub fn finish<Content, Platform>(self)
    where
        Desc: super::Desc<Content, Platform>,
        Cursor: Value<Content>,
        Content: ?Sized,
    {
        let Self {
            desc,
            cursor,
            total_weight,
            ..
        } = self;
        desc.watch(move |content, rect| {
            total_weight.set(cursor.value(content, rect));
        });
    }

    /// Declare a new child and add it to the layout group.
    ///
    /// Avoids duplication between `widget::Desc::child` and a layout group's `push`.
    #[must_use]
    pub fn push_new_child<F, Child, Content, Platform>(
        self,
        item: F,
    ) -> Distribute<'a, Dir, Desc, Bounds, Weight, impl Value<Content>>
    where
        Dir: Direction,
        Desc: super::Desc<Content, Platform>,
        Bounds: Value<Content, (f32, f32)>,
        Weight: Value<Content>,
        Cursor: Value<Content>,
        F: 'static
            + Clone
            + for<'b> Fn(&'b mut Content) -> &'b mut Widget<Child>,
        Child: super::Content<Platform>,
        Content: ?Sized,
    {
        self.desc.child(item.clone());
        self.push(item)
    }

    /// Add a rectangle to the layout group.
    #[must_use]
    pub fn push<F, R, Content, Platform>(
        self,
        item: F,
    ) -> Distribute<'a, Dir, Desc, Bounds, Weight, impl Value<Content>>
    where
        Dir: Direction,
        Desc: super::Desc<Content, Platform>,
        Bounds: Value<Content, (f32, f32)>,
        Weight: Value<Content>,
        Cursor: Value<Content>,
        F: 'static + Clone + for<'b> Fn(&'b mut Content) -> &'b mut R,
        R: Rect,
        Content: ?Sized,
    {
        let Self {
            desc,
            dir,
            bounds,
            weight,
            cursor,
            total_weight,
        } = self;
        desc.watch({
            let bounds = bounds.clone();
            let weight = weight.clone();
            let cursor = cursor.clone();
            let total_weight = Rc::clone(&total_weight);
            move |content, rect| {
                let (ba, bb) = bounds.value(content, rect);
                let this_weight = weight.value(content, rect);
                let cursor_weight = cursor.value(content, rect);
                let bounds_size = (ba - bb).abs();
                let conv_weight = bounds_size / total_weight.get();
                let size = this_weight * conv_weight;
                let pos = cursor_weight * conv_weight;
                let item_rect = item(content);
                let origin = Dir::bounds_origin((ba, bb));
                Dir::set_start(item_rect, origin + Dir::sign(pos));
                Dir::set_size(item_rect, size);
            }
        });

        #[derive(Clone)]
        struct NewCursor<Cursor, Weight> {
            cursor: Cursor,
            weight: Weight,
        }
        impl<Content, Cursor, Weight> Value<Content> for NewCursor<Cursor, Weight>
        where
            Content: ?Sized,
            Cursor: Value<Content>,
            Weight: Value<Content>,
        {
            fn value(&self, content: &mut Content, rect: &WidgetRect) -> f32 {
                self.cursor.value(content, rect)
                    + self.weight.value(content, rect)
            }
        }

        let cursor = NewCursor {
            cursor,
            weight: weight.clone(),
        };

        Distribute {
            desc,
            dir,
            bounds,
            weight,
            cursor,
            total_weight,
        }
    }
}
