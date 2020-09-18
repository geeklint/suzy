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
use crate::math::{
    Lerp,
    LerpDistance,
    Easing,
};
use crate::watch::{
    Watched,
};

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
