/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This module contains traits and types concerned with pure numerical
//! computation.

mod color;
mod dims;
mod easing;
mod lerp;
pub mod consts;

pub use color::Color;
pub use dims::*;
pub use easing::{Easing, CubicPoly};
pub use lerp::{
    Lerp,
    LerpDistance,
};

