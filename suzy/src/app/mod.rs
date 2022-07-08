/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

//! An App describes the context in which widgets exist.
//!
//! Apps have an associated window and "root" widgets, which are assigned
//! to fill the whole window area.

use std::{cell::RefCell, collections::HashMap, rc::Rc, time};

use drying_paint::{WatchContext, Watched};

use crate::dims::{Dim, Rect, SimplePadding2d, SimpleRect};
use crate::platform::{DefaultPlatform, Event, EventLoopState, Platform};
use crate::pointer::{PointerEvent, PointerEventData, PointerId};
use crate::widget::{self, AnonWidget, RootWidget, UniqueHandleId, Widget};
use crate::window;
use window::{Window, WindowEvent, WindowSettings};

mod builder;
mod tester;
mod values;

pub use builder::AppBuilder;
pub use tester::AppTesterInterface;
pub(crate) use values::{get_cell_size, AppValues};

type RootHolder<P> = Rc<RefCell<RootWidget<dyn AnonWidget<P>, P>>>;

/// A type which contains the context in which widgets run.
///
/// See the [module-level documentation](./index.html) for more details.
pub struct App<P = DefaultPlatform>
where
    P: Platform,
{
    platform: Option<P>,
    watch_ctx: WatchContext<'static>,
    window: P::Window,
    roots: Vec<RootHolder<P::Renderer>>,
    pointer_grab_map: HashMap<PointerId, UniqueHandleId>,
}

impl<P: Platform> App<P> {
    /// Get the time recorded at the start of the frame.
    ///
    /// This will bind watch closures it is called in, and can be used to
    /// intentionally cause a watch closure to re-run every frame.
    pub fn time() -> time::Instant {
        AppValues::expect_current(|values| *values.frame_start)
    }

    /// A version of `time` which will not bind watch closures.
    pub fn time_unwatched() -> time::Instant {
        AppValues::try_with_current(|values| {
            *Watched::get_unwatched(&values.frame_start)
        })
        .unwrap_or_else(time::Instant::now)
    }

    /// This is similar to `time`, however it updates much less frequently.
    ///
    /// You may bind to this in a watch closure to cause it to re-run
    /// periodically.
    ///
    /// Current precision is 1 second, however this should not be relied
    /// upon and may change in the future.
    pub fn coarse_time() -> time::Instant {
        AppValues::expect_current(|values| *values.coarse_time)
    }

    /// Start running the app.
    ///
    /// Because of platform-specific requirements, this requires control
    /// of the current thread.
    pub fn run(mut self) -> ! {
        let (width, height) = self.window.size();
        let values = AppValues::new_now(width, height);
        values
            .with(|| {
                self.platform.take().expect("app lost its platform").run(
                    move |state, event| {
                        self.handle_event(state, event);
                    },
                )
            })
            .1
    }

    /// Add a root widget to the app.
    ///
    /// Root widgets are assigned a Rect representing the whole window.
    /// They are drawn in the order they are added to the app.
    /// They recieve pointer events in reverse order of when they are added to
    /// the app.
    pub fn add_root<T>(&mut self, mut widget: Widget<T>)
    where
        T: widget::Content<P::Renderer>,
    {
        let (width, height) = self.window.size();
        let rect = SimpleRect::with_size(width, height);
        widget.set_fill(&rect, &SimplePadding2d::zero());
        let holder = Rc::new(RefCell::new(widget.into_root()));
        let watcher = Rc::downgrade(&holder);
        self.roots.push(holder);
        self.watch_ctx.add_watcher(&watcher);
    }

    /// Create a test interface for this app, which allows simulating
    /// behavior.
    pub fn test<F: FnOnce(AppTesterInterface<P>)>(mut self, func: F) {
        self.window.recalculate_viewport();
        let (width, height) = self.window.size();
        let values = AppValues::new_now(width, height);
        values.with(|| {
            func(AppTesterInterface::new(&mut self));
        });
        std::mem::drop(self.roots);
        std::mem::drop(self.window);
        std::mem::drop(self.platform);
    }

    fn handle_event<E: EventLoopState>(
        &mut self,
        state: &mut E,
        event: Event,
    ) {
        use self::WindowEvent::*;

        match event {
            Event::StartFrame(frame_time) => {
                AppValues::expect_current_mut(|values| {
                    *values.frame_start = frame_time;
                    let duration = frame_time.duration_since(
                        *Watched::get_unwatched(&values.coarse_time),
                    );
                    if duration >= AppValues::COARSE_STEP {
                        *values.coarse_time = frame_time;
                    }
                });
            }
            Event::Update => {
                self.watch_ctx.update();
            }
            Event::TakeScreenshot(dest) => {
                *dest = self.window.take_screenshot();
            }
            Event::Draw => {
                self.draw();
            }
            Event::FinishDraw => {
                self.window.flip();
            }
            Event::WindowEvent(Quit) => {
                state.request_shutdown();
            }
            Event::WindowEvent(Resize) => {
                let (x, y) = self.window.size();
                AppValues::expect_current_mut(|values| {
                    *values.cell_size = get_cell_size(x, y);
                    values.window_size.0 = x;
                    values.window_size.1 = y;
                });
                let xdim = Dim::with_length(x);
                let ydim = Dim::with_length(y);
                let rect = SimpleRect::new(xdim, ydim);
                for root in self.roots.iter_mut() {
                    root.borrow_mut()
                        .widget
                        .set_fill(&rect, &SimplePadding2d::zero());
                }
                self.window.recalculate_viewport();
            }
            Event::WindowEvent(DpScaleChange) => {
                let ppd = self.window.pixels_per_dp();
                AppValues::expect_current_mut(|values| {
                    *values.px_per_dp = ppd;
                });
            }
            Event::WindowEvent(KeyDown(_key)) => {}
            Event::WindowEvent(Pointer(mut pointer)) => {
                if !pointer.normalized {
                    self.window.normalize_pointer_event(&mut pointer);
                }
                self.pointer_event(pointer);
            }
        }
    }

    fn draw(&mut self) {
        let mut loop_count = 0;
        let mut first_pass = true;
        loop {
            let mut ctx = self.window.prepare_draw(first_pass);
            let mut borrowed_roots: Vec<_> = self
                .roots
                .iter_mut()
                .map(|root| root.borrow_mut())
                .collect();
            let iter = borrowed_roots.iter_mut().map(|boxed| {
                let as_ref: &mut dyn AnonWidget<_> = &mut boxed.widget;
                as_ref
            });
            let need_loop = ctx.draw(iter);
            std::mem::drop(borrowed_roots);
            std::mem::drop(ctx);
            if !need_loop {
                break;
            }
            first_pass = false;
            debug_assert!(
                loop_count < 1024,
                "render exceeded its loop count (possible infinite loop)",
            );
            self.watch_ctx.update();
            loop_count += 1;
        }
    }

    fn pointer_event(&mut self, pointer: PointerEventData) {
        let mut event = PointerEvent::new(pointer, &mut self.pointer_grab_map);
        let mut handled = false;
        let mut iter = self.roots.iter_mut().rev();
        while let (false, Some(root)) = (handled, iter.next()) {
            handled = root.borrow_mut().widget.pointer_event(&mut event);
        }
    }

    /// Consume the current app, cleaning up its resources immediately.
    pub fn shutdown(self) {
        let Self { window, roots, .. } = self;
        std::mem::drop(roots);
        std::mem::drop(window);
    }
}

impl Default for App {
    fn default() -> Self {
        AppBuilder::default().build()
    }
}
