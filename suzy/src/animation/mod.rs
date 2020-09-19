/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Animations integrate with Suzy's watch system to interpolate values over
//! time.
//!
//! [`watch`](../widget/trait.WidgetInit.html#tymethod.watch)
//! closures which contain
//! [`Animation::apply`](struct.Animation.html#method.apply) will be re-run
//! every frame while the animation is in progress.
//!
//! ## Examples
//!
//! Animate a color to green with a speed of 1.
//!
//! ```rust
//! # use suzy::animation::Animation;
//! # use suzy::widget::*;
//! # use suzy::graphics::Color;
//! struct MyWidgetData {
//!     current_color: Color,
//!     animation: Animation<Color>,
//! }
//!
//! impl WidgetContent for MyWidgetData {
//!     fn init<I: WidgetInit<Self>>(mut init: I) {
//!         init.watch(|this, rect| {
//!             this.animation.set_speed(1.0);
//!             this.animation.animate_to(Color::GREEN);
//!         });
//!         init.watch(|this, rect| {
//!             this.animation.apply(&mut this.current_color);
//!             println!("current color value: {:x}", this.current_color);
//!         });
//!     }
//!
//!     // ...
//! #   fn children<R: WidgetChildReceiver>(&mut self, _receiver: R) {
//! #   }
//! #   fn graphics<R: WidgetGraphicReceiver>(&mut self, _receiver: R) {}
//! }
//! ```

use std::time::{
    Duration,
    Instant,
};

use crate::app::App;
use crate::platform::DefaultPlatform;
use crate::watch::{
    Watched,
};

mod easing;

pub use easing::{Easing, CubicPoly, EASES};

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

enum Speed<T> {
    Duration(Duration),
    Speed(f32, fn(&T, &T) -> f32),
}

impl<T> Speed<T> {
    fn duration(&self, start: &T, end: &T) -> Duration {
        match self {
            Self::Duration(value) => *value,
            Self::Speed(speed, dist_fn) => {
                Duration::from_secs_f32(dist_fn(start, end) / speed)
            }
        }
    }
}

impl<T> Default for Speed<T> {
    fn default() -> Self { Self::Duration(Duration::from_millis(500)) }
}


pub struct Animation<T> {
    speed: Speed<T>,
    start_value: Option<T>,
    current: Watched<Option<(Instant, T)>>,
    easing: Option<Box<dyn Easing>>,
}

impl<T> Animation<T> {
    pub fn new() -> Self { Self::default() }
}

impl<T> Default for Animation<T> {
    fn default() -> Self {
        Self {
            speed: Speed::default(),
            start_value: None,
            current: Watched::new(None),
            easing: None,
        }
    }
}

impl<T: LerpDistance> Animation<T> {
    /// The preferred way to set the speed of an animation is to use this
    /// method.
    ///
    /// This calculates the duration of the animation based on the "distance"
    /// between the starting and ending values.  If the type to be animated
    /// does not have a measurable "distance", use `set_duration`.
    pub fn set_speed(&mut self, speed: f32) {
        self.speed = Speed::Speed(speed, T::lerp_distance);
    }
}

impl Animation<(f32, f32)> {
    /// `LerpDistance` is intentionally not implemented for tuples, because
    /// tuples may be used to implement more than just a position.
    /// This function is the same as `set_speed`, but explicitly treats
    /// tuples as a position.
    pub fn set_position_speed(&mut self, speed: f32) {
        self.speed = Speed::Speed(speed, |a, b| {
            ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt()
        });
    }
}

impl<T> Animation<T> {
    /// Set the easing curve applied to this animation.
    pub fn set_ease(&mut self, easing: Box<dyn Easing>) {
        self.easing = Some(easing);
    }
}

impl<T: Lerp<Output = T>> Animation<T> {
    pub fn animate_to(&mut self, value: T) {
        let start_time = App::<DefaultPlatform>::time_unwatched();
        *self.current = Some((start_time, value));
        self.start_value = None;
    }

    pub fn apply(&mut self, target: &mut T) {
        let (start_time, end_value) = match &*self.current {
            Some(value) => value,
            None => return,
        };
        let start_value = match self.start_value {
            Some(ref start) => start,
            None => &*target,
        };
        let total_duration = self.speed.duration(start_value, end_value);
        let frame_time = App::<DefaultPlatform>::time();
        let elapsed = frame_time.duration_since(*start_time);
        let t = elapsed.as_secs_f32() / total_duration.as_secs_f32();
        let (t, at_end) = if t > 1.0 {
            (1.0, true)
        } else {
            (t, false)
        };
        let t = match self.easing {
            None => t,
            Some(ref easing) => easing.ease(t),
        };
        let value = T::lerp(start_value, end_value, t);
        let prev = std::mem::replace(target, value);
        if at_end {
            *self.current = None;
        } else if self.start_value.is_none() {
            self.start_value = Some(prev);
        }
    }

    /// Manually set the duration of the animation.
    ///
    /// Prefer `set_speed` over this function, when appropriate.
    pub fn set_duration(&mut self, duration: Duration) {
        self.speed = Speed::Duration(duration);
    }
}
