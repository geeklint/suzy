use std::cell::RefCell;
use std::time;
use std::collections::HashMap;

use drying_paint::Watched;

use crate::platform::{
    DefaultPlatform,
    Platform,
    Event,
    EventLoopState,
};
use crate::pointer::{
    PointerAction,
    PointerEvent,
    PointerId,
    PointerEventData,
};
use crate::widget::{
    Widget,
    WidgetContent,
    WidgetId,
    OwnedWidgetProxy,
};
use crate::window;
use crate::graphics::DrawContext;
use crate::dims::{Dim, SimpleRect, Rect, SimplePadding2d, Padding2dNew};
use window::{Window, WindowEvent, WindowSettings};

mod builder;
mod values;

pub use builder::AppBuilder;
pub(crate) use values::{
    AppValues,
    get_cell_size,
};

pub struct App<P = DefaultPlatform>
where
    P: Platform,
{
    platform: P,
    watch_ctx: drying_paint::WatchContext,
    window: P::Window,
    roots: Vec<OwnedWidgetProxy<P::Renderer>>,
    values: AppValues,
    pointer_grab_map: HashMap<PointerId, WidgetId>,
}

impl<P: Platform> App<P> {
    pub fn time() -> time::Instant {
        AppValues::try_with_current(|values| {
            *values.frame_start
        }).unwrap_or_else(time::Instant::now)
    }

    pub fn with<F, R>(self, func: F) -> (Self, R)
    where
        F: FnOnce(&mut CurrentApp<P>) -> R
    {
        let Self {
            platform,
            watch_ctx,
            window,
            roots,
            values,
            pointer_grab_map,
        } = self;
        let mut current = CurrentApp {
            window, roots, pointer_grab_map,
        };
        let current_ref = &mut current;
        let (watch_ctx, (values, res)) = watch_ctx.with(|| {
            values.with(move || {
                func(current_ref)
            })
        });
        let CurrentApp { window, roots, pointer_grab_map } = current;
        let new_self = Self {
            platform,
            watch_ctx,
            window,
            roots,
            values,
            pointer_grab_map,
        };
        (new_self, res)
    }

    pub fn run(self) -> ! {
        let Self {
            platform,
            watch_ctx,
            window,
            roots,
            values,
            pointer_grab_map,
        } = self;
        let mut current = CurrentApp::<P> {
            window, roots, pointer_grab_map,
        };
        let _res = watch_ctx.with(|| {
            values.with(|| {
                platform.run(move |state, event| {
                    current.handle_event(state, event);
                })
            })
        });
        (_res.1).1
    }
}

pub struct CurrentApp<P = DefaultPlatform>
where
    P: Platform
{
    window: P::Window,
    roots: Vec<OwnedWidgetProxy<P::Renderer>>,
    pointer_grab_map: HashMap<PointerId, WidgetId>,
}

impl<P: Platform> CurrentApp<P> {
    pub fn add_root<F, T>(&mut self, f: F)
    where
        F: FnOnce() -> Widget<T, P::Renderer>,
        T: WidgetContent<P::Renderer>,
    {
        let Self { roots, window, .. } = self;
        let (width, height) = window.size();
        let rect = SimpleRect::with_size(width, height);
        let mut widget = f();
        widget.set_fill(&rect, &SimplePadding2d::zero());
        roots.push(widget.into());
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
                });
            },
            Event::Update => {
                drying_paint::WatchContext::update_current();
            },
            Event::TakeScreenshot(dest) => {
                *dest = self.window.take_screenshot();
            },
            Event::Draw => {
                let mut ctx = self.window.prepare_draw();
                let mut loop_count = 0;
                while DrawContext::draw(&mut ctx, self.roots.iter_mut()) {
                    debug_assert!(
                        loop_count < 1024,
                        "render exceeded its loop count (possible infinite loop)",
                    );
                    drying_paint::WatchContext::update_current();
                    loop_count += 1;
                }
            },
            Event::FinishDraw => {
                self.window.flip();
            },
            Event::WindowEvent(Quit) => {
                state.request_shutdown();
            },
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
                    root.set_fill(&rect, &SimplePadding2d::zero());
                }
                self.window.recalculate_viewport();
            },
            Event::WindowEvent(DpScaleChange) => {
                let ppd = self.window.pixels_per_dp();
                AppValues::expect_current_mut(|values| {
                    *values.px_per_dp = ppd;
                });
            },
            Event::WindowEvent(KeyDown(_key)) => {
            },
            Event::WindowEvent(Pointer(mut pointer)) => {
                self.window.normalize_pointer_event(&mut pointer);
                self.pointer_event(pointer);
            },
        }
    }

    fn pointer_event(
        &mut self,
        pointer: PointerEventData,
    ) {
        let grab_map = &mut self.pointer_grab_map;
        let mut event = PointerEvent::new(
            pointer,
            grab_map,
        );
        {
            let mut handled = false;
            let mut iter = self.roots.iter_mut().rev();
            while let (false, Some(root)) = (handled, iter.next()) {
                handled = root.pointer_event(&mut event);
            }
        }
        if let Some(id) = event.grab_stolen_from {
            let mut found = false;
            let mut iter = self.roots.iter_mut();
            while let (false, Some(root)) = (found, iter.next()) {
                root.find_widget(id.clone(), |widget| {
                    found = true;
                    widget.pointer_event_self(&mut PointerEvent::new(
                        PointerEventData {
                            action: PointerAction::GrabStolen,
                            ..pointer
                        },
                        grab_map,
                    ));
                });
            }
        }
    }

    pub fn shutdown(self) {
        let Self { window, roots, ..  } = self;
        std::mem::drop(roots);
        std::mem::drop(window);
    }
}

impl Default for App {
    fn default() -> Self {
        AppBuilder::default().build()
    }
}
