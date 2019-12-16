use std::convert::TryFrom;

use widestring::WideCString;

use crate::platform::tui as ffi;

pub struct MeasuredString {
    string: WideCString,
    len: usize,
}

impl MeasuredString {
    pub fn len(&self) -> usize { self.len }
}

impl MeasuredString {
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
        Ok(MeasuredString { string, len: col as usize })
    }
}

pub struct WrongMeasure;

pub struct SingleCellChar {
    symbol: WideCString,
}

impl SingleCellChar {
    fn from_str(s: impl AsRef<str>) -> Result<SingleCellChar, WrongMeasure> {
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

pub struct FreeformImage {
    ch: SingleCellChar,
}

impl FreeformImage {
    pub fn uniform(ch: SingleCellChar) -> Self {
        FreeformImage { ch }
    }
}
