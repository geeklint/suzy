use std::cell::RefCell;
use std::time;
use std::convert::TryInto;
use std::collections::HashMap;

use drying_paint::Watched;

use crate::platform::{
    DefaultPlatform,
    Platform,
    Event,
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

thread_local! {
    static APP_STACK: RefCell<Vec<AppValues>> = RefCell::new(Vec::new());
}

#[derive(Clone, Debug)]
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
    grab_map: HashMap<PointerId, WidgetId>,
}

pub struct App<P = DefaultPlatform>
where
    P: Platform,
{
    watch_ctx: Option<drying_paint::WatchContext>,
    window: P::Window,
    roots: Vec<OwnedWidgetProxy<P::Renderer>>,
    values: AppValues,
    events_state: EventsState,
}

impl<P: Platform> App<P> {
    pub fn add_root<F, T>(&mut self, f: F)
    where
        F: FnOnce() -> Widget<T, P::Renderer>,
        T: WidgetContent<P::Renderer>,
    {
        let Self { watch_ctx, roots, values, window, .. } = self;
        let (width, height) = window.size();
        let rect = SimpleRect::with_size(width, height);
        let watch_ctx_inner = watch_ctx.take().unwrap();
        let watch_ctx_inner = watch_ctx_inner.with(|| {
            Self::with_values(values, || {
                let mut widget = f();
                widget.set_fill(&rect, &SimplePadding2d::zero());
                roots.push(widget.into());
            });
        }).0;
        *watch_ctx = Some(watch_ctx_inner);
    }

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

    fn handle_event(
        roots: &mut Vec<OwnedWidgetProxy<P::Renderer>>,
        state: &mut EventsState,
        event: Event<P::Window>,
    ) {
        use self::WindowEvent::*;

        match event {
            Event::StartFrame(frame_time) => {
                APP_STACK.with(|cell| {
                    let mut handle = cell.borrow_mut();
                    let values = handle.last_mut().unwrap();
                    *values.frame_start = frame_time;
                });
            },
            Event::Update => {
                drying_paint::WatchContext::update_current();
            },
            Event::Draw(window) => {
                window.clear();
                let mut ctx = window.prepare_draw();
                let mut loop_count = 0;
                while DrawContext::draw(&mut ctx, roots.iter_mut()) {
                    debug_assert!(
                        loop_count < 1024,
                        "render exceeded its loop count (possible infinite loop)",
                    );
                    drying_paint::WatchContext::update_current();
                    loop_count += 1;
                }
            },
            Event::WindowEvent(Quit) => {
                state.quit = true;
            },
            Event::WindowEvent(Resize(x, y)) => {
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
                for root in roots.iter_mut() {
                    root.set_fill(&rect, &SimplePadding2d::zero());
                }
            },
            Event::WindowEvent(DpScaleChange(ppd)) => {
                APP_STACK.with(|cell| {
                    let mut handle = cell.borrow_mut();
                    let values = handle.last_mut().unwrap();
                    *values.px_per_dp = ppd;
                });
            },
            Event::WindowEvent(KeyDown(_key)) => {
            },
            Event::WindowEvent(Pointer(pointer)) => {
                Self::pointer_event(roots, &mut state.grab_map, pointer);
            },
        }
    }

    fn pointer_event(
        roots: &mut Vec<OwnedWidgetProxy<P::Renderer>>,
        grab_map: &mut HashMap<PointerId, WidgetId>,
        pointer: PointerEventData,
    ) {
        let mut event = PointerEvent::new(
            pointer,
            grab_map,
        );
        {
            let mut handled = false;
            let mut iter = roots.iter_mut().rev();
            while let (false, Some(root)) = (handled, iter.next()) {
                handled = root.pointer_event(&mut event);
            }
        }
        if let Some(id) = event.grab_stolen_from {
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
                        grab_map,
                    ));
                });
            }
        }
    }

    pub fn tick(&mut self) {
        self.update();
    }

    pub fn update(&mut self) {
        let Self {
            watch_ctx,
            window,
            roots,
            values,
            events_state,
            ..
        } = self;
        let watch_ctx_inner = watch_ctx.take().unwrap();
        let watch_ctx_inner = watch_ctx_inner.with(|| {
            Self::with_values(values, || {
                Self::handle_event(
                    roots,
                    events_state,
                    Event::StartFrame(time::Instant::now()),
                );
                while let Some(event) = window.next_event() {
                    Self::handle_event(
                        roots,
                        events_state,
                        Event::WindowEvent(event),
                    );
                }
                Self::handle_event(
                    roots,
                    events_state,
                    Event::Update,
                );
                Self::handle_event(
                    roots,
                    events_state,
                    Event::Draw(window),
                );
            });
        }).0;
        *watch_ctx = Some(watch_ctx_inner);
    }

    pub fn shutdown(self) {
        let Self { window, roots, ..  } = self;
        std::mem::drop(roots);
        std::mem::drop(window);
    }
}

impl Default for App {
    fn default() -> Self {
        let builder = AppBuilder::default();
        let window: <DefaultPlatform as Platform>::Window = {
            builder.win.try_into().unwrap()
        };
        let watch_ctx = drying_paint::WatchContext::new();

        let (width, height) = window.size();
        
        let mut values = AppValues {
            frame_start: Watched::new(time::Instant::now()),
            cell_size: Watched::new(get_cell_size(width, height)),
            px_per_dp: Watched::new(1.0),
            window_size: (width, height),
        };
        Self {
            watch_ctx: Some(watch_ctx),
            window,
            roots: Vec::new(),
            values,
            events_state: EventsState {
                quit: false,
                grab_map: HashMap::new(),
            },
        }
    }
}
