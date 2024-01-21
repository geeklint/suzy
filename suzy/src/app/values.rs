/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use std::{cell::Cell, rc::Rc, time};

use crate::watch::{DefaultOwner, WatchedValue};

type WatchedCellCore<T> =
    crate::watch::WatchedCellCore<'static, T, DefaultOwner>;

thread_local! {
    static CURRENT: Cell<Option<Rc<AppState>>> = Cell::new(None);
}

pub struct AppState {
    pub(super) frame_start: WatchedCellCore<time::Instant>,
    pub(super) coarse_time: WatchedCellCore<time::Instant>,
    pub(super) window_width: WatchedCellCore<f32>,
    pub(super) window_height: WatchedCellCore<f32>,
    pub(super) dpi: WatchedCellCore<[f32; 2]>,
}

impl AppState {
    pub const COARSE_STEP: time::Duration = time::Duration::from_secs(1);

    pub fn time(&self) -> impl '_ + WatchedValue<Value = time::Instant> {
        &self.frame_start
    }

    pub fn coarse_time(
        &self,
    ) -> impl '_ + WatchedValue<Value = time::Instant> {
        &self.coarse_time
    }

    pub fn window_width(&self) -> impl '_ + WatchedValue<Value = f32> {
        &self.window_width
    }

    pub fn window_height(&self) -> impl '_ + WatchedValue<Value = f32> {
        &self.window_height
    }

    pub fn dpi(&self) -> impl '_ + WatchedValue<Value = [f32; 2]> {
        &self.dpi
    }

    pub(crate) fn new_now(width: f32, height: f32) -> Self {
        let now = time::Instant::now();
        Self {
            frame_start: WatchedCellCore::new(now),
            coarse_time: WatchedCellCore::new(now),
            window_width: WatchedCellCore::new(width),
            window_height: WatchedCellCore::new(height),
            dpi: WatchedCellCore::new([96.0, 96.0]),
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
