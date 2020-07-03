use std::cell::RefCell;
use std::time;
use std::convert::TryInto;

use drying_paint::Watched;

use crate::platform::{DefaultPlatform, Platform, RenderPlatform};
use crate::pointer::{PointerEvent, PointerId};
use crate::widget::{Widget, WidgetContent, WidgetId};
use crate::window;
use crate::dims::{Dim, SimpleRect, Rect, SimplePadding2d, Padding2dNew};
use window::{Window, WindowEvent, WindowSettings};

thread_local! {
    static APP_STACK: RefCell<Vec<AppValues>> = RefCell::new(Vec::new());
}

#[derive(Clone)]
pub(crate) struct AppValues {
    pub frame_start: Watched<time::Instant>,
    pub cell_size: Watched<f32>,
    pub px_per_dp: Watched<f32>,
    pub window_size: (f32, f32),
}

pub(crate) fn try_with_current<F, R>(func: F) -> Option<R>
where
    F: FnOnce(&AppValues) -> R
{
    APP_STACK.with(|cell| {
        if let Some(top) = cell.borrow().last() {
            Some((func)(top))
        } else {
            None
        }
    })
}

pub(crate) fn expect_current<F, R>(func: F) -> R
where
    F: FnOnce(&AppValues) -> R
{
    APP_STACK.with(|cell| {
        let stack = cell.borrow();
        let top = stack.last().expect("App context is not valid");
        (func)(top)
    })
}

fn get_cell_size(width: f32, height: f32) -> f32 {
    if cfg!(feature = "tui") {
        16.0
    } else {
        for dist in 0..=4 {
            let dist = dist as f32;
            let high = 16.0 + dist;
            let low = 16.0 - dist;
            if width % high == 0.0 && height % high == 0.0 {
                return high;
            } else if width % low == 0.0 && height % low == 0.0 {
                return low;
            }
        }
        let (longer, shorter) = if width > height {
            (width, height)
        } else {
            (height, width)
        };
        let min_cells = (longer / 12.0).ceil() as u32;
        let max_cells = (longer / 20.0).floor() as u32;
        let mut best = 0.0;
        let mut best_dist = std::f32::INFINITY;
        for test_cells in min_cells..=max_cells {
            let cell_size = longer / (test_cells as f32);
            let tiles_shorter = shorter / cell_size;
            let dist = (tiles_shorter - tiles_shorter.round()).abs();
            if dist < best_dist {
                best = cell_size;
                best_dist = dist;
            }
        }
        best
    }
}

#[derive(Default)]
pub struct AppBuilder {
    win: window::WindowBuilder,
}

impl window::WindowSettings for AppBuilder {
    fn size(&self) -> (f32, f32) { self.win.size() }
    
    fn set_size(&mut self, size: (f32, f32)) {
        self.win.set_size(size);
    }

    fn title(&self) -> &str { self.win.title() }

    fn set_title(&mut self, title: String) {
        self.win.set_title(title);
    }

    fn fullscreen(&self) -> bool { self.win.fullscreen() }

    fn set_fullscreen(&mut self, fullscreen: bool) {
        self.win.set_fullscreen(fullscreen);
    }

    fn background_color(&self) -> crate::math::Color {
        self.win.background_color()
    }

    fn set_background_color(&mut self, color: crate::math::Color) {
        self.win.set_background_color(color);
    }
}

struct EventsState {
    quit: bool,
    grab_map: std::collections::HashMap<PointerId, WidgetId>,
}

pub struct App<Root, P = DefaultPlatform>
where
    Root: WidgetContent<P::Renderer>,
    P: Platform,
{
    watch_ctx: Option<drying_paint::WatchContext>,
    window: P::Window,
    root: Widget<Root, P::Renderer>,
    values: AppValues,
    events_state: EventsState,
    frame_start: Option<time::Instant>,
}

impl<Root, P> App<Root, P>
where
    Root: WidgetContent<P::Renderer>,
    P: Platform,
{
    pub fn time() -> time::Instant {
        try_with_current(|values| {
            *values.frame_start
        }).unwrap_or_else(time::Instant::now)
    }

    pub fn sync(&mut self) {
        self.window.flip();
    }

    pub fn run(mut self) {
        while !self.events_state.quit {
            self.tick();
            self.sync();
        }
        self.shutdown();
    }

    fn with_values<F: FnOnce() -> R, R>(values: &mut AppValues, func: F) -> R {
        let mut local = values.clone();
        std::mem::swap(&mut local, values);
        APP_STACK.with(|cell| cell.borrow_mut().push(local));
        let res = (func)();
        *values = APP_STACK.with(|cell| cell.borrow_mut().pop()).unwrap();
        res
    }

    fn poll_events(
        window: &mut P::Window,
        root: &mut Widget<Root, P::Renderer>,
        state: &mut EventsState,
    ) {
        while let Some(event) = window.next_event() {
            match event {
                WindowEvent::Quit => {
                    state.quit = true;
                }
                WindowEvent::Resize(x, y) => {
                    APP_STACK.with(|cell| {
                        let mut handle = cell.borrow_mut();
                        let values = handle.last_mut().unwrap();
                        *values.cell_size = get_cell_size(x, y);
                        values.window_size.0 = x;
                        values.window_size.1 = y;
                    });
                    let xdim = Dim::with_length(x);
                    let ydim = Dim::with_length(y);
                    let rect = SimpleRect::new(xdim, ydim);
                    root.set_fill(&rect, &SimplePadding2d::zero());
                },
                WindowEvent::DpScaleChange(ppd) => {
                    APP_STACK.with(|cell| {
                        let mut handle = cell.borrow_mut();
                        let values = handle.last_mut().unwrap();
                        *values.px_per_dp = ppd;
                    });
                },
                WindowEvent::KeyDown(_key) => {
                },
                WindowEvent::Pointer(pointer) => {
                    let mut event = PointerEvent::new(
                        pointer,
                        &mut state.grab_map,
                    );
                    root.pointer_event(&mut event);
                },
            }
        }
    }

    pub fn tick(&mut self) {
        self.update();
        self.render();
    }

    pub fn frame_start_time(&mut self) {
        self.frame_start = Some(time::Instant::now());
    }

    pub fn update(&mut self) {
        let Self {
            watch_ctx,
            window,
            root,
            values,
            events_state,
            ..
        } = self;
        let frame_start = self.frame_start.take()
            .unwrap_or_else(time::Instant::now);
        let watch_ctx_inner = watch_ctx.take().unwrap();
        let watch_ctx_inner = watch_ctx_inner.with(|| {
            *values.frame_start = frame_start;
            Self::with_values(values, || {
                Self::poll_events(window, root, events_state);
                drying_paint::WatchContext::update_current();
            });
        }).0;
        *watch_ctx = Some(watch_ctx_inner);
    }

    pub fn render(&mut self) {
        let Self { watch_ctx, window, root, ..  } = self;
        window.clear();
        let watch_ctx_inner = watch_ctx.take().unwrap();
        let watch_ctx_inner = watch_ctx_inner.with(|| {
            let mut ctx = window.prepare_draw();
            while crate::graphics::DrawContext::draw(&mut ctx, root) {
                drying_paint::WatchContext::update_current();
            }
        }).0;
        *watch_ctx = Some(watch_ctx_inner);
    }

    pub fn shutdown(self) {
        let Self { window, root, ..  } = self;
        std::mem::drop(root);
        std::mem::drop(window);
    }
}

impl<Root> App<Root>
where Root: WidgetContent + Default
{
    pub fn new() -> Self {
        let builder = AppBuilder::default();
        let window: <DefaultPlatform as Platform>::Window = {
            builder.win.try_into().unwrap()
        };
        let watch_ctx = drying_paint::WatchContext::new();

        let (width, height) = window.size();
        let xdim = Dim::with_length(width);
        let ydim = Dim::with_length(height);
        let rect = SimpleRect::new(xdim, ydim);
        
        let mut values = AppValues {
            frame_start: Watched::new(time::Instant::now()),
            cell_size: Watched::new(get_cell_size(width, height)),
            px_per_dp: Watched::new(1.0),
            window_size: (width, height),
        };
        let (watch_ctx, root) = watch_ctx.with(|| {
            Self::with_values(&mut values, || {
                Widget::<Root>::default_with_rect(&rect)
            })
        });
        Self {
            watch_ctx: Some(watch_ctx),
            window,
            root,
            values,
            events_state: EventsState {
                quit: false,
                grab_map: std::collections::HashMap::new(),
            },
            frame_start: None,
        }
    }
}
