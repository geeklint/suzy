
pub trait Lerp {
    type Output;
    fn lerp(from: Self, to: Self, t: f32) -> Self::Output;
}

impl Lerp for f32 {
    type Output = f32;

    fn lerp(from: f32, to: f32, t: f32) -> f32 {
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

    fn lerp(from: f64, to: f64, t: f32) -> f64 {
        let t = t as f64;
        let diff = to - from;
        if t <= 0.5 {
            from + diff * t
        } else {
            to - diff * (1.0 - t)
        }
    }
}

impl Lerp for &f32 {
    type Output = f32;
    fn lerp(from: &f32, to: &f32, t: f32) -> f32 {
        <f32 as Lerp>::lerp(*from, *to, t)
    }
}

impl Lerp for &f64 {
    type Output = f64;
    fn lerp(from: &f64, to: &f64, t: f32) -> f64 {
        <f64 as Lerp>::lerp(*from, *to, t)
    }
}

impl<T, U> Lerp for (T, U)
where T: Lerp, U: Lerp
{
    type Output = (<T as Lerp>::Output, <U as Lerp>::Output);
    fn lerp(from: (T, U), to: (T, U), t: f32) -> Self::Output {
        (Lerp::lerp(from.0, to.0, t), Lerp::lerp(from.1, to.1, t))
    }
}

pub trait Easing {
    fn ease(&self, t: f32) -> f32;
}


#[derive(Copy, Clone)]
pub struct CubicBezier(pub f32, pub f32, pub f32, pub f32);

impl Easing for CubicBezier {
    pub fn ease(&self, t: f32) -> f32 {
        let invt = 1.0 - t;
        let part0 = invt.powi(3) * self.0;
        let part1 = 3.0 * invt.powi(2) * t * self.1;
        let part2 = 3.0 * invt * t.powi(2) * self.2;
        let part3 = t.powi(3) * self.3;
        part0 + part1 + part2 + part3
    }
}

impl Default for CubicBezier {
    fn default() -> Self { EASE_LINEAR }
}

pub const EASE_LINEAR: CubicBezier = CubicBezier(0.0, 0.0, 1.0, 1.0);
pub const EASE_IN_SINE: CubicBezier = CubicBezier(0.47, 0.0, 0.745, 0.715);
pub const EASE_OUT_SINE: CubicBezier = CubicBezier(0.39, 0.575, 0.565, 1.0);
pub const EASE_IN_OUT_SINE: CubicBezier = CubicBezier(0.445, 0.05, 0.55, 0.95);
pub const EASE_IN_QUAD: CubicBezier = CubicBezier(0.55, 0.085, 0.68, 0.53);
pub const EASE_OUT_QUAD: CubicBezier = CubicBezier(0.25, 0.46, 0.45, 0.94);
pub const EASE_IN_OUT_QUAD: CubicBezier = CubicBezier(0.455, 0.03, 0.515, 0.955);
pub const EASE_IN_CUBIC: CubicBezier = CubicBezier(0.55, 0.055, 0.675, 0.19);
pub const EASE_OUT_CUBIC: CubicBezier = CubicBezier(0.215, 0.61, 0.355, 1.0);
pub const EASE_IN_OUT_CUBIC: CubicBezier = CubicBezier(0.645, 0.045, 0.355, 1.0);
pub const EASE_IN_QUART: CubicBezier = CubicBezier(0.895, 0.03, 0.685, 0.22);
pub const EASE_OUT_QUART: CubicBezier = CubicBezier(0.165, 0.84, 0.44, 1.0);
pub const EASE_IN_OUT_QUART: CubicBezier = CubicBezier(0.77, 0.0, 0.175, 1.0);
pub const EASE_IN_QUINT: CubicBezier = CubicBezier(0.755, 0.05, 0.855, 0.06);
pub const EASE_OUT_QUINT: CubicBezier = CubicBezier(0.23, 1.0, 0.32, 1.0);
pub const EASE_IN_OUT_QUINT: CubicBezier = CubicBezier(0.86, 0.0, 0.07, 1.0);
pub const EASE_IN_EXPO: CubicBezier = CubicBezier(0.95, 0.05, 0.795, 0.035);
pub const EASE_OUT_EXPO: CubicBezier = CubicBezier(0.19, 1.0, 0.22, 1.0);
pub const EASE_IN_OUT_EXPO: CubicBezier = CubicBezier(1.0, 0.0, 0.0, 1.0);
pub const EASE_IN_CIRC: CubicBezier = CubicBezier(0.6, 0.04, 0.98, 0.335);
pub const EASE_OUT_CIRC: CubicBezier = CubicBezier(0.075, 0.82, 0.165, 1.0);
pub const EASE_IN_OUT_CIRC: CubicBezier = CubicBezier(0.785, 0.135, 0.15, 0.86);
pub const EASE_IN_BACK: CubicBezier = CubicBezier(0.6, -0.28, 0.735, 0.045);
pub const EASE_OUT_BACK: CubicBezier = CubicBezier(0.175, 0.885, 0.32, 1.275);
pub const EASE_IN_OUT_BACK: CubicBezier = CubicBezier(0.68, -0.55, 0.265, 1.55);
