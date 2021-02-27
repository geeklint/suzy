/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This module provides shorthand to create layout groups to organize
//! widgets in common configurations.
//!
//! Use the method `create_layout_group` provided by WidgetInit to get a
//! instance of `LayoutTypes`, which will allow you to create a variety of
//! different layout types.
//!
//! ```rust
//! use suzy::dims::Rect;
//! use suzy::widget::{
//!     WidgetContent,
//!     WidgetInit,
//!     WidgetChildReceiver,
//!     WidgetGraphicReceiver,
//! };
//! use suzy::widgets::{Button, TextContent};
//!
//! struct CustomWidget {
//!     one: Button,
//!     two: Button,
//!     three: Button,
//! }
//!
//! impl WidgetContent for CustomWidget {
//!     fn init(mut init: impl WidgetInit<Self>) {
//!         init.create_layout_group()
//!             .stack_right()
//!             .start_at(|this| this.left())
//!             .spacing(|_| 10.0)
//!             .push(|this| &mut this.one)
//!             .push(|this| &mut this.two)
//!             .push(|this| &mut this.three)
//!         ;
//!     }
//!
//!     // ...
//! #    fn children(&mut self, mut receiver: impl WidgetChildReceiver) {
//! #        receiver.child(&mut self.one);
//! #        receiver.child(&mut self.two);
//! #        receiver.child(&mut self.three);
//! #    }
//! #    fn graphics(&mut self, _receiver: impl WidgetGraphicReceiver) {
//! #    }
//! }

use std::marker::PhantomData;

use crate::dims::Rect;
use crate::platform::RenderPlatform;

use super::{WidgetContent, WidgetInit, WidgetRect};

/// A combined reference to a widget's content and it's Rect, which is passed
/// to the closures used to customize layouts.
pub struct ContentRef<'a, T: ?Sized> {
    content: &'a mut T,
    rect: &'a mut WidgetRect,
}

impl<'a, T: ?Sized> std::ops::Deref for ContentRef<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.content
    }
}

impl<'a, T: ?Sized> std::ops::DerefMut for ContentRef<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.content
    }
}

impl<'a, T: ?Sized> Rect for ContentRef<'a, T> {
    fn x(&self) -> crate::dims::Dim {
        self.rect.x()
    }
    fn y(&self) -> crate::dims::Dim {
        self.rect.y()
    }

    fn x_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut crate::dims::Dim) -> R,
    {
        self.rect.x_mut(f)
    }

    fn y_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut crate::dims::Dim) -> R,
    {
        self.rect.y_mut(f)
    }
}

/// A trait representing the current values calculated by a layout.
pub trait LayoutValue<T: ?Sized>: 'static {
    /// Get the current calculated value.
    fn value(&self, content: &mut T, rect: &mut WidgetRect) -> f32;
}

#[derive(Clone, Copy, Debug)]
struct ValueImpl<F> {
    f: F,
}

impl<F, T> LayoutValue<T> for ValueImpl<F>
where
    T: ?Sized,
    F: 'static + for<'a> Fn(&'a mut ContentRef<T>) -> f32,
{
    fn value(&self, content: &mut T, rect: &mut WidgetRect) -> f32 {
        (self.f)(&mut ContentRef { content, rect })
    }
}

/// The base type returned from WidgetInit::create_layout_group, used to
/// create a variety of types of layouts.
#[derive(Debug)]
pub struct LayoutTypes<'a, I, T, P>
where
    I: ?Sized,
    T: ?Sized,
    P: ?Sized,
{
    init: &'a mut I,
    _types: PhantomData<(&'a T, &'a P)>,
}

impl<'a, I, T, P> LayoutTypes<'a, I, T, P>
where
    I: ?Sized,
    T: ?Sized,
    P: ?Sized,
{
    pub(super) fn new(init: &'a mut I) -> Self {
        LayoutTypes {
            init,
            _types: PhantomData,
        }
    }

    fn stack<D>(
        self,
    ) -> StackLayout<
        'a,
        D,
        I,
        T,
        P,
        impl Clone + LayoutValue<T>,
        impl LayoutValue<T>,
    > {
        let spacing = ValueImpl {
            f: |_: &mut ContentRef<T>| -> f32 { 0.0 },
        };
        let prev = ValueImpl {
            f: |_: &mut ContentRef<T>| -> f32 { 0.0 },
        };
        StackLayout {
            init: self.init,
            spacing,
            prev,
            _types: PhantomData,
        }
    }

    /// Create a layout which arranges elements vertically, putting each
    /// element above the previous one.
    pub fn stack_up(
        self,
    ) -> StackLayout<
        'a,
        Up,
        I,
        T,
        P,
        impl Clone + LayoutValue<T>,
        impl LayoutValue<T>,
    > {
        self.stack()
    }

    /// Create a layout which arranges elements vertically, putting each
    /// element below the previous one.
    pub fn stack_down(
        self,
    ) -> StackLayout<
        'a,
        Down,
        I,
        T,
        P,
        impl Clone + LayoutValue<T>,
        impl LayoutValue<T>,
    > {
        self.stack()
    }

    /// Create a layout which arranges elements horizontally, putting each
    /// element to the left of the previous one.
    pub fn stack_left(
        self,
    ) -> StackLayout<
        'a,
        Left,
        I,
        T,
        P,
        impl Clone + LayoutValue<T>,
        impl LayoutValue<T>,
    > {
        self.stack()
    }

    /// Create a layout which arranges elements horizontally, putting each
    /// element to the right of the previous one.
    pub fn stack_right(
        self,
    ) -> StackLayout<
        'a,
        Right,
        I,
        T,
        P,
        impl Clone + LayoutValue<T>,
        impl LayoutValue<T>,
    > {
        self.stack()
    }
}

/// A trait representing a direction for a stack layout.
pub trait StackLayoutDirection {
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

impl StackLayoutDirection for Up {
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

impl StackLayoutDirection for Down {
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

impl StackLayoutDirection for Left {
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

impl StackLayoutDirection for Right {
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
pub struct StackLayout<
    'a,
    Direction: ?Sized,
    Init: ?Sized,
    Content: ?Sized,
    Platform: ?Sized,
    Spacing,
    Value,
> {
    init: &'a mut Init,
    spacing: Spacing,
    prev: Value,
    _types: PhantomData<(&'a Content, &'a Platform, Direction)>,
}

impl<'a, Direction, Init, Content, Platform, Spacing, Value>
    StackLayout<'a, Direction, Init, Content, Platform, Spacing, Value>
where
    Direction: ?Sized,
    Init: ?Sized,
    Content: ?Sized,
    Platform: ?Sized,
{
    /// Specify the position the layout stack should start from.
    pub fn start_at<F>(
        self,
        f: F,
    ) -> StackLayout<
        'a,
        Direction,
        Init,
        Content,
        Platform,
        Spacing,
        impl LayoutValue<Content>,
    >
    where
        F: 'static + Fn(&mut ContentRef<Content>) -> f32,
    {
        let prev = ValueImpl { f };
        StackLayout {
            spacing: self.spacing,
            init: self.init,
            prev,
            _types: PhantomData,
        }
    }

    /// Specify the spacing between elements in the layout group.
    pub fn spacing<F>(
        self,
        f: F,
    ) -> StackLayout<
        'a,
        Direction,
        Init,
        Content,
        Platform,
        impl Clone + LayoutValue<Content>,
        Value,
    >
    where
        F: 'static + Clone + Fn(&mut ContentRef<Content>) -> f32,
    {
        let spacing = ValueImpl { f };
        StackLayout {
            spacing,
            init: self.init,
            prev: self.prev,
            _types: PhantomData,
        }
    }
}

impl<'a, Direction, Init, Content, Platform, Spacing, Value>
    StackLayout<'a, Direction, Init, Content, Platform, Spacing, Value>
where
    Direction: StackLayoutDirection + ?Sized,
    Init: WidgetInit<Content, Platform> + ?Sized,
    Content: WidgetContent<Platform> + ?Sized,
    Platform: RenderPlatform + ?Sized,
    Spacing: Clone + LayoutValue<Content>,
    Value: LayoutValue<Content>,
{
    /// Add a rectangle to the layout group.
    pub fn push<F, R>(
        self,
        f: F,
    ) -> StackLayout<
        'a,
        Direction,
        Init,
        Content,
        Platform,
        Spacing,
        impl LayoutValue<Content>,
    >
    where
        F: 'static
            + Copy
            + for<'b> Fn(&'b mut ContentRef<Content>) -> &'b mut R,
        R: Rect,
    {
        let Self {
            spacing,
            init,
            prev,
            ..
        } = self;
        let spacing_clone = spacing.clone();
        init.watch(move |content, rect| {
            let start = prev.value(content, rect);
            let mut cr = ContentRef { content, rect };
            let rect = f(&mut cr);
            Direction::set_start(rect, start);
        });
        let prev = ValueImpl {
            f: move |content: &mut ContentRef<Content>| -> f32 {
                let spacing =
                    spacing_clone.value(content.content, content.rect);
                let spacing = Direction::sign(spacing);
                let rect = f(content);
                Direction::get_end(rect) + spacing
            },
        };
        StackLayout {
            spacing,
            init,
            prev,
            _types: PhantomData,
        }
    }
}
