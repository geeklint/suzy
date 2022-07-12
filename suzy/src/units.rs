/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

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

use drying_paint::DefaultOwner;

use crate::{app::AppState, watch::WatchArg};

/// The ratio Suzy assumes between dp and inches.
pub const DPI: f32 = 96.0;

const MM_PER_INCH: f32 = 25.4;

// dp is an identity function but it conveys semantic meaning, this alias
// exists to avoid implying anything about the units of the value
fn to_f32<T: Units>(value: T) -> f32 {
    value.dp()
}

pub trait Units: Sized {
    /// This function is an identity function, since dp is the native unit
    fn to_dp(self) -> f32;

    /// This function is an identity function, since dp is the native unit
    fn dp(self) -> f32;

    /// Convert dp to inches
    #[inline]
    fn to_inches(self) -> f32 {
        to_f32(self) / DPI
    }

    /// Convert inches to dp
    #[inline]
    fn inches(self) -> f32 {
        to_f32(self) * DPI
    }

    /// Convert dp to millimeters
    #[inline]
    fn to_mm(self) -> f32 {
        self.inches() * MM_PER_INCH
    }

    /// Convert millimeters to dp
    #[inline]
    fn mm(self) -> f32 {
        (to_f32(self) / MM_PER_INCH).inches()
    }

    /// Convert dp to centimeters
    #[inline]
    fn to_cm(self) -> f32 {
        self.to_mm() / 10.0
    }

    /// Convert centimeters to dp
    #[inline]
    fn cm(self) -> f32 {
        (to_f32(self) * 10.0).mm()
    }

    /// Convert dp to real physical pixels
    #[inline]
    fn to_px_explicit(
        self,
        state: &AppState,
        ctx: WatchArg<'_, 'static, DefaultOwner>,
    ) -> f32 {
        self.dp() * state.px_per_dp().get(ctx)
    }

    /// Convert dp to real physical pixels
    #[inline]
    fn to_px(self) -> f32 {
        self.dp()
            * AppState::try_with_current(|state| *state.px_per_dp().get_auto())
                .expect("unable to find app state to get current DPI from")
    }

    /// Convert real physical pixels to dp
    #[inline]
    fn px_explicit(
        self,
        state: &AppState,
        ctx: WatchArg<'_, 'static, DefaultOwner>,
    ) -> f32 {
        to_f32(self) / state.px_per_dp().get(ctx)
    }

    /// Convert real physical pixels to dp
    #[inline]
    fn px(self) -> f32 {
        to_f32(self)
            / AppState::try_with_current(|state| *state.px_per_dp().get_auto())
                .expect("unable to find app state to get current DPI from")
    }

    /// Round dp to a cell boundry
    ///
    /// See the [module-level documentation](./index.html) for an explanation of
    /// the `cell` unit.
    fn to_cells_explicit(
        self,
        state: &AppState,
        ctx: WatchArg<'_, 'static, DefaultOwner>,
    ) -> i32 {
        (self.dp() / state.cell_size().get(ctx)).round() as i32
    }
}

impl Units for f32 {
    fn to_dp(self) -> f32 {
        self
    }

    fn dp(self) -> f32 {
        self
    }
}

pub trait CellUnit: Sized {
    /// Convert a number of cells to a size in dp
    ///
    /// See the [module-level documentation](./index.html) for an explanation of
    /// the `cell` unit.
    fn cells_explicit(
        self,
        state: &AppState,
        ctx: WatchArg<'_, 'static, DefaultOwner>,
    ) -> f32;
}

impl CellUnit for i32 {
    fn cells_explicit(
        self,
        state: &AppState,
        ctx: WatchArg<'_, 'static, DefaultOwner>,
    ) -> f32 {
        (self as f32) * state.cell_size().get(ctx)
    }
}
