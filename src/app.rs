use std::cell::RefCell;
use std::time;

use drying_paint::Watched;

use crate::widget::{Widget, WidgetData};
use crate::platform;
use crate::dims::{Dim, SimpleRect, Rect, SimplePadding2d, Padding2dNew};
use platform::WindowEvent;

thread_local! {
    static APP_STACK: RefCell<Vec<AppValues>> = RefCell::new(Vec::new());
}

pub(crate) struct AppValues {
    pub frame_start: Watched<time::Instant>,
    pub cell_size: Watched<f32>,
    pub px_per_dp: Watched<f32>,
}

pub(crate) fn try_with_current<F, R>(func: F) -> Option<R>
where F: FnOnce(&AppValues) -> R
{
    APP_STACK.with(|cell| {
        if let Some(top) = cell.borrow().last() {
            Some((func)(top))
        } else {
            None
        }
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

pub fn run<Root>()
where Root: WidgetData + Default
{
    let window = platform::Window::new().unwrap();
    let mut watch_ctx = drying_paint::WatchContext::new();

    let mut root = None;
    let (width, height) = window.get_size();
    let xdim = Dim::with_length(width);
    let ydim = Dim::with_length(height);
    let rect = SimpleRect::new(xdim, ydim);
    
    watch_ctx.with(|| {
        root = Some(Widget::<Root>::default_with_rect(&rect));
    });
    APP_STACK.with(|cell| {
        cell.borrow_mut().push(AppValues {
            frame_start: Watched::new(time::Instant::now()),
            cell_size: Watched::new(get_cell_size(width, height)),
            px_per_dp: Watched::new(1.0),
        });
    });
    let mut root = root.unwrap();
    watch_ctx.with(|| {
        for event in window.events() {
            match event {
                WindowEvent::Resize(x, y) => {
                    APP_STACK.with(|cell| {
                        *cell.borrow_mut().last_mut().unwrap().cell_size = {
                            get_cell_size(x, y)
                        };
                    });
                    let xdim = Dim::with_length(width);
                    let ydim = Dim::with_length(height);
                    let rect = SimpleRect::new(xdim, ydim);
                    root.set_fill(&rect, &SimplePadding2d::zero());
                },
                WindowEvent::KeyDown(key) => {

                },
            }
        }
    });
}
