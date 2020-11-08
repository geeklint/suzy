/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Suzy comes with a set of built-in widgets.

mod button;
mod togglebutton;

pub use button::Button;

pub use togglebutton::{
    ToggleButtonGroup,
    ToggleButton,
    ToggleButtonValue,
};

/// A trait for widgets which have an obvious singular text graphic associated
/// with them.
pub trait TextContent {
    /// Set the text of the widget.
    fn set_text(&mut self, text: &str);
}
