/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This module provides traits that describe how a component such as a
//! custom Widget responds to a change in an external data source.

pub trait Adaptable<T: ?Sized>: for<'a> From<&'a T> {
    fn adapt(&mut self, data: &T);
}
