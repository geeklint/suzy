/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

//! Animations integrate with Suzy's watch system to interpolate values over
//! time.
//!
//! [`watch`](crate::widget::Desc::watch)
//! closures which contain
//! [`Animation::apply`] will be re-run
//! every frame while the animation is in progress.
//!
//! ## Examples
//!
//! Animate a color to green with a speed of 1.
//!
#![cfg_attr(
    feature = "platform-opengl",
    doc = r#"```rust
# use suzy::{animation::Animation, widget, graphics::Color};
struct MyWidgetData {
    current_color: Color,
    animation: Animation<Color>,
}

impl widget::Content for MyWidgetData {
    fn desc(mut desc: impl widget::Desc<Self>) {
        desc.watch(|this, rect| {
            this.animation.set_speed(1.0);
            this.animation.animate_to(Color::GREEN);
        });
        desc.watch(|this, rect| {
            this.animation.apply(&mut this.current_color);
            println!("current color value: {:x}", this.current_color);
        });
    }
}
```"#
)]

use std::time::{Duration, Instant};

use crate::{app, watch::Watched};

mod easing;

pub use easing::{eases, CubicPoly, Easing};

/// A trait with describes how to linearly interpolate between two values
/// of the type.
///
/// A trivial example with floats:
///
/// ```rust
/// # use suzy::animation::Lerp;
/// let (start, end) = (2.0, 9.0);
/// let value = f32::lerp(&start, &end, 0.7);
/// // 70% of the distance between 2 and 9 is 6.9.
/// assert!((value - 6.9) < f32::EPSILON);
/// ```
pub trait Lerp {
    /// The result of the linear interpolation, typically the same type (Self)
    type Output;

    /// Linearly interpolate between two values.
    ///
    /// A t-value of 0.0 should generally return `from`,
    /// and a t-value of 1.0 should generally return `to`.
    fn lerp(from: &Self, to: &Self, t: f32) -> Self::Output;
}

impl Lerp for f32 {
    type Output = f32;

    fn lerp(from: &f32, to: &f32, t: f32) -> f32 {
        // version with an if-statement guarentees we hit the endpoints exactly
        // at t == 0.0 and t == 1.0 respectively
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
        let t = f64::from(t);
        let diff = to - from;
        if t <= 0.5_f64 {
            from + diff * t
        } else {
            to - diff * (1.0_f64 - t)
        }
    }
}

/// A trait for calculating the distance between two values, for the
/// purposes of linear interpolation.
pub trait LerpDistance {
    /// Calculate the distance between two values
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
    fn default() -> Self {
        Self::Duration(Duration::from_millis(500))
    }
}

#[derive(Clone, Copy, Debug)]
enum RefTime {
    StartTime(Instant),
    FinishNow,
}

/// An instance of an animation.
///
/// See the [module-level documentation](self) for more details.
pub struct Animation<T> {
    speed: Speed<T>,
    start_value: Option<T>,
    current: Watched<Option<(RefTime, T)>>,
    easing: Option<Box<dyn Easing>>,
    on_complete: crate::watch::WatchedQueue<'static, ()>,
}

impl<T> Animation<T> {
    /// Create a new animation.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl<T> Default for Animation<T> {
    fn default() -> Self {
        Self {
            speed: Speed::default(),
            start_value: None,
            current: Watched::new(None),
            easing: None,
            on_complete: crate::watch::WatchedQueue::default(),
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

impl Animation<[f32; 2]> {
    pub fn set_position_speed(&mut self, speed: f32) {
        self.speed = Speed::Speed(speed, |a, b| {
            let [[ax, ay], [bx, by]] = [a, b];
            ((bx - ax).powi(2) + (by - ay).powi(2)).sqrt()
        });
    }
}

impl<T> Animation<T> {
    /// Set the easing curve applied to this animation.
    pub fn set_ease(&mut self, easing: Box<dyn Easing>) {
        self.easing = Some(easing);
    }

    /// Returns true if the animation is currently running
    pub fn running(&self) -> bool {
        self.current.is_some()
    }

    /// The provided function is run when the animation completes
    pub fn on_complete<F: FnOnce()>(&self, f: F) {
        crate::watch::WatchArg::try_with_current(|arg| {
            self.on_complete.handle_item(arg, |()| f());
        });
    }

    /// Proceed as though the remaining time in the animation has passed,
    /// causing the next update to apply the final value and notify
    /// [`Self::on_complete`].
    ///
    /// Does nothing if the animation is not currently running.
    pub fn finish_now(&mut self) {
        if let Some((ref_time, _)) = self.current.as_mut() {
            *ref_time = RefTime::FinishNow;
        }
    }

    /// Cancel the animation, preventing it from further updates, leaving
    /// any output state as-is.
    ///
    /// Does nothing if the animation is not currently running.
    pub fn cancel(&mut self) {
        *self.current = None;
    }

    /// Reset the animation to the starting value.
    ///
    /// If an animation is in-progress, this 'rewinds' the animation, so that
    /// the next update will apply the value that was present at the start.
    /// The animation then stops running.
    ///
    /// Does nothing if the animation is not currently running.
    pub fn reset(&mut self)
    where
        T: Copy,
    {
        if let Some((ref_time, target_value)) = self.current.as_mut() {
            if let Some(start_value) = self.start_value {
                *ref_time = RefTime::FinishNow;
                *target_value = start_value;
            } else {
                *self.current = None;
            }
        }
    }
}

impl<T: Lerp<Output = T>> Animation<T> {
    /// Start the animation, with a specified value to interpolate towards.
    pub fn animate_to(&mut self, value: T) {
        let start_time = app::time_unwatched();
        *self.current = Some((RefTime::StartTime(start_time), value));
        self.start_value = None;
    }

    /// This is the primary output of the animation.  A
    /// [`watch`](crate::widget::Desc::watch)
    /// closure which calls this method will be re-run every frame with
    /// an interpolated value while the animation is in-progress.
    pub fn apply(&mut self, target: &mut T) {
        let Some((ref_time, end_value)) = &*self.current else {
            return;
        };
        let start_value = match self.start_value {
            Some(ref start) => start,
            None => &*target,
        };
        let total_duration =
            self.speed.duration(start_value, end_value).as_secs_f32();
        let elapsed = match ref_time {
            RefTime::StartTime(start_time) => {
                app::time().duration_since(*start_time).as_secs_f32()
            }
            RefTime::FinishNow => total_duration * 2.0,
        };
        let t = elapsed / total_duration;
        let (t, at_end) = if t > 1.0 { (1.0, true) } else { (t, false) };
        let t = match self.easing {
            None => t,
            Some(ref easing) => easing.ease(t),
        };
        let value = T::lerp(start_value, end_value, t);
        let prev = std::mem::replace(target, value);
        if at_end {
            *self.current = None;
            self.on_complete.push_external(());
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
