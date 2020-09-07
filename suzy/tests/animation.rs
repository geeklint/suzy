extern crate suzy;

use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use suzy::animation::Animation;
use suzy::window::WindowSettings;
use suzy::math::consts::{
    BLACK,
};
use suzy::app::{
    App,
    AppBuilder,
};
use suzy::widget::{
    Widget,
    WidgetContent,
    WidgetInit,
    WidgetChildReceiver,
    WidgetMutChildReceiver,
    WidgetGraphicReceiver,
};
use suzy::platform::opengl::{
    OpenGlRenderPlatform,
};
use suzy::platform::osmesa::OSMesaPlatform;


#[derive(Default)]
struct Root {
    anim: Animation<f32>,
    value_feedback: Rc<Cell<f32>>,
    started: bool,
}

impl WidgetContent<OpenGlRenderPlatform> for Root {
    fn init<I: WidgetInit<Self, OpenGlRenderPlatform>>(mut init: I) {
        init.watch(|root, _rect| {
            let mut value = root.value_feedback.get();
            root.anim.apply(&mut value);
            root.value_feedback.set(value);
        });
        init.watch(|root, _rect| {
            if !root.started {
                root.anim.set_duration(Duration::from_secs(1));
                root.anim.animate_to(261.0);
                root.started = true;
            }
        });
    }

    fn children<R: WidgetChildReceiver<OpenGlRenderPlatform>>(&self, _receiver: R) {
    }

    fn children_mut<R: WidgetMutChildReceiver<OpenGlRenderPlatform>>(&mut self, _receiver: R) {
    }

    fn graphics<R: WidgetGraphicReceiver<OpenGlRenderPlatform>>(&mut self, _receiver: R) {
    }

}

#[test]
fn animation() {
    let mut builder = AppBuilder::default();
    builder.set_size((480.0, 360.0));
    builder.set_background_color(BLACK);
    let app: App<OSMesaPlatform> = builder.build();
    let value_output = Rc::new(Cell::new(142.0));
    let value_feedback = Rc::clone(&value_output);
    let app = app.with(|app| {
        app.add_root(move || {
            let mut root = Widget::<Root>::default();
            root.value_feedback = value_feedback;
            root
        });
    }).0;
    app.test(|mut app| {
        assert!(
            (value_output.get() - 142.0).abs() < f32::EPSILON,
            "value is {}", value_output.get(),
        );
        app.next_frame(Duration::from_millis(100));
        app.draw_if_needed();
        assert!(
            (value_output.get() - 153.9).abs() < f32::EPSILON,
            "value is {}", value_output.get(),
        );
        app.next_frame(Duration::from_millis(226));
        app.draw_if_needed();
        assert!(
            (value_output.get() - 180.794).abs() < f32::EPSILON,
            "value is {}", value_output.get(),
        );
        app.next_frame(Duration::from_millis(195));
        app.draw_if_needed();
        assert!(
            (value_output.get() - 203.99901).abs() < f32::EPSILON,
            "value is {}", value_output.get(),
        );
        app.next_frame(Duration::from_millis(407));
        app.draw_if_needed();
        assert!(
            (value_output.get() - 252.43199).abs() < f32::EPSILON,
            "value is {}", value_output.get(),
        );
        app.next_frame(Duration::from_millis(72));
        app.draw_if_needed();
        assert!(
            (value_output.get() - 261.0).abs() < f32::EPSILON,
            "value is {}", value_output.get(),
        );
    });
}
