/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time,
};

use crate::{
    app,
    watch::{self, Watched},
};

#[derive(Clone, Copy)]
struct NextFrame {
    init_frame: time::Instant,
}

impl NextFrame {
    fn new() -> Self {
        Self {
            init_frame: app::time_unwatched(),
        }
    }
}

impl Future for NextFrame {
    type Output = ();
    fn poll(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Self::Output> {
        let time = app::time();
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
        let time = app::time_unwatched();
        let end_time = time + duration;
        Self { end_time }
    }
}

impl Future for Timer {
    type Output = ();
    fn poll(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Self::Output> {
        let time = app::time();
        if time < self.end_time {
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}

struct WatchedWaker {
    trigger: watch::SyncTrigger,
}

impl WatchedWaker {
    fn from_meta(meta: &watch::SyncWatchedMeta) -> std::sync::Arc<Self> {
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

#[derive(Default)]
enum State<T> {
    #[default]
    Inactive,
    Starting(T),
    Running(Pin<Box<dyn Future<Output = ()>>>),
}

/// A Coroutine stores the state of a Rust Future which is run within
/// Suzy's watch system.  This can be used to sequence events over multiple
/// frames.
///
#[cfg_attr(
    feature = "platform-opengl",
    doc = r#"```rust,no_run
# use suzy::dims::Rect;
# use suzy::widget::{self, *};
# use suzy::widgets::Button;
struct Root {
    button: Button,
    coroutine: Coroutine<()>,
}

impl widget::Content for Root {
    fn desc(mut desc: impl widget::Desc<Self>) {
        desc.watch(|this, _rect| {
            let Self { button, coroutine } = this;
            button.on_click(|| {
                coroutine.start(());
            });
        });
        desc.register_coroutine(
            |this| &mut this.coroutine,
            |()| async {
                Coroutine::delay_secs(5.0).await;
                println!("Button clicked after delay");
            },
        );
        desc.child(|this| &mut this.button);
    }
}
```"#
)]
#[derive(Default)]
pub struct Coroutine<T> {
    wake_meta: watch::SyncWatchedMeta,
    paused: Watched<bool>,
    state: Watched<State<T>>,
}

impl Coroutine<()> {
    /// A utility future which waits a single frame before continuing
    pub async fn next_frame() {
        NextFrame::new().await
    }

    /// A utility future which waits for the specified duration before
    /// continuing
    pub async fn delay(duration: time::Duration) {
        Timer::new(duration).await
    }

    /// A utility future which waits for the specified seconds before
    /// continuing
    pub async fn delay_secs(duration: f32) {
        Self::delay(time::Duration::from_secs_f32(duration)).await
    }

    /// A utility future which calls the provided closure every frame until it
    /// returns Some.
    pub async fn until<F, T>(func: F) -> T
    where
        F: Fn() -> Option<T>,
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
    pub(crate) fn register<Get, Desc, Fac, Wid, Plat, Fut>(
        getter: Get,
        desc: &mut Desc,
        factory: Fac,
    ) where
        Get: 'static + Fn(&mut Wid) -> &mut Self,
        Wid: ?Sized,
        Desc: super::Desc<Wid, Plat> + ?Sized,
        Fut: 'static + Future<Output = ()>,
        Fac: 'static + Fn(T) -> Fut,
    {
        desc.watch(move |wid_content, _rect| {
            let coroutine = getter(wid_content);
            let mut future = match std::mem::take(&mut *coroutine.state) {
                State::Inactive => return,
                State::Starting(args) => Box::pin(factory(args)),
                State::Running(fut) => fut,
            };
            if !*coroutine.paused {
                watch::WatchArg::try_with_current(|arg| {
                    coroutine.wake_meta.watched(arg);
                });
                let waker =
                    WatchedWaker::from_meta(&coroutine.wake_meta).into();
                let mut ctx = Context::from_waker(&waker);
                match future.as_mut().poll(&mut ctx) {
                    Poll::Pending => (),
                    Poll::Ready(()) => return,
                }
            }
            *coroutine.state = State::Running(future);
        });
    }

    /// Start the coroutine, using the provided arguments to create the
    /// future.  This will restart the couroutine, canceling one currently
    /// in-progress.
    pub fn start(&mut self, args: T) {
        *self.paused = false;
        *self.state = State::Starting(args);
    }

    /// Cancel a coroutine, preventing it from executing further.
    pub fn stop(&mut self) {
        *self.state = State::Inactive;
    }

    /// Pause a coroutine, preventing execution until unpaused.
    pub fn pause(&mut self) {
        *self.paused = true;
    }

    /// Unpaused a paused coroutine, or start it if it is not running.
    /// Does nothing if the coroutine is currently running and not paused.
    pub fn unpause_or_start(&mut self, args: T) {
        if matches!(*self.state, State::Inactive) {
            self.start(args);
        } else {
            *self.paused = false;
        }
    }

    /// Unpause a paused coroutine, or do nothing if one is not currently
    /// running.
    pub fn unpause_or_ignore(&mut self) {
        *self.paused = false;
    }

    /// Return true if the coroutine is stopped, finished, or never started.
    pub fn is_stopped(&self) -> bool {
        matches!(*self.state, State::Inactive)
    }

    /// Return true if the coroutine is currently paused.
    pub fn is_paused(&self) -> bool {
        *self.paused
    }
}
