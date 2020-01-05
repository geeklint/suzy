use std::cell::RefCell;
use std::time;
use std::convert::TryInto;

use drying_paint::Watched;

use crate::platform::DefaultWindow;
use crate::widget::{Widget, WidgetData};
use crate::window;
use crate::dims::{Dim, SimpleRect, Rect, SimplePadding2d, Padding2dNew};
use window::{WindowEvent, WindowSettings};

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
}

pub struct App<Root, W>
where
    Root: WidgetData,
    for<'a> W: window::Window<'a>,
{
    watch_ctx: drying_paint::WatchContext,
    window: W,
    root: Widget<Root>,
    values: AppValues,
    quit: bool,
}

impl<Root, W> App<Root, W>
where
    Root: WidgetData,
    for<'a> W: window::Window<'a>,
{
    pub fn time() -> time::Instant {
        try_with_current(|values| {
            *values.frame_start
        }).unwrap_or_else(time::Instant::now)
    }

    pub fn sync(&mut self) {
        self.window.flip();
    }

    pub fn run(&mut self) {
        while !self.quit {
            self.tick();
            self.sync();
        }
    }

    pub fn tick(&mut self) {
        let mut values = self.values.clone();
        std::mem::swap(&mut values, &mut self.values);
        let watch_ctx = &mut self.watch_ctx;
        let window = &mut self.window;
        let root = &mut self.root;
        let quit = &mut self.quit;
        watch_ctx.with(|| {
            *values.frame_start = time::Instant::now();
            APP_STACK.with(|cell| cell.borrow_mut().push(values));
            for event in window.events() {
                match event {
                    WindowEvent::Quit => {
                        *quit = true;
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
                    WindowEvent::KeyDown(key) => {
                        if key == 0x1b {
                            *quit = true;
                        }
                    },
                }
            }
            drying_paint::WatchContext::update_current();
        });
        window.clear();
        let mut ctx = window.before_draw();
        root.draw(&mut ctx);
        self.values = {
            APP_STACK.with(|cell| cell.borrow_mut().pop()).unwrap()
        };
    }
}

impl<Root> App<Root, DefaultWindow>
where Root: WidgetData + Default
{
    pub fn new() -> Self {
        let builder = AppBuilder::default();
        let window: DefaultWindow = builder.win.try_into().unwrap();
        let mut watch_ctx = drying_paint::WatchContext::new();

        let (width, height) = window.size();
        let xdim = Dim::with_length(width);
        let ydim = Dim::with_length(height);
        let rect = SimpleRect::new(xdim, ydim);
        
        APP_STACK.with(|cell| {
            cell.borrow_mut().push(AppValues {
                frame_start: Watched::new(time::Instant::now()),
                cell_size: Watched::new(get_cell_size(width, height)),
                px_per_dp: Watched::new(1.0),
                window_size: (width, height),
            });
        });
        let root = watch_ctx.with(|| {
            Widget::<Root>::default_with_rect(&rect)
        });
        let values = APP_STACK.with(|cell| cell.borrow_mut().pop()).unwrap();
        Self { watch_ctx, window, root, values, quit: false }
    }
}
