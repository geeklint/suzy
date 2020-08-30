/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::app::AppValues;

#[inline]
pub fn to_dp(value: f32) -> f32 { value }
#[inline]
pub fn dp(dp: f32) -> f32 { dp }

const DPI: f32 = 160.0;

#[inline]
pub fn to_inches(value: f32) -> f32 { value / DPI }
#[inline]
pub fn inches(inches: f32) -> f32 { inches * DPI }

const MM_PER_INCH: f32 = 25.4;

#[inline]
pub fn to_mm(value: f32) -> f32 { to_inches(value) * MM_PER_INCH }
#[inline]
pub fn mm(mm: f32) -> f32 { inches(mm / MM_PER_INCH) }

#[inline]
pub fn to_cm(value: f32) -> f32 { to_mm(value) / 10.0 }
#[inline]
pub fn cm(cm: f32) -> f32 { mm(cm * 10.0) }

pub fn to_px(value: f32) -> f32 {
    value * AppValues::px_per_dp()
}

pub fn px(px: f32) -> f32 {
    px / AppValues::px_per_dp()
}

pub fn to_cells(value: f32) -> i32 {
    (value / AppValues::cell_size()).round() as i32
}

pub fn cells(cells: i32) -> f32 {
    (cells as f32) * AppValues::cell_size()
}
