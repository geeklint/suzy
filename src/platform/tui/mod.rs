use std::fs;
use std::io;
use std::ffi::CStr;
use std::convert::TryInto;
use std::os::unix::io::{AsRawFd, IntoRawFd};

mod ffi;


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
            out = in_;
            (stdout_gag, stderr_gag)
        };
        let in_ = unsafe {
            libc::fdopen(
                in_,
                CStr::from_bytes_with_nul_unchecked(b"r\0").as_ptr())
        };
        let out = unsafe {
            libc::fdopen(
                out,
                CStr::from_bytes_with_nul_unchecked(b"w\0").as_ptr())
        };
        let term = unsafe {
            ffi::newterm(
                std::ptr::null(),
                out as *mut ffi::FILE,
                in_ as *mut ffi::FILE)
        };
        let stdscr = unsafe {
            ffi::set_term(term);
            ffi::initscr()
        };
        let curs;
        unsafe {
            ffi::noecho();
            ffi::cbreak();
            ffi::keypad(stdscr, true);
            ffi::halfdelay(1);
            curs = ffi::curs_set(0);
        }
        Ok(Window { term, stdscr, in_, out, stdout_gag, stderr_gag, curs })
    }

    pub fn get_size(&self) -> (u32, u32) {
        let rows;
        let cols;
        unsafe {
            rows = ffi::getmaxy(self.stdscr);
            cols = ffi::getmaxx(self.stdscr);
        }
        (cols as u32, rows as u32)
    }

    pub fn events(&self) -> Events {
        Events { window: self }
    }

    pub fn flip(&mut self) {
        unsafe {
            ffi::refresh();
        }
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
            None
        }
    }
}
