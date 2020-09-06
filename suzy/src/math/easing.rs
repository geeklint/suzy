/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub trait Easing {
    fn ease(&self, t: f32) -> f32;
}


#[derive(Copy, Clone)]
pub struct CubicPoly(pub f32, pub f32, pub f32, pub f32);

impl Easing for CubicPoly {
    fn ease(&self, t: f32) -> f32 {
        (self.0 * t.powi(3))
        + (self.1 * t.powi(2))
        + (self.2 * t.powi(1))
        + (self.3 * t)
    }
}

impl Default for CubicPoly {
    fn default() -> Self { CubicPoly(0.0, 0.0, 1.0, 0.0) }
}
