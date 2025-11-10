/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

//! An interface for Widgets to respond to interaction.
//!
//! Widgets like Button require their content implement the trait `Selectable`
//! so that they can update their graphics in response to a change in the
//! button's state.
//!
//! The type `SelectableData` provides a simple way to select between values.
//!
//! The type `SelectableIgnored` provides a no-op implementation of
//! `Selectable`, in case a particular Widget has no need to respond to a
//! change in selection state.
//!
//! Selectable implementations are provided with an opaque struct
//! `SelectionState`.  This can be converted into a number of "versioned"
//! enums.  This pattern allows additional states to be added in the future
//! with reasonable fallbacks for backwards-compatibility.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
enum SelectionStateAll {
    #[default]
    Normal,
    Hover,
    Focus,
    Pressed,
    Active,
}

/// A selection state is an opaque type which indicates the current state
/// a selectable widget should transition to.
///
/// In order to account for potential future selection states, this type
/// primarily provides "versioned" conversions, in increasing order of
/// complexity.  Matching on a versioned type means that if a future state is
/// added, it can "fall back" to a similar state of the version the match
/// already handles.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct SelectionState(SelectionStateAll);

impl SelectionState {
    /// Normal selection state
    #[must_use]
    pub const fn normal() -> Self {
        Self(SelectionStateAll::Normal)
    }

    /// Hover selection state
    #[must_use]
    pub const fn hover() -> Self {
        Self(SelectionStateAll::Hover)
    }

    /// Focus selection state
    #[must_use]
    pub const fn focus() -> Self {
        Self(SelectionStateAll::Focus)
    }

    /// Pressed selection state
    #[must_use]
    pub const fn pressed() -> Self {
        Self(SelectionStateAll::Pressed)
    }

    /// Active selection state
    #[must_use]
    pub const fn active() -> Self {
        Self(SelectionStateAll::Active)
    }

    /// Get version 0 selection states.
    #[must_use]
    pub fn v0(self) -> SelectionStateV0 {
        match self.0 {
            SelectionStateAll::Normal => SelectionStateV0::Normal,
            SelectionStateAll::Hover => SelectionStateV0::Normal,
            SelectionStateAll::Focus => SelectionStateV0::Focus,
            SelectionStateAll::Pressed => SelectionStateV0::Focus,
            SelectionStateAll::Active => SelectionStateV0::Active,
        }
    }

    /// Get version 1 selection states.
    #[must_use]
    pub fn v1(self) -> SelectionStateV1 {
        match self.0 {
            SelectionStateAll::Normal => SelectionStateV1::Normal,
            SelectionStateAll::Hover => SelectionStateV1::Hover,
            SelectionStateAll::Focus => SelectionStateV1::Focus,
            SelectionStateAll::Pressed => SelectionStateV1::Focus,
            SelectionStateAll::Active => SelectionStateV1::Active,
        }
    }

    /// Get version 2 selection states.
    #[must_use]
    pub fn v2(self) -> SelectionStateV2 {
        match self.0 {
            SelectionStateAll::Normal => SelectionStateV2::Normal,
            SelectionStateAll::Hover => SelectionStateV2::Hover,
            SelectionStateAll::Focus => SelectionStateV2::Focus,
            SelectionStateAll::Pressed => SelectionStateV2::Pressed,
            SelectionStateAll::Active => SelectionStateV2::Active,
        }
    }

    /// Reduce the selection state to a resonable fallback assumed to be more
    /// widely implemented.
    #[must_use]
    pub fn reduce(self) -> Self {
        Self(match self.0 {
            SelectionStateAll::Normal => SelectionStateAll::Normal,
            SelectionStateAll::Hover => SelectionStateAll::Normal,
            SelectionStateAll::Focus => SelectionStateAll::Normal,
            SelectionStateAll::Pressed => SelectionStateAll::Focus,
            SelectionStateAll::Active => SelectionStateAll::Normal,
        })
    }
}

/// Version 0 selection states.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SelectionStateV0 {
    /// Normal selection state.
    Normal,

    /// Focused selection state.
    Focus,

    /// Active selection state.
    Active,
}

impl Default for SelectionStateV0 {
    fn default() -> Self {
        SelectionState::default().into()
    }
}

impl From<SelectionState> for SelectionStateV0 {
    fn from(all: SelectionState) -> Self {
        all.v0()
    }
}

/// Version 1 selection states.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SelectionStateV1 {
    /// Normal selection state.
    Normal,

    /// Hovered selection state.
    Hover,

    /// Focused selection state.
    Focus,

    /// Active selection state.
    Active,
}

impl From<SelectionState> for SelectionStateV1 {
    fn from(all: SelectionState) -> Self {
        all.v1()
    }
}

impl Default for SelectionStateV1 {
    fn default() -> Self {
        SelectionState::default().into()
    }
}

/// Version 2 selection states.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SelectionStateV2 {
    /// Normal selection state.
    Normal,

    /// Hovered selection state.
    Hover,

    /// Focused selection state.
    Focus,

    /// Pressed selection state.
    Pressed,

    /// Active selection state.
    Active,
}

impl From<SelectionState> for SelectionStateV2 {
    fn from(all: SelectionState) -> Self {
        all.v2()
    }
}

impl Default for SelectionStateV2 {
    fn default() -> Self {
        SelectionState::default().into()
    }
}

/// A trait which enables a widget to respond to changes in selection state.
pub trait Selectable {
    /// Notify the selectable element of a change in state.
    fn selection_changed(&mut self, state: SelectionState);
}

/// A type which provides a simple implementation of `Selectable` which
/// selects between a set of values.
///
/// This type dereferences to the the best instance of the value
/// corosponding to the current selection state.
#[derive(Debug, Default)]
pub struct SelectableData<T> {
    state: drying_paint::Watched<SelectionState>,
    normal: T,
    hover: Option<T>,
    focus: Option<T>,
    pressed: Option<T>,
    active: Option<T>,
}

impl<T> SelectableData<T> {
    /// Create a builder to populate the [`SelectableData`].
    ///
    /// The value provided is the only one requred: the value for the normal
    /// state.
    pub fn builder(normal: T) -> SelectableDataBuilder<T> {
        SelectableDataBuilder {
            content: Self {
                state: drying_paint::Watched::default(),
                normal,
                hover: None,
                focus: None,
                pressed: None,
                active: None,
            },
        }
    }
}

impl<T> std::ops::Deref for SelectableData<T> {
    type Target = T;
    fn deref(&self) -> &T {
        match self.state.0 {
            SelectionStateAll::Normal => &self.normal,
            SelectionStateAll::Hover => {
                self.hover.as_ref().unwrap_or(&self.normal)
            }
            SelectionStateAll::Focus => {
                self.focus.as_ref().unwrap_or(&self.normal)
            }
            SelectionStateAll::Pressed =>
            {
                #[allow(clippy::or_fun_call)]
                self.pressed
                    .as_ref()
                    .or(self.focus.as_ref())
                    .unwrap_or(&self.normal)
            }
            SelectionStateAll::Active => {
                self.active.as_ref().unwrap_or(&self.normal)
            }
        }
    }
}

impl<T> std::ops::DerefMut for SelectableData<T> {
    fn deref_mut(&mut self) -> &mut T {
        match self.state.0 {
            SelectionStateAll::Normal => &mut self.normal,
            SelectionStateAll::Hover => {
                self.hover.as_mut().unwrap_or(&mut self.normal)
            }
            SelectionStateAll::Focus => {
                self.focus.as_mut().unwrap_or(&mut self.normal)
            }
            SelectionStateAll::Pressed =>
            {
                #[allow(clippy::or_fun_call)]
                self.pressed
                    .as_mut()
                    .or(self.focus.as_mut())
                    .unwrap_or(&mut self.normal)
            }
            SelectionStateAll::Active => {
                self.active.as_mut().unwrap_or(&mut self.normal)
            }
        }
    }
}

/// A builder enables populating the selectable data.
#[derive(Debug, Default)]
pub struct SelectableDataBuilder<T> {
    content: SelectableData<T>,
}

impl<T> SelectableDataBuilder<T> {
    /// Provide a value for the hover state.
    #[must_use]
    pub fn hover(mut self, item: T) -> Self {
        self.content.hover = Some(item);
        self
    }

    /// Provide a value for the focus state.
    #[must_use]
    pub fn focus(mut self, item: T) -> Self {
        self.content.focus = Some(item);
        self
    }

    /// Provide a value for the pressed state.
    #[must_use]
    pub fn pressed(mut self, item: T) -> Self {
        self.content.pressed = Some(item);
        self
    }

    /// Provide a value for the active state.
    #[must_use]
    pub fn active(mut self, item: T) -> Self {
        self.content.active = Some(item);
        self
    }

    /// Finish providing values and create the `SelectableData`.
    pub fn build(self) -> SelectableData<T> {
        self.content
    }
}

impl<T> Selectable for SelectableData<T> {
    fn selection_changed(&mut self, state: SelectionState) {
        *self.state = state;
    }
}

/// A type which ignores changes to selection state.
///
/// This type implements `Selectable`, but does nothing in response.  It can be
/// useful if a interface (such as button) requires that a type parameter be
/// Selectable, but you do not actually care about changes to the selection
/// state.
#[derive(Clone, Copy, Debug, Default)]
pub struct SelectableIgnored<T> {
    data: T,
}

impl<T> std::ops::Deref for SelectableIgnored<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.data
    }
}

impl<T> std::ops::DerefMut for SelectableIgnored<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.data
    }
}

impl<T> Selectable for SelectableIgnored<T> {
    fn selection_changed(&mut self, _state: SelectionState) {}
}

mod extra_impls {
    use super::*;
    use crate::{
        graphics::{DrawContext, Graphic},
        platform::RenderPlatform,
        pointer::PointerEvent,
        widget::{self, WidgetRect},
    };

    impl<T, P> widget::Content<P> for SelectableIgnored<T>
    where
        T: widget::Content<P>,
    {
        fn desc(mut desc: impl widget::Desc<Self, P>) {
            desc.bare_child(|this| &mut this.data);
        }

        fn hittest(&self, rect: &WidgetRect, point: [f32; 2]) -> bool {
            self.data.hittest(rect, point)
        }

        fn pointer_event(
            &mut self,
            rect: &WidgetRect,
            event: &mut PointerEvent<'_>,
        ) -> bool {
            self.data.pointer_event(rect, event)
        }
    }

    impl<T, P> Graphic<P> for SelectableIgnored<T>
    where
        P: RenderPlatform,
        T: Graphic<P>,
    {
        fn draw(&mut self, ctx: &mut DrawContext<'_, P>) {
            self.data.draw(ctx)
        }
    }

    impl<T, P> Graphic<P> for SelectableData<T>
    where
        P: RenderPlatform,
        T: Graphic<P>,
    {
        fn draw(&mut self, ctx: &mut DrawContext<'_, P>) {
            T::draw(self, ctx)
        }
    }
}
