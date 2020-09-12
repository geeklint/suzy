/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::math::Rect;
use crate::platform::{DefaultPlatform, DefaultRenderPlatform, RenderPlatform};
use crate::pointer::PointerEvent;

use super::{
    WidgetChildReceiver,
    WidgetGraphicReceiver,
    WidgetInit,
    WidgetExtra,
};

/// This trait should be implemented for the types you provide as data for
/// Widget implementations.
pub trait WidgetContent<P = DefaultRenderPlatform>
where
    P: RenderPlatform,
    Self: 'static,
{
    /// This method provides a convient place to register functions which
    /// watch values and update parts of your widget when they change.
    fn init<I: WidgetInit<Self, P>>(init: I);

    /// Use this method to specify the children a custom widget contains.
    ///
    /// Call `receiver.child` for each child.
    fn children<R: WidgetChildReceiver<P>>(&mut self, receiver: R);

    /// Use this method to specify the graphics a custom widget contains.
    ///
    /// Call `receiver.graphic` for each graphic.
    fn graphics<R: WidgetGraphicReceiver<P>>(&mut self, receiver: R);

    /// Override this method to define a custom shape for your widget.
    ///
    /// This is used by e.g. Button to test if it should handle a pointer
    /// event.  The default is a standard rectangular test.
    fn hittest(&self, extra: &mut WidgetExtra<'_>, point: (f32, f32)) -> bool {
        extra.contains(point)
    }

    /// Override this method to handle pointer events directly by a custom
    /// widget.
    ///
    /// Return true if you successfully handled the event.
    fn pointer_event(
        &mut self,
        extra: &mut WidgetExtra<'_>,
        event: &mut PointerEvent,
    ) -> bool {
        let _unused = (extra, event);
        false
    }

    /// This is a convience function to create and run an App with this
    /// content as the only initial root widget.
    fn run_as_app() -> !
    where
        Self: Default + WidgetContent<DefaultRenderPlatform>,
    {
        run_widget_as_app::<Self>()
    }
}

impl<P: RenderPlatform> WidgetContent<P> for () {
    fn init<I: WidgetInit<Self, P>>(_init: I) {}
    fn children<R: WidgetChildReceiver<P>>(&mut self, _receiver: R) {}
    fn graphics<R: WidgetGraphicReceiver<P>>(&mut self, _receiver: R) {}
}

fn run_widget_as_app<T>() -> !
where
    T: Default + WidgetContent<DefaultRenderPlatform>,
{
    use crate::app::{App, AppBuilder};
    use crate::window::WindowSettings;

    let name = std::any::type_name::<T>().rsplit("::").next().unwrap();
    let (_, title) = name.chars().fold(
        (false, String::new()),
        |(prev, mut title), ch| {
            if prev && ch.is_uppercase() {
                title.push(' ');
            }
            title.push(ch);
            (ch.is_lowercase(), title)
        }
    );
    let mut builder = AppBuilder::default();
    builder.set_title(title);
    let app: App<DefaultPlatform> = builder.build();
    let (app, _) = app.with(|current| {
        current.add_root(super::Widget::<T>::default);
    });
    app.run();
}
