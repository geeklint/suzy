/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

//! An App describes the context in which widgets exist.
//!
//! Apps have an associated window and "root" widgets, which are assigned
//! to fill the whole window area.

use std::{cell::RefCell, rc::Rc, time};

use crate::{
    dims::{Padding2d, Rect, SimpleRect},
    platform::RenderPlatform,
    pointer::{PointerEvent, PointerEventData},
    widget::{self, Widget},
};

mod tester;
mod values;

pub use tester::AppTestingExt;
pub(crate) use values::AppState;

#[cfg(feature = "platform_opengl")]
pub type App<P = crate::platforms::DefaultRenderPlatform> = app_struct::App<P>;

#[cfg(not(feature = "platform_opengl"))]
pub type App<P> = app_struct::App<P>;

mod app_struct {
    use crate::{
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
        P: ?Sized,
    {
        pub(crate) watch_ctx: WatchContext<'static>,
        pub(super) roots: Vec<RootHolder<P>>,
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

impl<P> App<P> {
    pub fn new(width: f32, height: f32) -> Self {
        use std::collections::HashMap;

        use crate::watch::WatchContext;

        let state = Rc::new(AppState::new_now(width, height));

        let watch_ctx: WatchContext<'static> = WatchContext::new();

        Self {
            watch_ctx,
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
        P: RenderPlatform,
        T: widget::Content<P>,
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

    pub fn draw(&mut self, ctx: &mut crate::graphics::DrawContext<'_, P>)
    where
        P: RenderPlatform,
    {
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
}
