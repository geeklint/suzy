
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum SelectionStateAll {
    Normal,
    Hover,
    Focus,
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
    pub const fn active() -> Self { Self(SelectionStateAll::Active) }

    pub fn v0(self) -> SelectionStateV0 {
        match self.0 {
            SelectionStateAll::Normal => SelectionStateV0::Normal,
            SelectionStateAll::Hover => SelectionStateV0::Normal,
            SelectionStateAll::Focus => SelectionStateV0::Focus,
            SelectionStateAll::Active => SelectionStateV0::Active,
        }
    }
    pub fn v1(self) -> SelectionStateV1 {
        match self.0 {
            SelectionStateAll::Normal => SelectionStateV1::Normal,
            SelectionStateAll::Hover => SelectionStateV1::Hover,
            SelectionStateAll::Focus => SelectionStateV1::Focus,
            SelectionStateAll::Active => SelectionStateV1::Active,
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

pub trait Selectable {
    fn selection_changed(&mut self, state: SelectionState);
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
