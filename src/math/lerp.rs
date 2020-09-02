/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub trait Lerp {
    type Output;
    fn lerp(from: &Self, to: &Self, t: f32) -> Self::Output;
}

impl Lerp for f32 {
    type Output = f32;

    fn lerp(from: &f32, to: &f32, t: f32) -> f32 {
        let diff = to - from;
        if t <= 0.5 {
            from + diff * t
        } else {
            to - diff * (1.0 - t)
        }
    }
}

impl Lerp for f64 {
    type Output = f64;

    fn lerp(from: &f64, to: &f64, t: f32) -> f64 {
        let t = t as f64;
        let diff = to - from;
        if t <= 0.5 {
            from + diff * t
        } else {
            to - diff * (1.0 - t)
        }
    }
}


pub trait LerpDistance {
    fn lerp_distance(a: &Self, b: &Self) -> f32;
}

impl LerpDistance for f32 {
    fn lerp_distance(a: &Self, b: &Self) -> f32 {
        (a - b).abs()
    }
}
