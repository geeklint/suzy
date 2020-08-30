/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub trait Easing {
    fn ease(&self, t: f32) -> f32;
}


#[derive(Copy, Clone)]
pub struct CubicBezier(pub f32, pub f32, pub f32, pub f32);

impl Easing for CubicBezier {
    fn ease(&self, t: f32) -> f32 {
        let invt = 1.0 - t;
        let part0 = invt.powi(3) * self.0;
        let part1 = 3.0 * invt.powi(2) * t * self.1;
        let part2 = 3.0 * invt * t.powi(2) * self.2;
        let part3 = t.powi(3) * self.3;
        part0 + part1 + part2 + part3
    }
}

impl Default for CubicBezier {
    fn default() -> Self { CubicBezier(0.0, 0.0, 1.0, 1.0) }
}
