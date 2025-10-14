/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

//! Suzy comes with a set of built-in widgets.

mod button;
mod label;
mod togglebutton;

pub use button::{Button, ButtonBehavior, DefaultButtonContent};

pub use label::{Label, LabelContent};

pub use togglebutton::{ToggleButton, ToggleButtonGroup, ToggleButtonValue};

/// A trait for widgets which have an obvious singular text graphic associated
/// with them.
pub trait TextContent {
    /// Set the text of the widget.
    fn set_text(&mut self, text: &str);
}
