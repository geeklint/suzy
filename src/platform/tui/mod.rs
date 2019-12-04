use std::sync::mpsc;
use std::convert::TryInto;

mod ffi;


pub struct Window {
    stdscr: *mut ffi::_win_st,
    lock: mpsc::Sender<()>,
    curs: i32,
}

impl Window {
    pub fn new() -> Result<Self, ()> {
        // hold stdout lock indefinitly
        let (lock, wait) = mpsc::channel();
        std::thread::spawn(move || {
            let stdout = std::io::stdout();
            let lock = stdout.lock();
            let _ = wait.recv();
            std::mem::drop(lock);
        });
        let stdscr = unsafe {
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
        Ok(Window { stdscr, lock, curs })
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
        let _ = self.lock.send(());
        unsafe {
            ffi::curs_set(self.curs);
            ffi::nocbreak();
            ffi::keypad(self.stdscr, false);
            ffi::echo();
            ffi::endwin();
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
            let mut rows;
            let mut cols;
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
