/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// TODO: clippy is probably right
#![allow(clippy::excessive_precision)]

use crate::animation::CubicPoly;

pub const EASE_LINEAR: CubicPoly = CubicPoly(0.0, 0.0, 1.0, 0.0);
pub const EASE_IN_SINE: CubicPoly = CubicPoly(-0.78008673, 2.0375746, -0.2556811, 0.01346003);
pub const EASE_OUT_SINE: CubicPoly = CubicPoly(-0.66147224, 0.02441384, 1.63212644, -0.01000451);
pub const EASE_IN_OUT_SINE: CubicPoly = CubicPoly(-2.60474551, 3.89310364, -0.31926717, 0.01461064);
pub const EASE_IN_QUAD: CubicPoly = CubicPoly(-0.27077543, 1.42494076, -0.15095372, 0.01786484);
pub const EASE_OUT_QUAD: CubicPoly = CubicPoly(0.18493503, -1.39214212, 2.21376862, -0.01454346);
pub const EASE_IN_OUT_QUAD: CubicPoly = CubicPoly(-2.87436732, 4.2468884, -0.41211761, 0.01461061);
pub const EASE_IN_CUBIC: CubicPoly = CubicPoly(1.03151375, -0.0926829, 0.07636252, 0.00467814);
pub const EASE_OUT_CUBIC: CubicPoly = CubicPoly(1.40088334, -3.61834191, 3.23457457, -0.00851172);
pub const EASE_IN_OUT_CUBIC: CubicPoly = CubicPoly(-4.64051239, 6.88921169, -1.36914323, 0.05766491);
pub const EASE_IN_QUART: CubicPoly = CubicPoly(2.83740293, -2.32635054, 0.58117863, -0.02060926);
pub const EASE_OUT_QUART: CubicPoly = CubicPoly(2.3564686, -5.09611614, 3.72860354, 0.04444882);
pub const EASE_IN_OUT_QUART: CubicPoly = CubicPoly(-6.36106047, 9.29423722, -2.13853761, 0.08042486);
pub const EASE_IN_QUINT: CubicPoly = CubicPoly(3.95219707, -4.34286945, 1.38332925, -0.0753837 );
pub const EASE_OUT_QUINT: CubicPoly = CubicPoly(3.19480389, -6.62369255, 4.45672564, 0.01313234);
pub const EASE_IN_OUT_QUINT: CubicPoly = CubicPoly(-7.26860583, 10.56178222, -2.54419582, 0.0938484 );
pub const EASE_IN_EXPO: CubicPoly = CubicPoly(5.11185986, -5.93778299, 1.88534478, -0.10342393);
pub const EASE_OUT_EXPO: CubicPoly = CubicPoly(4.1479451, -8.02277051, 4.9010832, 0.04570079);
pub const EASE_IN_OUT_EXPO: CubicPoly = CubicPoly(-8.03325348, 12.04988023, -3.30321709, 0.14329518);
pub const EASE_IN_CIRC: CubicPoly = CubicPoly(2.5479135, -2.5861972, 0.95807322, -0.05306623);
pub const EASE_OUT_CIRC: CubicPoly = CubicPoly(4.26695502, -7.87573726, 4.55294431, 0.15110784);
pub const EASE_IN_OUT_CIRC: CubicPoly = CubicPoly(-5.44327092, 7.90793653, -1.64317345, 0.0663622 );
pub const EASE_IN_BACK: CubicPoly = CubicPoly(1.93207231, -0.60271724, -0.30346557, -0.00439286);
pub const EASE_OUT_BACK: CubicPoly = CubicPoly(3.11091319, -6.87839232, 4.78347743, 0.02458581);
pub const EASE_IN_OUT_BACK: CubicPoly = CubicPoly(-9.64322882, 14.13277082, -3.76763802, 0.10749723);


