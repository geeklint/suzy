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
//!         desc.child(|this| &mut this.one);
//!         desc.child(|this| &mut this.two);
//!         desc.child(|this| &mut this.three);
//!         desc.create_layout_group()
//!             .stack_right()
//!             .start_at(|this| this.left())
//!             .spacing(|_| 10.0)
//!             .push(|this| &mut this.one)
//!             .push(|this| &mut this.two)
//!             .push(|this| &mut this.three)
//!         ;
//!     }
//! }

use crate::dims::Rect;

use super::WidgetRect;

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
pub trait Value<T: ?Sized>: 'static + Clone {
    /// Get the current calculated value.
    fn value(&self, content: &mut T, rect: &WidgetRect) -> f32;
}

impl<Content> Value<Content> for f32
where
    Content: ?Sized,
{
    fn value(&self, _content: &mut Content, _rect: &WidgetRect) -> f32 {
        *self
    }
}

#[derive(Clone, Copy, Debug)]
struct ValueImpl<F> {
    f: F,
}

impl<F, Content> Value<Content> for ValueImpl<F>
where
    Content: ?Sized,
    F: 'static + Clone + Fn(&ContentWithRect<'_, Content>) -> f32,
{
    fn value(&self, content: &mut Content, rect: &WidgetRect) -> f32 {
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
}

/// A trait representing the direction items can be layed out.
pub trait Direction: 'static + Copy {
    /// The sign of the direction.
    fn sign(value: f32) -> f32;

    /// Set the start of the rect, as understood by this direction.
    fn set_start<R: Rect>(rect: &mut R, value: f32);

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

    fn set_start<R: Rect>(rect: &mut R, value: f32) {
        rect.set_bottom(value);
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

    fn set_start<R: Rect>(rect: &mut R, value: f32) {
        rect.set_top(value);
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

    fn set_start<R: Rect>(rect: &mut R, value: f32) {
        rect.set_right(value);
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

    fn set_start<R: Rect>(rect: &mut R, value: f32) {
        rect.set_left(value);
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
        Content: ?Sized,
        F: 'static + Copy + for<'b> Fn(&'b mut Content) -> &'b mut R,
        R: Rect,
    {
        let Self {
            spacing,
            desc,
            cursor,
            dir,
        } = self;
        let spacing_clone = spacing.clone();
        desc.watch(move |content, rect| {
            let start = cursor.value(content, rect);
            let rect = item(content);
            Dir::set_start(rect, start);
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
            End: 'static + Copy + Fn(&mut Content) -> f32,
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
            F: 'static + Copy + Fn(&mut Content) -> f32,
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
                    let rect = item(content);
                    Dir::get_end(rect)
                }),
            },
            dir,
        }
    }
}
