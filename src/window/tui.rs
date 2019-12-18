use std::fs;
use std::io;
use std::ffi::{CString, CStr};
use std::convert::TryInto;
use std::os::unix::io::{AsRawFd, IntoRawFd};

use crate::platform::tui as ffi;

macro_rules! assert_nerr {
    ($res:expr) => {
        assert!($res != ffi::ERR, "Curses Library Error");
    };
}

pub struct Window {
    term: *mut ffi::screen,
    stdscr: *mut ffi::_win_st,
    in_: *mut libc::FILE,
    out: *mut libc::FILE,
    #[allow(dead_code)]
    stdout_gag: Option<gag::Hold>,
    #[allow(dead_code)]
    stderr_gag: Option<gag::Hold>,
    curs: i32,
}

impl Window {
    pub fn new() -> Result<Self, io::Error> {
        unsafe {
            let empty = std::ffi::CString::new("").unwrap();
            libc::setlocale(libc::LC_CTYPE, empty.as_ptr());
        };
        let mut in_ = io::stdin().as_raw_fd();
        let mut out = io::stdout().as_raw_fd();
        let err = io::stderr().as_raw_fd();
        let normal = unsafe {
            libc::isatty(in_) != 0 && libc::isatty(out) != 0
        };
        let (stdout_gag, stderr_gag) = if normal {
            let term = fs::read_link("/proc/self/fd/1")?;
            let err_dest = fs::read_link("/proc/self/fd/2")?;
            let stderr_gag = if err_dest == term {
                Some(gag::Hold::stderr()?)
            } else {
                None
            };
            in_ = unsafe { libc::dup(in_) };
            out = unsafe { libc::dup(out) };
            (Some(gag::Hold::stdout()?), stderr_gag)
        } else {
            let tty = fs::OpenOptions::new()
                .read(true).write(true).open("/dev/tty")?;
            let stdout_gag = if unsafe { libc::isatty(out) != 0 } {
                Some(gag::Hold::stdout()?)
            } else {
                None
            };
            let stderr_gag = if unsafe { libc::isatty(err) != 0 } {
                Some(gag::Hold::stderr()?)
            } else {
                None
            };
            in_ = tty.into_raw_fd();
            out = unsafe { libc::dup(in_) };
            (stdout_gag, stderr_gag)
        };
        assert!(unsafe { libc::isatty(in_) } != 0);
        assert!(unsafe { libc::isatty(out) } != 0);
        let in_ = unsafe {
            libc::fdopen(
                in_,
                CStr::from_bytes_with_nul_unchecked(b"r\0").as_ptr())
        };
        if in_.is_null() {
            panic!("Could not turn in_ into FILE *");
        }
        let out = unsafe {
            libc::fdopen(
                out,
                CStr::from_bytes_with_nul_unchecked(b"w\0").as_ptr())
        };
        if out.is_null() {
            panic!("Could not turn out into FILE *");
        }
        let term = unsafe {
            ffi::newterm(
                std::ptr::null(),
                out as *mut ffi::FILE,
                in_ as *mut ffi::FILE,
            )
        };
        if term.is_null() {
            panic!("Curses newterm returned null");
        }
        let stdscr = unsafe {
            ffi::set_term(term);
            ffi::stdscr
        };
        if stdscr.is_null() {
            panic!("Curses initscr returned null");
        }
        let curs = unsafe {
            assert_nerr!(ffi::noecho());
            assert_nerr!(ffi::cbreak());
            assert_nerr!(ffi::keypad(stdscr, true));
            assert_nerr!(ffi::nodelay(stdscr, true));
            ffi::curs_set(0)
        };
        unsafe { ffi::clear(); }
        Ok(Window { term, stdscr, curs, in_, out, stdout_gag, stderr_gag })
    }

    pub fn get_size(&self) -> (f32, f32) {
        let rows;
        let cols;
        unsafe {
            rows = ffi::getmaxy(self.stdscr);
            cols = ffi::getmaxx(self.stdscr);
        }
        ((cols as f32) * 16.0, (rows as f32) * 16.0)
    }

    pub fn events(&self) -> Events {
        Events { window: self }
    }

    pub fn flip(&mut self) {
        let ch = unsafe {
            ffi::refresh();
            ffi::nodelay(self.stdscr, false);
            ffi::halfdelay(1);
            ffi::getch()
        };
        if ch != ffi::ERR {
            unsafe { ffi::ungetch(ch); }
        }
        unsafe { ffi::nodelay(self.stdscr, true); }
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        unsafe {
            ffi::curs_set(self.curs);
            ffi::nocbreak();
            ffi::keypad(self.stdscr, false);
            ffi::echo();
            ffi::endwin();
            ffi::delscreen(self.term);
            libc::fclose(self.in_);
            libc::fclose(self.out);
        }
    }
}

pub struct Events<'a> {
    window: &'a Window,
}

impl Iterator for Events<'_> {
    type Item = super::WindowEvent;
    fn next(&mut self) -> Option<super::WindowEvent> {
        let event = unsafe { ffi::getch() };
        if event == ffi::ERR {
            None
        } else if Ok(event) == ffi::KEY_RESIZE.try_into() {
            let rows;
            let cols;
            unsafe {
                rows = ffi::getmaxy(self.window.stdscr);
                cols = ffi::getmaxx(self.window.stdscr);
            }
            let width = (cols as f32) * 16.0;
            let height = (rows as f32) * 16.0;
            Some(super::WindowEvent::Resize(width, height))
        } else {
            Some(super::WindowEvent::KeyDown(event))
        }
    }
}
