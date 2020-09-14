/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![warn(clippy::clone_on_ref_ptr)]
#![warn(clippy::todo)]
#![warn(clippy::print_stdout)]
#![allow(clippy::needless_doctest_main)]

//! ## Create an application with Suzy
//!
//! An application made with Suzy is comprised of widgets.  The typical
//! process will involve the following:
//! * Create a struct to hold the data associated with a custom widget
//! * Implement the trait [WidgetContent](widget/trait.WidgetContent.html)
//! * Add that widget to an [App](app/struct.App.html) as a "root" widget.
//!
//! The most basic app template will look something like this:
//!
//! ```rust,no_run
//! # use suzy::widget::*;
//! #[derive(Default)]
//! struct Data { }
//!
//! impl WidgetContent for Data {
//!     fn init<I: WidgetInit<Self>>(_init: I) {}
//!     fn children<R: WidgetChildReceiver>(&mut self, _receiver: R) {}
//!     fn graphics<R: WidgetGraphicReceiver>(&mut self, _receiver: R) {}
//! }
//!
//! fn main() {
//!     Data::run_as_app();
//! }
//! ```
//!
//! See the provided examples for examples of specific functionality.
//!
//! See the [watch](watch/index.html) module documentation for more
//! information about the observer patterns used in suzy.
//!
//! See the [WidgetContent](widget/trait.WidgetContent.html) documentation
//! for more information on the three required methods of that trait.

pub mod adapter;
pub mod animation;
pub mod app;
pub mod graphics;
pub mod math;
pub mod pointer;
pub mod selectable;
pub mod units;
pub mod widget;
pub mod window;
pub mod platform;
pub mod watch;
pub mod widgets;
