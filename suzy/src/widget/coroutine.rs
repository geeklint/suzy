/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time;

use crate::app::App;
use crate::platform::DefaultPlatform;
use crate::watch::{AtomicWatchedMeta, AtomicWatchedMetaTrigger, Watched};
use crate::widget::WidgetInit;

#[derive(Clone, Copy)]
struct NextFrame {
    init_frame: time::Instant,
}

impl NextFrame {
    fn new() -> Self {
        Self {
            init_frame: App::<DefaultPlatform>::time_unwatched(),
        }
    }
}

impl Future for NextFrame {
    type Output = ();
    fn poll(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Self::Output> {
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
    fn poll(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Self::Output> {
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
    Running(Pin<Box<dyn Future<Output = ()>>>),
}

impl<T> Default for State<T> {
    fn default() -> Self {
        Self::Inactive
    }
}

/// A Coroutine stores the state of a Rust Future which is run within
/// Suzy's watch system.  This can be used to sequence events over multiple
/// frames.
///
/// ```rust,no_run
/// # use suzy::dims::{Rect, SimplePadding2d};
/// # use suzy::widget::{self, *};
/// # use suzy::widgets::Button;
/// struct Root {
///     button: Button,
///     coroutine: Coroutine<()>,
/// }
///
/// impl widget::Content for Root {
///     fn init(mut init: impl WidgetInit<Self>) {
///         init.watch(|this, _rect| {
///             if let Some(()) = this.button.on_click() {
///                 this.coroutine.start(());
///             }
///         });
///         init.register_coroutine(
///             |this| &mut this.coroutine,
///             |()| async {
///                 Coroutine::delay_secs(5.0).await;
///                 println!("Button clicked after delay");
///             },
///         );
///     }
/// #    fn children(mut receiver: impl WidgetChildReceiver<Self>) {
/// #        receiver.child(|this| &mut this.button);
/// #    }
/// #    fn graphics(_receiver: impl WidgetGraphicReceiver<Self>) {
/// #    }
/// }
/// ```
#[derive(Default)]
pub struct Coroutine<T> {
    wake_meta: AtomicWatchedMeta,
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
    pub(crate) fn register<Get, Init, Fac, Wid, Plat, Fut>(
        getter: Get,
        init: &mut Init,
        factory: Fac,
    ) where
        Get: 'static + Fn(&mut Wid) -> &mut Self,
        Plat: crate::platform::RenderPlatform + ?Sized,
        Wid: super::Content<Plat> + ?Sized,
        Init: WidgetInit<Wid, Plat> + ?Sized,
        Fut: 'static + Future<Output = ()>,
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
