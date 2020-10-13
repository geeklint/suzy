/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![warn(clippy::clone_on_ref_ptr)]
#![warn(clippy::todo)]
#![warn(clippy::print_stdout)]
#![allow(clippy::needless_doctest_main)]

//! ## Create an application with Suzy
//!
//! Suzy allows you to create a GUI comprised of widgets.
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
//! ## Watch System
//!
//! Suzy's watch system provides the main way to define functionality within
//! the framework.  It enables you to describe the relationships between
//! widgets in a declarative way.
//!
//! For example, if you wanted to make a widget half the width of its
//! parent:
//!
//! ```rust
//! # use suzy::widget::*;
//! # use suzy::dims::Rect;
//! # struct Data { child: Widget<()> }
//! # impl WidgetContent for Data {
//! #     fn init<I: WidgetInit<Self>>(mut init: I) {
//! init.watch(|this, rect| {
//!     this.child.set_width(rect.width() / 2.0);
//! });
//! #     }
//! #     fn children<R: WidgetChildReceiver>(&mut self, _receiver: R) {}
//! #     fn graphics<R: WidgetGraphicReceiver>(&mut self, _receiver: R) {}
//! # }
//! ```
//!
//! When the parent changes size, the closure will be re-run and update the
//! size of the child.
//!
//! See the [`watch`](watch/index.html) module documentation for more
//! information about the watch system.

pub mod adapter;
pub mod animation;
pub mod app;
pub mod dims;
pub mod graphics;
pub mod pointer;
pub mod selectable;
pub mod units;
pub mod widget;
pub mod window;
pub mod platform;
pub mod watch;
pub mod widgets;


/// A version of the tweak! macro from the crate
/// [`inline_tweak`](https://crates.io/crates/inline_tweak), but designed to
/// work within Suzy's watch system.
///
/// Using this macro in a watch closure will cause the closure to be re-run
/// periodically, so that a tweaked value in the parsed source code can
/// be observed.
///
/// This macro is available if the `inline_tweak` feature is enabled for Suzy.
#[cfg(feature = "inline_tweak")]
#[cfg(debug_assertions)]
#[macro_export]
macro_rules! tweak {
    // this macro adapted from crate `inline_tweak`, version 1.0.8
    ($default:expr) => {{
        $crate::app::App::<$crate::platform::DefaultPlatform>::coarse_time();
        $crate::watch::inline_tweak(None, file!(), line!(), column!())
            .unwrap_or_else(|| $default)
    }};
    ($value:literal; $default:expr) => {{
        $crate::app::App::<$crate::platform::DefaultPlatform>::coarse_time();
        $crate::watch::inline_tweak(Some($value), file!(), line!(), column!())
            .unwrap_or_else(|| $default)
    }};
}

#[cfg(feature = "inline_tweak")]
#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! tweak {
    // this macro adapted from crate `inline_tweak`, version 1.0.8
    ($default:expr) => {
        $default
    };
    ($value:literal; $default:expr) => {
        $default
    };
}
