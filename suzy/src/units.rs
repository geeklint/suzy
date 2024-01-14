/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

//! Convenience functions to convert between measurable units of visible size.
//!
//! The default unit in Suzy is called a 'dp' and represents a logical dimension
//! according to a window's scale factor.
//!
//! The units `inches`, `mm`, `cm` are standard physical sizes.
//!
//! The `cell` unit represents some larger size (close to 16dp), which evenly
//! divides the window.  This is intended for compatibility with character
//! cell based interfaces, like a text-based terminal.

use drying_paint::DefaultOwner;

use crate::{app::AppState, watch::WatchArg};

/// The ratio Suzy assumes between dp and inches.
pub const DPI: f32 = 96.0;

const MM_PER_INCH: f32 = 25.4;

pub trait UnitConversion: Sized {
    /// This function is an identity function, since dp is the native unit
    fn to_dp(self) -> f32;

    /// This function is an identity function, since dp is the native unit
    fn dp(self) -> f32;

    fn convert<'a>(
        self,
        state: &'a AppState,
        ctx: WatchArg<'a, 'static, DefaultOwner>,
    ) -> Convert<'a> {
        Convert {
            source: self.dp(),
            state,
            ctx,
        }
    }

    /// Convert dp to inches
    #[inline]
    fn to_inches(self) -> impl AxisRelativeValue {
        Convert::auto(self, |cvt| cvt.to_inches())
    }

    /// Convert inches to dp
    #[inline]
    fn inches(self) -> impl AxisRelativeValue {
        Convert::auto(self, |cvt| cvt.inches())
    }

    /// Convert dp to millimeters
    #[inline]
    fn to_mm(self) -> impl AxisRelativeValue {
        Convert::auto(self, |cvt| cvt.to_mm())
    }

    /// Convert millimeters to dp
    #[inline]
    fn mm(self) -> impl AxisRelativeValue {
        Convert::auto(self, |cvt| cvt.mm())
    }

    /// Convert dp to centimeters
    #[inline]
    fn to_cm(self) -> impl AxisRelativeValue {
        Convert::auto(self, |cvt| cvt.to_cm())
    }

    /// Convert centimeters to dp
    #[inline]
    fn cm(self) -> impl AxisRelativeValue {
        Convert::auto(self, |cvt| cvt.cm())
    }
}

impl UnitConversion for f32 {
    fn to_dp(self) -> f32 {
        self
    }

    fn dp(self) -> f32 {
        self
    }
}

#[derive(Clone, Copy)]
pub struct Convert<'a> {
    source: f32,
    state: &'a AppState,
    ctx: WatchArg<'a, 'static, DefaultOwner>,
}

impl Convert<'_> {
    fn auto<T, F, R>(source: T, func: F) -> R
    where
        T: UnitConversion,
        F: FnOnce(Convert<'_>) -> R,
    {
        let source = source.dp();
        // TODO: support conversions outside of watch fns?
        let mut ret = None;
        WatchArg::try_with_current(|ctx| {
            AppState::try_with_current(|state| {
                ret = Some(func(Convert { source, state, ctx }));
            });
        });
        ret.expect("unable to find app state to get conversions from")
    }

    fn calc_dpi<F>(self, f: F) -> impl AxisRelativeValue
    where
        F: FnOnce(f32, f32) -> f32,
    {
        AxisRelValue {
            source: self.source,
            dpi: self.state.dpi.get(self.ctx),
            f,
        }
    }

    pub fn to_inches(self) -> impl AxisRelativeValue {
        self.calc_dpi(|source, dpi| source / dpi)
    }

    pub fn inches(self) -> impl AxisRelativeValue {
        self.calc_dpi(|source, dpi| source * dpi)
    }

    pub fn to_mm(self) -> impl AxisRelativeValue {
        self.calc_dpi(|source, dpi| source / dpi * MM_PER_INCH)
    }

    pub fn mm(self) -> impl AxisRelativeValue {
        self.calc_dpi(|source, dpi| source / MM_PER_INCH * dpi)
    }

    pub fn to_cm(self) -> impl AxisRelativeValue {
        self.calc_dpi(|source, dpi| source / dpi * MM_PER_INCH / 10.0)
    }

    pub fn cm(self) -> impl AxisRelativeValue {
        self.calc_dpi(|source, dpi| source * 10.0 / MM_PER_INCH * dpi)
    }
}

pub trait AxisRelativeValue {
    fn horizontal(self) -> f32;
    fn vertical(self) -> f32;
}

struct AxisRelValue<F> {
    source: f32,
    dpi: [f32; 2],
    f: F,
}

impl<F> AxisRelativeValue for AxisRelValue<F>
where
    F: FnOnce(f32, f32) -> f32,
{
    fn horizontal(self) -> f32 {
        (self.f)(self.source, self.dpi[0])
    }

    fn vertical(self) -> f32 {
        (self.f)(self.source, self.dpi[1])
    }
}
