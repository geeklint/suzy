
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
