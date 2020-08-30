/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

mod color;
mod easing;
mod lerp;
pub mod consts;

pub use lerp::Lerp;
pub use easing::{Easing, CubicBezier};
pub use color::Color;

