/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(missing_docs)]

use std::time;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::watch::{AtomicWatchedMeta, AtomicWatchedMetaTrigger, Watched};
use crate::platform::DefaultPlatform;
use crate::app::App;
use crate::widget::{WidgetInit, WidgetContent};

#[derive(Clone, Copy)]
struct NextFrame {
    init_frame: time::Instant,
}

impl NextFrame {
    fn new() -> Self {
        Self { init_frame: App::<DefaultPlatform>::time_unwatched() }
    }
}

impl Future for NextFrame {
    type Output = ();
    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let time = App::<DefaultPlatform>::time();
        if time == self.init_frame {
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}

#[derive(Clone, Copy)]
struct Timer {
    end_time: time::Instant,
}

impl Timer {
    fn new(duration: time::Duration) -> Self {
        let time = App::<DefaultPlatform>::time_unwatched();
        let end_time = time + duration;
        Self { end_time }
    }
}

impl Future for Timer {
    type Output = ();
    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let time = App::<DefaultPlatform>::time();
        if time < self.end_time {
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}

struct WatchedWaker {
    trigger: AtomicWatchedMetaTrigger,
}

impl WatchedWaker {
    fn from_meta(meta: &AtomicWatchedMeta) -> std::sync::Arc<Self> {
        std::sync::Arc::new(Self {
            trigger: meta.create_trigger(),
        })
    }
}

impl std::task::Wake for WatchedWaker {
    fn wake(self: std::sync::Arc<Self>) {
        self.trigger.trigger();
    }
}

enum State<T> {
    Inactive,
    Starting(T),
    Running(Pin<Box<dyn Future<Output=()>>>),
}

impl<T> Default for State<T> {
    fn default() -> Self {
        Self::Inactive
    }
}

#[derive(Default)]
pub struct Coroutine<T> {
    wake_meta: AtomicWatchedMeta,
    paused: Watched<bool>,
    state: Watched<State<T>>,
}

impl Coroutine<()> {
    pub async fn next_frame() {
        NextFrame::new().await
    }

    pub async fn delay(duration: time::Duration) {
        Timer::new(duration).await
    }

    pub async fn delay_secs(duration: f32) {
        Self::delay(time::Duration::from_secs_f32(duration)).await
    }

    pub async fn until<F, T>(func: F) -> T
    where
        F: Fn() -> Option<T>
    {
        loop {
            if let Some(result) = func() {
                break result;
            }
            Self::next_frame().await;
        }
    }
}

impl<T> Coroutine<T> {
    pub(crate) fn register<Get, Init, Fac, Wid, Plat, Fut>(getter: Get, init: &mut Init, factory: Fac)
    where
        Get: 'static + Fn(&mut Wid) -> &mut Self,
        Plat: crate::platform::RenderPlatform + ?Sized,
        Wid: WidgetContent<Plat> + ?Sized,
        Init: WidgetInit<Wid, Plat> + ?Sized,
        Fut: 'static + Future<Output=()>,
        Fac: 'static + Fn(T) -> Fut,
    {
        init.watch(move |wid_content, _rect| {
            let coroutine = getter(wid_content);
            let mut future = match std::mem::take(&mut *coroutine.state) {
                State::Inactive => return,
                State::Starting(args) => Box::pin(factory(args)),
                State::Running(fut) => fut,
            };
            if !*coroutine.paused {
                coroutine.wake_meta.watched();
                let waker = WatchedWaker::from_meta(&coroutine.wake_meta).into();
                let mut ctx = Context::from_waker(&waker);
                match future.as_mut().poll(&mut ctx) {
                    Poll::Pending => (),
                    Poll::Ready(()) => return,
                }
            }
            *coroutine.state = State::Running(future);
        });
    }

    pub fn is_paused(&self) -> bool {
        *self.paused
    }

    pub fn start(&mut self, args: T) {
        *self.paused = false;
        *self.state = State::Starting(args);
    }

    pub fn pause(&mut self) {
        *self.paused = true;
    }

    pub fn unpause_or_start(&mut self, args: T) {
        if *self.paused {
            *self.paused = false;
        } else {
            self.start(args);
        }
    }

    pub fn unpause_or_ignore(&mut self) {
        *self.paused = false;
    }
}
