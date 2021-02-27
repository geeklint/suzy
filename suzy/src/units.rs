/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Convenience functions to convert between measurable units of visible size.
//!
//! The default unit in Suzy is defined to be a 'dp' (1/96 inches, or a best
//! approximation based on a user's scaling factor.
//!
//! The units `inches`, `mm`, `cm` are standard physical sizes.
//!
//! The `px` unit represents a "real" pixel, regardless of scaling.
//!
//! The `cell` unit represents some larger size (close to 16dp), which evenly
//! divides the screen.  This is intended for compatibility with character
//! cell based interfaces, like a text-based terminal.

use crate::app::AppValues;

/// This function is an identity function, for API symetry.
#[inline]
pub fn to_dp(value: f32) -> f32 {
    value
}

/// This function is an identity function, for API symetry.
#[inline]
pub fn dp(dp: f32) -> f32 {
    dp
}

/// The ratio Suzy assumes between dp and inches.
pub const DPI: f32 = 96.0;

/// Convert dp to inches
#[inline]
pub fn to_inches(value: f32) -> f32 {
    value / DPI
}

/// Convert inches to dp
#[inline]
pub fn inches(inches: f32) -> f32 {
    inches * DPI
}

const MM_PER_INCH: f32 = 25.4;

/// Convert dp to millimeters
#[inline]
pub fn to_mm(value: f32) -> f32 {
    to_inches(value) * MM_PER_INCH
}

/// Convert millimeters to dp
#[inline]
pub fn mm(mm: f32) -> f32 {
    inches(mm / MM_PER_INCH)
}

/// Convert dp to centimeters
#[inline]
pub fn to_cm(value: f32) -> f32 {
    to_mm(value) / 10.0
}

/// Convert centimeters to dp
#[inline]
pub fn cm(cm: f32) -> f32 {
    mm(cm * 10.0)
}

/// Convert dp to real physical pixels
pub fn to_px(value: f32) -> f32 {
    value * AppValues::px_per_dp()
}

/// Convert real physical pixels to dp
pub fn px(px: f32) -> f32 {
    px / AppValues::px_per_dp()
}

/// Round dp to a cell boundry
///
/// See the [module-level documentation](./index.html) for an explanation of
/// the `cell` unit.
pub fn to_cells(value: f32) -> i32 {
    (value / AppValues::cell_size()).round() as i32
}

/// Convert a number of cells to a size in dp
///
/// See the [module-level documentation](./index.html) for an explanation of
/// the `cell` unit.
pub fn cells(cells: i32) -> f32 {
    (cells as f32) * AppValues::cell_size()
}
