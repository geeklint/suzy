/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum SelectionStateAll {
    Normal,
    Hover,
    Focus,
    Pressed,
    Active,
}

impl Default for SelectionStateAll {
    fn default() -> Self { Self::Normal }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct SelectionState(SelectionStateAll);

impl SelectionState {
    pub const fn normal() -> Self { Self(SelectionStateAll::Normal) }
    pub const fn hover() -> Self { Self(SelectionStateAll::Hover) }
    pub const fn focus() -> Self { Self(SelectionStateAll::Focus) }
    pub const fn pressed() -> Self { Self(SelectionStateAll::Pressed) }
    pub const fn active() -> Self { Self(SelectionStateAll::Active) }

    pub fn v0(self) -> SelectionStateV0 {
        match self.0 {
            SelectionStateAll::Normal => SelectionStateV0::Normal,
            SelectionStateAll::Hover => SelectionStateV0::Normal,
            SelectionStateAll::Focus => SelectionStateV0::Focus,
            SelectionStateAll::Pressed => SelectionStateV0::Focus,
            SelectionStateAll::Active => SelectionStateV0::Active,
        }
    }
    pub fn v1(self) -> SelectionStateV1 {
        match self.0 {
            SelectionStateAll::Normal => SelectionStateV1::Normal,
            SelectionStateAll::Hover => SelectionStateV1::Hover,
            SelectionStateAll::Focus => SelectionStateV1::Focus,
            SelectionStateAll::Pressed => SelectionStateV1::Focus,
            SelectionStateAll::Active => SelectionStateV1::Active,
        }
    }

    pub fn v2(self) -> SelectionStateV2 {
        match self.0 {
            SelectionStateAll::Normal => SelectionStateV2::Normal,
            SelectionStateAll::Hover => SelectionStateV2::Hover,
            SelectionStateAll::Focus => SelectionStateV2::Focus,
            SelectionStateAll::Pressed => SelectionStateV2::Pressed,
            SelectionStateAll::Active => SelectionStateV2::Active,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SelectionStateV0 {
    Normal,
    Focus,
    Active,
}

impl Default for SelectionStateV0 {
    fn default() -> Self { SelectionState::default().into() }
}

impl From<SelectionState> for SelectionStateV0 {
    fn from(all: SelectionState) -> Self { all.v0() }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SelectionStateV1 {
    Normal,
    Hover,
    Focus,
    Active,
}

impl From<SelectionState> for SelectionStateV1 {
    fn from(all: SelectionState) -> Self { all.v1() }
}

impl Default for SelectionStateV1 {
    fn default() -> Self { SelectionState::default().into() }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SelectionStateV2 {
    Normal,
    Hover,
    Focus,
    Pressed,
    Active,
}

impl From<SelectionState> for SelectionStateV2 {
    fn from(all: SelectionState) -> Self { all.v2() }
}

impl Default for SelectionStateV2 {
    fn default() -> Self { SelectionState::default().into() }
}

pub trait Selectable {
    fn selection_changed(&mut self, state: SelectionState);
}

#[derive(Clone, Debug, Default)]
pub struct SelectableData<T> {
    state: drying_paint::Watched<SelectionState>,
    normal: T,
    hover: Option<T>,
    focus: Option<T>,
    pressed: Option<T>,
    active: Option<T>,
}

impl<T> SelectableData<T> {
    pub fn builder(normal: T) -> SelectableDataBuilder<T> {
        SelectableDataBuilder {
            content: Self {
                state: Default::default(),
                normal,
                hover: None,
                focus: None,
                pressed: None,
                active: None,
            }
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
            },
            SelectionStateAll::Focus => {
                self.focus.as_ref().unwrap_or(&self.normal)
            },
            SelectionStateAll::Pressed => {
                #[allow(clippy::or_fun_call)]
                self.pressed.as_ref()
                    .or(self.focus.as_ref())
                    .unwrap_or(&self.normal)
            },
            SelectionStateAll::Active => {
                self.active.as_ref().unwrap_or(&self.normal)
            },
        }
    }
}

impl<T> std::ops::DerefMut for SelectableData<T> {
    fn deref_mut(&mut self) -> &mut T {
        match self.state.0 {
            SelectionStateAll::Normal => &mut self.normal,
            SelectionStateAll::Hover => {
                self.hover.as_mut().unwrap_or(&mut self.normal)
            },
            SelectionStateAll::Focus => {
                self.focus.as_mut().unwrap_or(&mut self.normal)
            },
            SelectionStateAll::Pressed => {
                #[allow(clippy::or_fun_call)]
                self.pressed.as_mut()
                    .or(self.focus.as_mut())
                    .unwrap_or(&mut self.normal)
            },
            SelectionStateAll::Active => {
                self.active.as_mut().unwrap_or(&mut self.normal)
            },
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct SelectableDataBuilder<T> {
    content: SelectableData<T>,
}

impl<T> SelectableDataBuilder<T> {
    pub fn hover(mut self, item: T) -> Self {
        self.content.hover = Some(item);
        self
    }

    pub fn focus(mut self, item: T) -> Self {
        self.content.focus = Some(item);
        self
    }

    pub fn pressed(mut self, item: T) -> Self {
        self.content.pressed = Some(item);
        self
    }

    pub fn active(mut self, item: T) -> Self {
        self.content.active = Some(item);
        self
    }

    pub fn build(self) -> SelectableData<T> {
        self.content
    }
}

impl<T> Selectable for SelectableData<T> {
    fn selection_changed(&mut self, state: SelectionState) {
        *self.state = state;
    }
}


#[derive(Clone, Copy, Debug, Default)]
pub struct SelectableIgnored<T> {
    data: T,
}

impl<T> std::ops::Deref for SelectableIgnored<T> {
    type Target = T;
    fn deref(&self) -> &T { &self.data }
}

impl<T> std::ops::DerefMut for SelectableIgnored<T> {
    fn deref_mut(&mut self) -> &mut T { &mut self.data }
}

impl<T> Selectable for SelectableIgnored<T> {
    fn selection_changed(&mut self, _state: SelectionState) { }
}

mod extra_impls {
    use crate::platform::RenderPlatform;
    use crate::pointer::PointerEvent;
    use crate::graphics::{
        DrawContext,
        Graphic,
    };
    use crate::widget::{
        WidgetContent,
        WidgetInit,
        WidgetChildReceiver,
        WidgetGraphicReceiver,
        WidgetExtra,
    };
    use super::*;

    impl<T, P> WidgetContent<P> for SelectableIgnored<T>
    where
        P: RenderPlatform,
        T: WidgetContent<P>,
    {
        fn init<I: WidgetInit<Self, P>>(mut init: I) {
            init.init_child_inline(|x| &mut x.data);
        }

        fn children<R: WidgetChildReceiver<P>>(&mut self, receiver: R) {
            self.data.children(receiver);
        }

        fn graphics<R: WidgetGraphicReceiver<P>>(&mut self, receiver: R) {
            self.data.graphics(receiver);
        }

        fn hittest(&self, extra: &mut WidgetExtra<'_>, point: (f32, f32)) -> bool {
            self.data.hittest(extra, point)
        }

        fn pointer_event(
            &mut self,
            extra: &mut WidgetExtra<'_>,
            event: &mut PointerEvent,
        ) -> bool {
            self.data.pointer_event(extra, event)
        }
    }

    impl <T, P> Graphic<P> for SelectableIgnored<T>
    where
        P: RenderPlatform,
        T: Graphic<P>,
    {
        fn draw(&mut self, ctx: &mut DrawContext<P>) {
            self.data.draw(ctx)
        }
    }

    impl <T, P> Graphic<P> for SelectableData<T>
    where
        P: RenderPlatform,
        T: Graphic<P>,
    {
        fn draw(&mut self, ctx: &mut DrawContext<P>) {
            T::draw(self, ctx)
        }
    }
}
