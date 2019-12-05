use std::sync::atomic::{AtomicU32, Ordering};

struct Units {
    px: AtomicU32,
    cell: AtomicU32,
}

static UNITS: Units = Units {
    px: AtomicU32::new(1065353216),  // transmuted '1f32'
    cell: AtomicU32::new(1098907648),  // transmuted '16f32'
};

pub(crate) fn set_px_per_dp(value: f32) {
    UNITS.px.store(value.to_bits(), Ordering::Relaxed);
}

pub(crate) fn set_px_per_cell(value: f32) {
    UNITS.cell.store(value.to_bits(), Ordering::Relaxed);
}

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
    value * f32::from_bits(UNITS.px.load(Ordering::Relaxed))
}

pub fn px(px: f32) -> f32 {
    px / f32::from_bits(UNITS.px.load(Ordering::Relaxed))
}

pub fn to_cell(value: f32) -> i32 {
    (value / f32::from_bits(UNITS.cell.load(Ordering::Relaxed))).round() as i32
}

pub fn cell(cell: i32) -> f32 {
    (cell as f32) * f32::from_bits(UNITS.cell.load(Ordering::Relaxed))
}
