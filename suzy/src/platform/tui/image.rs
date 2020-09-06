/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::convert::TryFrom;

use widestring::{WideCString, WideCStr};

use crate::app;
use crate::units;
use crate::dims::{Rect, SimpleRect, Dim};
use crate::platform::tui as ffi;
use crate::graphics::Graphic;

fn draw_at_cell(
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    string: &WideCStr,
    calced_len: u32,
) {
    // turn distance betwen bottoms to distance between tops
    let y = height - (y + 1);
    let end = (x as i64) + (calced_len as i64);
    if y >= 0 && y < height && x < width && end > 0 {
        let ptr = string.as_ptr() as *const libc::wchar_t;
        unsafe { ffi::mvaddwstr(y, x, ptr); }
    }
}

pub struct MeasuredString {
    string: WideCString,
    len: u32,
}

impl MeasuredString {
    pub fn len(&self) -> u32 { self.len }

    pub(crate) fn draw(&self, x: f32, y: f32) {
        let (width, height) = app::expect_current(|vals| vals.window_size );
        draw_at_cell(
            units::to_cells(x), units::to_cells(y),
            units::to_cells(width), units::to_cells(height),
            &self.string,
            self.len,
        );
    }

    // TODO: fill in real error
    fn from_str(s: impl AsRef<str>) -> Result<MeasuredString, ()> {
        let string = if let Ok(string) = WideCString::from_str(s) {
            string
        } else {
            return Err(())
        };
        let ptr = string.as_ptr() as *const libc::wchar_t;
        if unsafe { ffi::mvaddwstr(0, 0, ptr) } == ffi::ERR {
            return Err(())
        }
        let row = unsafe { ffi::getcury(ffi::stdscr) };
        let col = unsafe { ffi::getcurx(ffi::stdscr) };
        if row != 0 {
            return Err(());
        }
        Ok(MeasuredString { string, len: col as u32 })
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct WrongMeasure;

#[derive(Clone)]
pub struct SingleCellChar {
    symbol: WideCString,
}

impl SingleCellChar {
    pub fn from_str(s: impl AsRef<str>) -> Result<SingleCellChar, WrongMeasure> {
        if let Ok(ms) = MeasuredString::from_str(s) {
            if ms.len() == 1 {
                Ok(SingleCellChar { symbol: ms.string })
            } else {
                Err(WrongMeasure {})
            }
        } else {
            Err(WrongMeasure {})
        }
    }
}

#[derive(Clone)]
pub struct FreeformImage {
    ch: SingleCellChar,
    rect: SimpleRect,
}

impl FreeformImage {
    pub fn uniform(ch: SingleCellChar) -> Self {
        FreeformImage { ch , rect: Default::default() }
    }
}

impl Rect for FreeformImage {
    fn x(&self) -> Dim { self.rect.x() }
    fn y(&self) -> Dim { self.rect.y() }

    fn x_mut<F: FnOnce(&mut Dim)>(&mut self, f: F) {
        self.rect.x_mut(f);
    }

    fn y_mut<F: FnOnce(&mut Dim)>(&mut self, f: F) {
        self.rect.y_mut(f);
    }
}

impl Graphic for FreeformImage {
    fn draw(&self) {
        let (width, height) = app::expect_current(|vals| vals.window_size );
        let left = units::to_cells(self.rect.left());
        let right = units::to_cells(self.rect.right());
        let top = units::to_cells(self.rect.top());
        let bottom = units::to_cells(self.rect.bottom());
        for y in bottom..top {
            for x in left..right {
                draw_at_cell(
                    x, y,
                    units::to_cells(width), units::to_cells(height),
                    &self.ch.symbol,
                    1,
                );
            }
        }
    }
}
