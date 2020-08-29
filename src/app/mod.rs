use std::time;
use std::collections::HashMap;

use drying_paint::{
    WatchContext,
};

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
mod tester;
mod values;

pub use builder::AppBuilder;
pub use tester::AppTesterInterface;
pub(crate) use values::{
    AppValues,
    get_cell_size,
};

pub struct App<P = DefaultPlatform>
where
    P: Platform,
{
    platform: P,
    watch_ctx: WatchContext,
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
        let window = Some(window);
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
        let window = window.expect("CurrentApp lost its Window");
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
        let window = Some(window);
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

    pub fn test<F: FnOnce(AppTesterInterface<P>)>(self, func: F) {
        let Self {
            platform,
            watch_ctx,
            window,
            roots,
            values,
            pointer_grab_map,
        } = self;
        let window = Some(window);
        let mut current = CurrentApp::<P> {
            window, roots, pointer_grab_map,
        };
        watch_ctx.with(|| {
            values.with(|| {
                func(AppTesterInterface::new(&mut current));
            });
        });
        std::mem::drop(current.roots);
        std::mem::drop(current.window);
        std::mem::drop(platform);
    }
}

pub struct CurrentApp<P = DefaultPlatform>
where
    P: Platform
{
    window: Option<P::Window>,
    roots: Vec<OwnedWidgetProxy<P::Renderer>>,
    pointer_grab_map: HashMap<PointerId, WidgetId>,
}

impl<P: Platform> CurrentApp<P> {
    fn window(&mut self) -> &mut P::Window {
        self.window.as_mut().expect("CurrentApp lost its Window")
    }

    pub fn add_root<F, T>(&mut self, f: F)
    where
        F: 'static + FnOnce() -> Widget<T, P::Renderer>,
        T: WidgetContent<P::Renderer>,
    {
        let (width, height) = self.window().size();
        let rect = SimpleRect::with_size(width, height);
        self.access_roots(move |roots| {
            let mut widget = f();
            widget.set_fill(&rect, &SimplePadding2d::zero());
            roots.push(widget.into());
        });
    }

    fn access_roots<F, R>(&mut self, func: F) -> R
    where
        F: 'static + FnOnce(&mut Vec<OwnedWidgetProxy<P::Renderer>>) -> R,
        R: 'static
    {
        let roots = std::mem::take(&mut self.roots);
        let (roots, ret) = WatchContext::allow_watcher_access(
            roots,
            move |mut roots| {
                let ret = func(&mut roots);
                (roots, ret)
            }
        );
        self.roots = roots;
        ret
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
                WatchContext::update_current();
            },
            Event::TakeScreenshot(dest) => {
                *dest = self.window().take_screenshot();
            },
            Event::Draw => {
                self.draw();
            },
            Event::FinishDraw => {
                self.window().flip();
            },
            Event::WindowEvent(Quit) => {
                state.request_shutdown();
            },
            Event::WindowEvent(Resize) => {
                let (x, y) = self.window().size();
                AppValues::expect_current_mut(|values| {
                    *values.cell_size = get_cell_size(x, y);
                    values.window_size.0 = x;
                    values.window_size.1 = y;
                });
                let xdim = Dim::with_length(x);
                let ydim = Dim::with_length(y);
                let rect = SimpleRect::new(xdim, ydim);
                self.access_roots(move |roots| {
                    for root in roots.iter_mut() {
                        root.set_fill(&rect, &SimplePadding2d::zero());
                    }
                });
                self.window().recalculate_viewport();
            },
            Event::WindowEvent(DpScaleChange) => {
                let ppd = self.window().pixels_per_dp();
                AppValues::expect_current_mut(|values| {
                    *values.px_per_dp = ppd;
                });
            },
            Event::WindowEvent(KeyDown(_key)) => {
            },
            Event::WindowEvent(Pointer(mut pointer)) => {
                if !pointer.normalized {
                    self.window().normalize_pointer_event(&mut pointer);
                }
                self.pointer_event(pointer);
            },
        }
    }

    fn draw(&mut self) {
        let mut loop_count = 0;
        let mut first_pass = true;
        loop {
            let mut window = self.window.take()
                .expect("CurrentApp lost its Window");
            let (window, need_loop) = self.access_roots(move |roots| {
                let mut ctx = window.prepare_draw(first_pass);
                let need_loop = DrawContext::draw(
                    &mut ctx,
                    roots.iter_mut(),
                );
                (window, need_loop)
            });
            self.window = Some(window);
            if !need_loop {
                break;
            }
            first_pass = false;
            debug_assert!(
                loop_count < 1024,
                "render exceeded its loop count (possible infinite loop)",
            );
            WatchContext::update_current();
            loop_count += 1;
        }
    }

    fn pointer_event(
        &mut self,
        pointer: PointerEventData,
    ) {
        let mut grab_map = std::mem::take(&mut self.pointer_grab_map);
        let (stolen_from, mut grab_map) = self.access_roots(move |roots| {
            let mut event = PointerEvent::new(
                pointer,
                &mut grab_map,
            );
            let mut handled = false;
            let mut iter = roots.iter_mut().rev();
            while let (false, Some(root)) = (handled, iter.next()) {
                handled = root.pointer_event(&mut event);
            }
            (event.grab_stolen_from, grab_map)
        });
        self.pointer_grab_map = if let Some(id) = stolen_from {
            self.access_roots(move |roots| {
                let mut found = false;
                let mut iter = roots.iter_mut();
                while let (false, Some(root)) = (found, iter.next()) {
                    root.find_widget(id.clone(), |widget| {
                        found = true;
                        widget.pointer_event_self(&mut PointerEvent::new(
                            PointerEventData {
                                action: PointerAction::GrabStolen,
                                ..pointer
                            },
                            &mut grab_map,
                        ));
                    });
                }
                grab_map
            })
        } else {
            grab_map
        };
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
