/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

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
    fn default() -> Self {
        CubicPoly(0.0, 0.0, 1.0, 0.0)
    }
}

/// Built-in easing function constants.
///
/// The Easing equations in this module are adapted from Robert Penner's
/// easing equations found on [this website](http://robertpenner.com/easing/).
pub mod eases {
    use std::f32::consts::PI;

    use super::{CubicPoly, Easing};

    trait ToBoxedEasing {
        fn to_boxed_easing(&self) -> Box<dyn Easing>;
    }

    impl<T> ToBoxedEasing for T
    where
        T: 'static + Clone + Easing + Send + Sync,
    {
        fn to_boxed_easing(&self) -> Box<dyn Easing> {
            Box::new(self.clone())
        }
    }

    #[derive(Clone, Copy, Debug)]
    struct EasingFn(fn(f32) -> f32);

    impl Easing for EasingFn {
        fn ease(&self, t: f32) -> f32 {
            (self.0)(t)
        }
    }

    /// A type which represents a built-in constant easing function.
    ///
    /// The method `get` returns a boxed trait object for use with
    /// `Animation::set_ease`.
    ///
    /// ```
    /// use suzy::animation::{Animation, eases};
    ///
    /// let mut anim: Animation<f32> = Animation::new();
    /// anim.set_ease(eases::EASE_LINEAR.get());
    /// ```
    #[derive(Clone, Copy)]
    pub struct BuiltInEasingFunction {
        inner: &'static (dyn ToBoxedEasing + Send + Sync),
    }

    impl BuiltInEasingFunction {
        /// Get a trait object representing this easing function.
        #[must_use]
        pub fn get(&self) -> Box<dyn Easing> {
            self.inner.to_boxed_easing()
        }
    }

    impl From<BuiltInEasingFunction> for Box<dyn Easing> {
        fn from(func: BuiltInEasingFunction) -> Box<dyn Easing> {
            func.get()
        }
    }

    /// Built-in easing function "LINEAR"
    /// ```
    /// use suzy::animation::{Animation, eases};
    ///
    /// let mut anim: Animation<f32> = Animation::new();
    /// anim.set_ease(eases::EASE_LINEAR.get());
    /// ```
    pub const EASE_LINEAR: BuiltInEasingFunction = BuiltInEasingFunction {
        inner: &EasingFn(|t| t),
    };

    /// Built-in easing function "IN_SINE"
    /// ```
    /// use suzy::animation::{Animation, eases};
    ///
    /// let mut anim: Animation<f32> = Animation::new();
    /// anim.set_ease(eases::EASE_IN_SINE.get());
    /// ```
    pub const EASE_IN_SINE: BuiltInEasingFunction = BuiltInEasingFunction {
        inner: &EasingFn(|t| 1.0 - (t * PI / 2.0).cos()),
    };

    /// Built-in easing function "OUT_SINE"
    /// ```
    /// use suzy::animation::{Animation, eases};
    ///
    /// let mut anim: Animation<f32> = Animation::new();
    /// anim.set_ease(eases::EASE_OUT_SINE.get());
    /// ```
    pub const EASE_OUT_SINE: BuiltInEasingFunction = BuiltInEasingFunction {
        inner: &EasingFn(|t| (t * PI / 2.0).sin()),
    };

    /// Built-in easing function "IN_OUT_SINE"
    /// ```
    /// use suzy::animation::{Animation, eases};
    ///
    /// let mut anim: Animation<f32> = Animation::new();
    /// anim.set_ease(eases::EASE_IN_OUT_SINE.get());
    /// ```
    pub const EASE_IN_OUT_SINE: BuiltInEasingFunction =
        BuiltInEasingFunction {
            inner: &EasingFn(|t| -0.5 * ((t * PI).cos() - 1.0)),
        };

    /// Built-in easing function "IN_QUAD"
    /// ```
    /// use suzy::animation::{Animation, eases};
    ///
    /// let mut anim: Animation<f32> = Animation::new();
    /// anim.set_ease(eases::EASE_IN_QUAD.get());
    /// ```
    pub const EASE_IN_QUAD: BuiltInEasingFunction = BuiltInEasingFunction {
        inner: &EasingFn(|t| t.powi(2)),
    };

    /// Built-in easing function "OUT_QUAD"
    /// ```
    /// use suzy::animation::{Animation, eases};
    ///
    /// let mut anim: Animation<f32> = Animation::new();
    /// anim.set_ease(eases::EASE_OUT_QUAD.get());
    /// ```
    pub const EASE_OUT_QUAD: BuiltInEasingFunction = BuiltInEasingFunction {
        inner: &EasingFn(|t| -t * (t - 2.0)),
    };

    /// Built-in easing function "IN_OUT_QUAD"
    /// ```
    /// use suzy::animation::{Animation, eases};
    ///
    /// let mut anim: Animation<f32> = Animation::new();
    /// anim.set_ease(eases::EASE_IN_OUT_QUAD.get());
    /// ```
    pub const EASE_IN_OUT_QUAD: BuiltInEasingFunction =
        BuiltInEasingFunction {
            inner: &EasingFn(|t| {
                if t < 0.5 {
                    2.0 * t.powi(2)
                } else {
                    (-2.0 * t.powi(2)) + (4.0 * t) - 1.0
                }
            }),
        };

    /// Built-in easing function "IN_CUBIC"
    /// ```
    /// use suzy::animation::{Animation, eases};
    ///
    /// let mut anim: Animation<f32> = Animation::new();
    /// anim.set_ease(eases::EASE_IN_CUBIC.get());
    /// ```
    pub const EASE_IN_CUBIC: BuiltInEasingFunction = BuiltInEasingFunction {
        inner: &EasingFn(|t| t.powi(3)),
    };

    /// Built-in easing function "OUT_CUBIC"
    /// ```
    /// use suzy::animation::{Animation, eases};
    ///
    /// let mut anim: Animation<f32> = Animation::new();
    /// anim.set_ease(eases::EASE_OUT_CUBIC.get());
    /// ```
    pub const EASE_OUT_CUBIC: BuiltInEasingFunction = BuiltInEasingFunction {
        inner: &EasingFn(|t| (t - 1.0).powi(3) + 1.0),
    };

    /// Built-in easing function "IN_OUT_CUBIC"
    /// ```
    /// use suzy::animation::{Animation, eases};
    ///
    /// let mut anim: Animation<f32> = Animation::new();
    /// anim.set_ease(eases::EASE_IN_OUT_CUBIC.get());
    /// ```
    pub const EASE_IN_OUT_CUBIC: BuiltInEasingFunction =
        BuiltInEasingFunction {
            inner: &EasingFn(|t| {
                if t < 0.5 {
                    4.0 * t.powi(3)
                } else {
                    0.5 * ((2.0 * t - 2.0).powi(3) + 2.0)
                }
            }),
        };

    /// Built-in easing function "IN_QUART"
    /// ```
    /// use suzy::animation::{Animation, eases};
    ///
    /// let mut anim: Animation<f32> = Animation::new();
    /// anim.set_ease(eases::EASE_IN_QUART.get());
    /// ```
    pub const EASE_IN_QUART: BuiltInEasingFunction = BuiltInEasingFunction {
        inner: &EasingFn(|t| t.powi(4)),
    };

    /// Built-in easing function "OUT_QUART"
    /// ```
    /// use suzy::animation::{Animation, eases};
    ///
    /// let mut anim: Animation<f32> = Animation::new();
    /// anim.set_ease(eases::EASE_OUT_QUART.get());
    /// ```
    pub const EASE_OUT_QUART: BuiltInEasingFunction = BuiltInEasingFunction {
        inner: &EasingFn(|t| -((t - 1.0).powi(4) - 1.0)),
    };

    /// Built-in easing function "IN_OUT_QUART"
    /// ```
    /// use suzy::animation::{Animation, eases};
    ///
    /// let mut anim: Animation<f32> = Animation::new();
    /// anim.set_ease(eases::EASE_IN_OUT_QUART.get());
    /// ```
    pub const EASE_IN_OUT_QUART: BuiltInEasingFunction =
        BuiltInEasingFunction {
            inner: &EasingFn(|t| {
                if t < 0.5 {
                    8.0 * t.powi(4)
                } else {
                    -0.5 * ((2.0 * t - 2.0).powi(4) - 2.0)
                }
            }),
        };

    /// Built-in easing function "IN_QUINT"
    /// ```
    /// use suzy::animation::{Animation, eases};
    ///
    /// let mut anim: Animation<f32> = Animation::new();
    /// anim.set_ease(eases::EASE_IN_QUINT.get());
    /// ```
    pub const EASE_IN_QUINT: BuiltInEasingFunction = BuiltInEasingFunction {
        inner: &EasingFn(|t| t.powi(5)),
    };

    /// Built-in easing function "OUT_QUINT"
    /// ```
    /// use suzy::animation::{Animation, eases};
    ///
    /// let mut anim: Animation<f32> = Animation::new();
    /// anim.set_ease(eases::EASE_OUT_QUINT.get());
    /// ```
    pub const EASE_OUT_QUINT: BuiltInEasingFunction = BuiltInEasingFunction {
        inner: &EasingFn(|t| (t - 1.0).powi(5) + 1.0),
    };

    /// Built-in easing function "IN_OUT_QUINT"
    /// ```
    /// use suzy::animation::{Animation, eases};
    ///
    /// let mut anim: Animation<f32> = Animation::new();
    /// anim.set_ease(eases::EASE_IN_OUT_QUINT.get());
    /// ```
    pub const EASE_IN_OUT_QUINT: BuiltInEasingFunction =
        BuiltInEasingFunction {
            inner: &EasingFn(|t| {
                if t < 0.5 {
                    16.0 * t.powi(5)
                } else {
                    0.5 * ((2.0 * t - 2.0).powi(5) + 2.0)
                }
            }),
        };

    /// Built-in easing function "IN_EXPO"
    /// ```
    /// use suzy::animation::{Animation, eases};
    ///
    /// let mut anim: Animation<f32> = Animation::new();
    /// anim.set_ease(eases::EASE_IN_EXPO.get());
    /// ```
    pub const EASE_IN_EXPO: BuiltInEasingFunction = BuiltInEasingFunction {
        inner: &EasingFn(|t| {
            if t == 0.0 {
                0.0
            } else {
                (10.0 * (t - 1.0)).exp2()
            }
        }),
    };

    /// Built-in easing function "OUT_EXPO"
    /// ```
    /// use suzy::animation::{Animation, eases};
    ///
    /// let mut anim: Animation<f32> = Animation::new();
    /// anim.set_ease(eases::EASE_OUT_EXPO.get());
    /// ```
    pub const EASE_OUT_EXPO: BuiltInEasingFunction = BuiltInEasingFunction {
        inner: &EasingFn(|t| {
            #[allow(clippy::float_cmp)]
            if t == 1.0 {
                1.0
            } else {
                1.0 - (-10.0 * t).exp2()
            }
        }),
    };

    /// Built-in easing function "IN_OUT_EXPO"
    /// ```
    /// use suzy::animation::{Animation, eases};
    ///
    /// let mut anim: Animation<f32> = Animation::new();
    /// anim.set_ease(eases::EASE_IN_OUT_EXPO.get());
    /// ```
    pub const EASE_IN_OUT_EXPO: BuiltInEasingFunction =
        BuiltInEasingFunction {
            inner: &EasingFn(|t| {
                #[allow(clippy::float_cmp)]
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else if t < 0.5 {
                    0.5 * (10.0 * (t - 1.0)).exp2()
                } else {
                    0.5 * (2.0 - (-10.0 * (t - 1.0)).exp2())
                }
            }),
        };

    /// Built-in easing function "IN_CIRC"
    /// ```
    /// use suzy::animation::{Animation, eases};
    ///
    /// let mut anim: Animation<f32> = Animation::new();
    /// anim.set_ease(eases::EASE_IN_CIRC.get());
    /// ```
    pub const EASE_IN_CIRC: BuiltInEasingFunction = BuiltInEasingFunction {
        inner: &EasingFn(|t| 1.0 - (1.0 - t.powi(2)).sqrt()),
    };

    /// Built-in easing function "OUT_CIRC"
    /// ```
    /// use suzy::animation::{Animation, eases};
    ///
    /// let mut anim: Animation<f32> = Animation::new();
    /// anim.set_ease(eases::EASE_OUT_CIRC.get());
    /// ```
    pub const EASE_OUT_CIRC: BuiltInEasingFunction = BuiltInEasingFunction {
        inner: &EasingFn(|t| (1.0 - (t - 1.0).powi(2)).sqrt()),
    };

    /// Built-in easing function "IN_OUT_CIRC"
    /// ```
    /// use suzy::animation::{Animation, eases};
    ///
    /// let mut anim: Animation<f32> = Animation::new();
    /// anim.set_ease(eases::EASE_IN_OUT_CIRC.get());
    /// ```
    pub const EASE_IN_OUT_CIRC: BuiltInEasingFunction =
        BuiltInEasingFunction {
            inner: &EasingFn(|t| {
                if t < 0.5 {
                    0.5 * (1.0 - (1.0 - 4.0 * t.powi(2)).sqrt())
                } else {
                    0.5 * (1.0 + (1.0 - (2.0 * t - 2.0).powi(2)).sqrt())
                }
            }),
        };

    /// Built-in easing function "IN_BACK"
    /// ```
    /// use suzy::animation::{Animation, eases};
    ///
    /// let mut anim: Animation<f32> = Animation::new();
    /// anim.set_ease(eases::EASE_IN_BACK.get());
    /// ```
    pub const EASE_IN_BACK: BuiltInEasingFunction = BuiltInEasingFunction {
        // TODO: Use real function here instead of CubicPoly
        inner: &CubicPoly(1.9320723, -0.6027172, -0.3034656, -0.0043929),
    };

    /// Built-in easing function "OUT_BACK"
    /// ```
    /// use suzy::animation::{Animation, eases};
    ///
    /// let mut anim: Animation<f32> = Animation::new();
    /// anim.set_ease(eases::EASE_OUT_BACK.get());
    /// ```
    pub const EASE_OUT_BACK: BuiltInEasingFunction = BuiltInEasingFunction {
        // TODO: Use real function here instead of CubicPoly
        inner: &CubicPoly(3.110913, -6.878392, 4.783477, 0.0245858),
    };

    /// Built-in easing function "IN_OUT_BACK"
    /// ```
    /// use suzy::animation::{Animation, eases};
    ///
    /// let mut anim: Animation<f32> = Animation::new();
    /// anim.set_ease(eases::EASE_IN_OUT_BACK.get());
    /// ```
    pub const EASE_IN_OUT_BACK: BuiltInEasingFunction =
        BuiltInEasingFunction {
            // TODO: Use real function here instead of CubicPoly
            inner: &CubicPoly(-9.643229, 14.132771, -3.767638, 0.1074972),
        };

    fn bounce(t: f32) -> f32 {
        if t < (1.0 / 2.75) {
            7.5625 * t.powi(2)
        } else if t < (2.0 / 2.75) {
            7.5625 * (t - (1.5 / 2.75)).powi(2) + 0.75
        } else if t < (2.5 / 2.75) {
            7.5625 * (t - (2.25 / 2.75)).powi(2) + 0.9375
        } else {
            7.5625 * (t - (2.625 / 2.75)).powi(2) + 0.984375
        }
    }

    /// Built-in easing function "IN_BOUNCE"
    /// ```
    /// use suzy::animation::{Animation, eases};
    ///
    /// let mut anim: Animation<f32> = Animation::new();
    /// anim.set_ease(eases::EASE_IN_BOUNCE.get());
    /// ```
    pub const EASE_IN_BOUNCE: BuiltInEasingFunction = BuiltInEasingFunction {
        inner: &EasingFn(|t| 1.0 - bounce(1.0 - t)),
    };

    /// Built-in easing function "OUT_BOUNCE"
    /// ```
    /// use suzy::animation::{Animation, eases};
    ///
    /// let mut anim: Animation<f32> = Animation::new();
    /// anim.set_ease(eases::EASE_OUT_BOUNCE.get());
    /// ```
    pub const EASE_OUT_BOUNCE: BuiltInEasingFunction = BuiltInEasingFunction {
        inner: &EasingFn(bounce),
    };

    /// Built-in easing function "IN_OUT_BOUNCE"
    /// ```
    /// use suzy::animation::{Animation, eases};
    ///
    /// let mut anim: Animation<f32> = Animation::new();
    /// anim.set_ease(eases::EASE_IN_OUT_BOUNCE.get());
    /// ```
    pub const EASE_IN_OUT_BOUNCE: BuiltInEasingFunction =
        BuiltInEasingFunction {
            inner: &EasingFn(|t| {
                if t < 0.5 {
                    0.5 * (1.0 - bounce(1.0 - 2.0 * t))
                } else {
                    0.5 + 0.5 * bounce(2.0 * t - 1.0)
                }
            }),
        };
}
