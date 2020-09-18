/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Animations integrate with Suzy's watch system to interpolate values over
//! time.

use std::time::{
    Duration,
    Instant,
};

use crate::app::App;
use crate::platform::DefaultPlatform;
use crate::watch::{
    Watched,
};

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
    pub fn set_speed(&mut self, speed: f32) {
        self.speed = Speed::Speed(speed, T::lerp_distance);
    }
}

impl Animation<(f32, f32)> {
    pub fn set_position_speed(&mut self, speed: f32) {
        self.speed = Speed::Speed(speed, |a, b| {
            ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt()
        });
    }
}

impl<T> Animation<T> {
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

    pub fn set_duration(&mut self, duration: Duration) {
        self.speed = Speed::Duration(duration);
    }
}
