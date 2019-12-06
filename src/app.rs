use crate::widget;
use crate::platform;
use crate::dims::{SimpleRect, Rect};

pub struct App<Root>
where Root: widget::WidgetData + Default
{
}

impl<Root: widget::WidgetData> App<Root> {
    pub fn run() {
        let window = platform::Window::new().unwrap();
        let watch_ctx = drying_paint::WatchContext::new();

        let mut root = None;
        {
            let (width, height) = window.get_size() as (f32, f32);
            let width = Dim::with_length(width);
            let height = Dim::with_length(height);
            let rect = SimpleRect::new(width, height);
            
            watch_ctx.with(|| {
                root = Some(Widget<Root>::default_with_rect(&rect));
            });
        }
        let root = root.unwrap();
        for event in window.events() {

        }
    }
}
