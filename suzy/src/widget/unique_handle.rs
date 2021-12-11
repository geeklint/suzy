/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use crate::{pointer::PointerId, watch::WatchedEvent};

#[derive(Default)]
struct HandleContents {
    grab_stolen: RefCell<WatchedEvent<PointerId>>,
}

#[derive(Default)]
pub struct UniqueHandle {
    ptr: Rc<HandleContents>,
}

impl UniqueHandle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn id(&self) -> UniqueHandleId {
        UniqueHandleId {
            ptr: Rc::downgrade(&self.ptr),
        }
    }

    pub fn handle_pointer_grab_stolen<F: FnOnce(PointerId)>(&self, f: F) {
        if let Some(&id) = self.ptr.grab_stolen.borrow().bind() {
            f(id);
        }
    }
}

#[derive(Clone)]
pub struct UniqueHandleId {
    ptr: Weak<HandleContents>,
}

impl UniqueHandleId {
    pub fn notify_grab_stolen(&self, pointer: PointerId) {
        if let Some(strong) = self.ptr.upgrade() {
            strong.grab_stolen.borrow_mut().dispatch(pointer);
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
