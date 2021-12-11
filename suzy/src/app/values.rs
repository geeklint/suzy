/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use drying_paint::Watched;
use std::cell::RefCell;
use std::time;

thread_local! {
    static APP_STACK: RefCell<Vec<AppValues>> = RefCell::new(Vec::new());
}

#[derive(Clone, Debug)]
pub(crate) struct AppValues {
    pub frame_start: Watched<time::Instant>,
    pub coarse_time: Watched<time::Instant>,
    pub cell_size: Watched<f32>,
    pub px_per_dp: Watched<f32>,
    pub window_size: (f32, f32),
}

impl AppValues {
    pub const COARSE_STEP: time::Duration = time::Duration::from_secs(1);

    pub(super) fn with<F: FnOnce() -> R, R>(self, func: F) -> (Self, R) {
        APP_STACK.with(|cell| cell.borrow_mut().push(self));
        let res = (func)();
        let values =
            APP_STACK
                .with(|cell| cell.borrow_mut().pop())
                .expect(concat!(
                    "Failed to pop from APP_STACK,",
                    "it must have been modified incorrectly"
                ));
        (values, res)
    }

    pub(crate) fn try_with_current<F, R>(func: F) -> Option<R>
    where
        F: FnOnce(&AppValues) -> R,
    {
        APP_STACK.with(|cell| cell.borrow().last().map(func))
    }

    pub(crate) fn expect_current<F, R>(func: F) -> R
    where
        F: FnOnce(&AppValues) -> R,
    {
        APP_STACK.with(|cell| {
            let stack = cell.borrow();
            let top = stack.last().expect("App context is not valid");
            (func)(top)
        })
    }

    pub(crate) fn expect_current_mut<F, R>(func: F) -> R
    where
        F: FnOnce(&mut AppValues) -> R,
    {
        APP_STACK.with(|cell| {
            let mut stack = cell.borrow_mut();
            let top = stack.last_mut().expect("App context is not valid");
            (func)(top)
        })
    }

    pub(crate) fn px_per_dp() -> f32 {
        Self::try_with_current(|values| *values.px_per_dp).unwrap_or(1.0)
    }

    pub(crate) fn cell_size() -> f32 {
        Self::try_with_current(|values| *values.cell_size).unwrap_or(16.0)
    }
}

pub(crate) fn get_cell_size(width: f32, height: f32) -> f32 {
    for dist in 0..=4 {
        let dist = dist as f32;
        let high = 16.0 + dist;
        let low = 16.0 - dist;
        if width % high == 0.0 && height % high == 0.0 {
            return high;
        } else if width % low == 0.0 && height % low == 0.0 {
            return low;
        }
    }
    let (longer, shorter) = if width > height {
        (width, height)
    } else {
        (height, width)
    };
    let min_cells = (longer / 12.0).ceil() as u32;
    let max_cells = (longer / 20.0).floor() as u32;
    let mut best = 0.0;
    let mut best_dist = std::f32::INFINITY;
    for test_cells in min_cells..=max_cells {
        let cell_size = longer / (test_cells as f32);
        let tiles_shorter = shorter / cell_size;
        let dist = (tiles_shorter - tiles_shorter.round()).abs();
        if dist < best_dist {
            best = cell_size;
            best_dist = dist;
        }
    }
    best
}
