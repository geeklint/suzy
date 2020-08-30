/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub mod adapter;
pub mod app;
pub mod dims;
pub mod graphics;
pub mod math;
pub mod pointer;
pub mod selectable;
pub mod units;
pub mod widget;
pub mod window;
pub mod platform;
pub mod widgets;


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
