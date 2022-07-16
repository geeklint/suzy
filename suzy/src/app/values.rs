/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use std::{cell::Cell, rc::Rc, time};

use crate::watch::DefaultOwner;

type WatchedCellCore<T> =
    crate::watch::WatchedCellCore<'static, T, DefaultOwner>;

thread_local! {
    static CURRENT: Cell<Option<Rc<AppState>>> = Cell::new(None);
}

pub struct AppState {
    pub(super) frame_start: WatchedCellCore<time::Instant>,
    pub(super) coarse_time: WatchedCellCore<time::Instant>,
    pub(super) cell_size: WatchedCellCore<f32>,
    pub(super) px_per_dp: WatchedCellCore<f32>,
    pub(super) window_width: WatchedCellCore<f32>,
    pub(super) window_height: WatchedCellCore<f32>,
}

impl AppState {
    pub const COARSE_STEP: time::Duration = time::Duration::from_secs(1);

    pub fn time(&self) -> &WatchedCellCore<time::Instant> {
        &self.frame_start
    }

    pub fn coarse_time(&self) -> &WatchedCellCore<time::Instant> {
        &self.coarse_time
    }

    pub fn cell_size(&self) -> &WatchedCellCore<f32> {
        &self.cell_size
    }

    pub fn px_per_dp(&self) -> &WatchedCellCore<f32> {
        &self.px_per_dp
    }

    pub(crate) fn new_now(width: f32, height: f32) -> Self {
        let now = time::Instant::now();
        Self {
            frame_start: WatchedCellCore::new(now),
            coarse_time: WatchedCellCore::new(now),
            cell_size: WatchedCellCore::new(get_cell_size(width, height)),
            px_per_dp: WatchedCellCore::new(1.0),
            window_width: WatchedCellCore::new(width),
            window_height: WatchedCellCore::new(height),
        }
    }

    pub(crate) fn use_as_current<F: FnOnce() -> R, R>(
        this: Rc<Self>,
        func: F,
    ) -> (Rc<Self>, R) {
        CURRENT.with(|cell| {
            let prev = cell.replace(Some(this));
            let res = (func)();
            let self_again = cell.replace(prev).expect("AppState removed from current before end of use_as_current call");
            (self_again, res)
        })
    }

    pub(crate) fn try_with_current<F, R>(func: F) -> Option<R>
    where
        F: FnOnce(&AppState) -> R,
    {
        CURRENT.with(|cell| {
            let state = cell.take()?;
            let ret = func(&state);
            cell.set(Some(state));
            Some(ret)
        })
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
