/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

#![warn(
    //missing_docs,
    elided_lifetimes_in_paths,
    rust_2018_idioms,
    clippy::clone_on_ref_ptr,
    clippy::default_numeric_fallback,
    clippy::todo,
    clippy::print_stdout,
    clippy::default_trait_access,
    clippy::unwrap_used,
    clippy::ptr_as_ptr,
)]

//! ## Create an application with Suzy
//!
//! Suzy allows you to create a GUI comprised of widgets.
//!
//! The most basic app template will look something like this:
//!
#![cfg_attr(
    feature = "platform-opengl",
    doc = "```rust,no_run
# use suzy::widget;
#[derive(Default)]
struct Data { }

impl widget::Content for Data {
    fn desc(_desc: impl widget::Desc<Self>) {}
}"
)]
#![cfg_attr(
    feature = "quickstart",
    doc = "
# use suzy::widget::RunAsApp;
    Data::run_as_app();
"
)]
#![cfg_attr(feature = "platform-opengl", doc = "```")]
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
#![cfg_attr(
    feature = "platform-opengl",
    doc = "```rust
# use suzy::widget::{self, Widget};
# use suzy::dims::Rect;
# struct Data { child: Widget<()> }
# impl widget::Content for Data {
#     fn desc(mut desc: impl widget::Desc<Self>) {
        desc.watch(|this, rect| {
            this.child.set_width(rect.width() / 2.0);
        });
# }}
```"
)]
//!
//! When the parent changes size, the closure will be re-run and update the
//! size of the child.
//!
//! See the [`watch`](crate::watch) module documentation for more
//! information about the watch system.

pub mod adapter;
pub mod animation;
pub mod app;
pub mod dims;
pub mod graphics;
pub mod platform;
#[macro_use]
pub mod platforms;
pub mod pointer;
pub mod selectable;
pub mod text;
pub mod units;
pub mod watch;
pub mod widget;
pub mod widgets;

/// The Suzy prelude contains exports of the most frequently-used types
pub mod prelude {
    #[doc(no_inline)]
    pub use crate::{
        dims::Rect,
        watch::Watched,
        widget::{self, Widget},
    };
}

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
