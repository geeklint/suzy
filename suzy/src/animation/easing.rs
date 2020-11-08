/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

/// A trait which represents an easing function.
///
/// Easing functions transform the time value for an animation.  This usually
/// corosponds to a visual acceleration at the start of an animation or a
/// visual deceleration at the end of an animation, but may involve other
/// effects.
pub trait Easing {
    /// Transform the time value according to this easing definition.
    fn ease(&self, t: f32) -> f32;
}


/// An easing function implementation based on cubic polynomials.
///
/// Given the four parameters, (a, b, c, d), compute the easing to be
/// `a*t³ + b*t² + c*t + d`.
#[derive(Copy, Clone)]
pub struct CubicPoly(
    /// The cubic coefficient.
    pub f32,
    /// The quadratic coefficient.
    pub f32,
    /// The linear coefficient.
    pub f32,
    /// The constant coefficient.
    pub f32,
);

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

/// Built-in easing function constants.
pub mod eases {
    use super::CubicPoly;

    /// Built-in easing function "LINEAR"
    pub const EASE_LINEAR: CubicPoly = CubicPoly(0.0, 0.0, 1.0, 0.0);
    /// Built-in easing function "IN_SINE"
    pub const EASE_IN_SINE: CubicPoly = CubicPoly(-0.7800867, 2.037575, -0.2556811, 0.0134600);
    /// Built-in easing function "OUT_SINE"
    pub const EASE_OUT_SINE: CubicPoly = CubicPoly(-0.6614722, 0.0244138, 1.632126, -0.0100045);
    /// Built-in easing function "IN_OUT_SINE"
    pub const EASE_IN_OUT_SINE: CubicPoly = CubicPoly(-2.604746, 3.8931036, -0.3192672, 0.0146106);
    /// Built-in easing function "IN_QUAD"
    pub const EASE_IN_QUAD: CubicPoly = CubicPoly(-0.2707754, 1.4249408, -0.1509537, 0.0178648);
    /// Built-in easing function "OUT_QUAD"
    pub const EASE_OUT_QUAD: CubicPoly = CubicPoly(0.184935, -1.392142, 2.213768, -0.0145435);
    /// Built-in easing function "IN_OUT_QUAD"
    pub const EASE_IN_OUT_QUAD: CubicPoly = CubicPoly(-2.874367, 4.246888, -0.4121176, 0.0146106);
    /// Built-in easing function "IN_CUBIC"
    pub const EASE_IN_CUBIC: CubicPoly = CubicPoly(1.0315138, -0.0926829, 0.0763625, 0.0046781);
    /// Built-in easing function "OUT_CUBIC"
    pub const EASE_OUT_CUBIC: CubicPoly = CubicPoly(1.4008833, -3.618342, 3.2345746, -0.0085117);
    /// Built-in easing function "IN_OUT_CUBIC"
    pub const EASE_IN_OUT_CUBIC: CubicPoly = CubicPoly(-4.640512, 6.8892117, -1.3691432, 0.0576649);
    /// Built-in easing function "IN_QUART"
    pub const EASE_IN_QUART: CubicPoly = CubicPoly(2.837403, -2.3263505, 0.5811786, -0.0206093);
    /// Built-in easing function "OUT_QUART"
    pub const EASE_OUT_QUART: CubicPoly = CubicPoly(2.356469, -5.096116, 3.728604, 0.0444488);
    /// Built-in easing function "IN_OUT_QUART"
    pub const EASE_IN_OUT_QUART: CubicPoly = CubicPoly(-6.361061, 9.294237, -2.1385376, 0.0804249);
    /// Built-in easing function "IN_QUINT"
    pub const EASE_IN_QUINT: CubicPoly = CubicPoly(3.952197, -4.34287, 1.3833293, -0.0753837 );
    /// Built-in easing function "OUT_QUINT"
    pub const EASE_OUT_QUINT: CubicPoly = CubicPoly(3.194804, -6.623693, 4.4567256, 0.0131323);
    /// Built-in easing function "IN_OUT_QUINT"
    pub const EASE_IN_OUT_QUINT: CubicPoly = CubicPoly(-7.268606, 10.561782, -2.544196, 0.0938484 );
    /// Built-in easing function "IN_EXPO"
    pub const EASE_IN_EXPO: CubicPoly = CubicPoly(5.11186, -5.937783, 1.885345, -0.1034239);
    /// Built-in easing function "OUT_EXPO"
    pub const EASE_OUT_EXPO: CubicPoly = CubicPoly(4.147945, -8.022771, 4.901083, 0.0457008);
    /// Built-in easing function "IN_OUT_EXPO"
    pub const EASE_IN_OUT_EXPO: CubicPoly = CubicPoly(-8.033254, 12.04988, -3.303217, 0.1432952);
    /// Built-in easing function "IN_CIRC"
    pub const EASE_IN_CIRC: CubicPoly = CubicPoly(2.547914, -2.586197, 0.9580732, -0.0530662);
    /// Built-in easing function "OUT_CIRC"
    pub const EASE_OUT_CIRC: CubicPoly = CubicPoly(4.266955, -7.875737, 4.552944, 0.1511078);
    /// Built-in easing function "IN_OUT_CIRC"
    pub const EASE_IN_OUT_CIRC: CubicPoly = CubicPoly(-5.443271, 7.907937, -1.6431735, 0.0663622 );
    /// Built-in easing function "IN_BACK"
    pub const EASE_IN_BACK: CubicPoly = CubicPoly(1.9320723, -0.6027172, -0.3034656, -0.0043929);
    /// Built-in easing function "OUT_BACK"
    pub const EASE_OUT_BACK: CubicPoly = CubicPoly(3.110913, -6.878392, 4.783477, 0.0245858);
    /// Built-in easing function "IN_OUT_BACK"
    pub const EASE_IN_OUT_BACK: CubicPoly = CubicPoly(-9.643229, 14.132771, -3.767638, 0.1074972);
}
