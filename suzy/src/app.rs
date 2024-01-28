/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

//! An App describes the context in which widgets exist.
//!
//! Apps have an associated window and "root" widgets, which are assigned
//! to fill the whole window area.

use std::{cell::RefCell, rc::Rc, time};

use crate::{
    dims::{Padding2d, Rect, SimpleRect},
    graphics::PlatformDrawContext,
    platform::Platform,
    pointer::{PointerEvent, PointerEventData},
    widget::{self, Widget},
    window::Window,
};

mod tester;
mod values;

pub use tester::AppTestingExt;
pub(crate) use values::AppState;

#[cfg(feature = "platform_sdl")]
pub type App<P = crate::platforms::DefaultPlatform> = app_struct::App<P>;

#[cfg(not(feature = "platform_sdl"))]
pub type App<P> = app_struct::App<P>;

mod app_struct {
    use crate::{
        platform::Platform,
        pointer::PointerId,
        watch::WatchContext,
        widget::{AnonWidget, UniqueHandleId},
    };
    use std::{cell::RefCell, collections::HashMap, rc::Rc};

    type RootHolder<P> = Rc<RefCell<dyn AnonWidget<P>>>;

    /// A type which contains the context in which widgets run.
    ///
    /// See the [module-level documentation](crate::app) for more details.
    pub struct App<P>
    where
        P: ?Sized + Platform,
    {
        pub(crate) watch_ctx: WatchContext<'static>,
        pub(crate) window: P::Window,
        pub(super) roots: Vec<RootHolder<P::Renderer>>,
        pub(super) pointer_grab_map: HashMap<PointerId, UniqueHandleId>,
        pub(crate) state: Rc<super::AppState>,
        pub(super) needs_draw: bool,
    }
}

/// Get the time recorded at the start of the frame.
///
/// This will bind watch closures it is called in, and can be used to
/// intentionally cause a watch closure to re-run every frame.
pub fn time() -> time::Instant {
    AppState::try_with_current(|state| state.frame_start.get_auto())
        .expect("there is no valid app state to get time from")
}

/// A version of `time` which will not bind watch closures.
pub fn time_unwatched() -> time::Instant {
    AppState::try_with_current(|state| state.frame_start.get_unwatched())
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
    AppState::try_with_current(|state| state.coarse_time.get_auto())
        .expect("there is no valid app state to get coarse_time from")
}

impl<P: Platform> App<P> {
    pub fn from_window(window: P::Window) -> Self {
        use std::collections::HashMap;

        use crate::watch::WatchContext;

        let [width, height] = window.size();
        let state = Rc::new(AppState::new_now(width, height));

        let watch_ctx: WatchContext<'static> = WatchContext::new();

        Self {
            watch_ctx,
            window,
            roots: Vec::new(),
            pointer_grab_map: HashMap::new(),
            state,
            needs_draw: true,
        }
    }

    pub fn state(&self) -> &AppState {
        &self.state
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
        let width = self.state.window_width.get_unwatched();
        let height = self.state.window_height.get_unwatched();
        let rect = SimpleRect::with_size(width, height);
        widget.set_fill(&rect, &Padding2d::zero());
        let holder = Rc::new(RefCell::new(widget));
        let watcher = Rc::downgrade(&holder);
        self.roots.push(holder);
        Widget::init(watcher, self);
        self.needs_draw = true;
    }

    pub fn start_frame(&mut self, frame_time: time::Instant) {
        self.state.frame_start.set_external(frame_time);
        let duration =
            frame_time.duration_since(self.state.coarse_time.get_unwatched());
        if duration >= AppState::COARSE_STEP {
            self.state.coarse_time.set_external(frame_time);
        }
        self.needs_draw = true;
    }

    pub fn update_watches(&mut self) {
        self.watch_ctx.update();
        self.needs_draw = true;
    }

    pub fn loop_draw(&mut self) {
        let mut loop_count: u32 = 0;
        let mut pass_arg = None;
        loop {
            let mut ctx = self.window.prepare_draw(pass_arg);
            for root in self.roots.iter_mut() {
                root.borrow_mut().draw(&mut ctx);
            }
            pass_arg = ctx.finish();
            if pass_arg.is_none() {
                break;
            }
            debug_assert!(
                loop_count < 1024,
                "render exceeded its loop count (possible infinite loop)",
            );
            self.watch_ctx.update();
            loop_count += 1;
        }
        self.needs_draw = false;
    }

    pub fn draw(
        &mut self,
        ctx: &mut crate::graphics::DrawContext<'_, P::Renderer>,
    ) {
        for root in self.roots.iter_mut() {
            root.borrow_mut().draw(ctx);
        }
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        self.state.window_width.set_external(width);
        self.state.window_height.set_external(height);
        for root in self.roots.iter_mut() {
            let mut wid = root.borrow_mut();
            wid.set_horizontal_stretch(0.0, width);
            wid.set_vertical_stretch(0.0, height);
        }
        self.needs_draw = true;
    }

    pub fn update_dpi(&mut self, dpi: [f32; 2]) {
        self.state.dpi.set_external(dpi);
        self.needs_draw = true;
    }

    pub fn pointer_event(&mut self, pointer: PointerEventData) -> bool {
        let mut event = PointerEvent::new(pointer, &mut self.pointer_grab_map);
        let mut handled = false;
        let mut iter = self.roots.iter_mut().rev();
        while let (false, Some(root)) = (handled, iter.next()) {
            handled = root.borrow_mut().pointer_event(&mut event);
        }
        self.needs_draw = true;
        handled
    }

    /// Consume the current app, cleaning up its resources immediately.
    pub fn shutdown(self) {
        let Self { window, roots, .. } = self;
        std::mem::drop(roots);
        std::mem::drop(window);
    }
}
