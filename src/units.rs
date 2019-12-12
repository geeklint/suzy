use crate::app::try_with_current;

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
    value * try_with_current(|values| *values.px_per_dp).unwrap_or(1.0)
}

pub fn px(px: f32) -> f32 {
    px / try_with_current(|values| *values.px_per_dp).unwrap_or(1.0)
}

pub fn to_cell(value: f32) -> i32 {
    let cell_size = try_with_current(
        |values| *values.cell_size).unwrap_or(16.0);
    (value / cell_size).round() as i32
}

pub fn cell(cell: i32) -> f32 {
    let cell_size = try_with_current(
        |values| *values.cell_size).unwrap_or(16.0);
    (cell as f32) * cell_size
}
