/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use crate::{pointer::PointerId, watch::WatchedQueue};

#[derive(Default)]
struct HandleContents {
    grab_stolen: RefCell<WatchedQueue<'static, PointerId>>,
}

#[derive(Default)]
pub struct UniqueHandle {
    ptr: Rc<HandleContents>,
}

impl UniqueHandle {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn id(&self) -> UniqueHandleId {
        UniqueHandleId {
            ptr: Rc::downgrade(&self.ptr),
        }
    }

    pub fn handle_pointer_grab_stolen<F: FnOnce(PointerId)>(&self, f: F) {
        crate::watch::WatchArg::try_with_current(|arg| {
            self.ptr.grab_stolen.borrow().handle_item(arg, |id| {
                f(*id);
            });
        });
    }
}

#[derive(Clone)]
pub struct UniqueHandleId {
    ptr: Weak<HandleContents>,
}

impl UniqueHandleId {
    pub fn notify_grab_stolen(&self, pointer: PointerId) {
        if let Some(strong) = self.ptr.upgrade() {
            let mut pushed = false;
            crate::watch::WatchArg::try_with_current(|arg| {
                strong.grab_stolen.borrow_mut().push(arg, pointer);
                pushed = true;
            });
            if !pushed {
                strong.grab_stolen.borrow_mut().push_external(pointer);
            }
        }
    }
}

impl Eq for UniqueHandleId {}

impl PartialEq for UniqueHandleId {
    fn eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.ptr, &other.ptr)
    }
}

impl From<&UniqueHandle> for UniqueHandleId {
    fn from(handle: &UniqueHandle) -> Self {
        handle.id()
    }
}
