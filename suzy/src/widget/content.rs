/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use crate::dims::Rect;
use crate::platform::{
    DefaultPlatform, DefaultRenderPlatform, RenderPlatform,
};
use crate::pointer::PointerEvent;

use super::{
    WidgetChildReceiver, WidgetExtra, WidgetGraphicReceiver, WidgetInit,
};

/// This trait provides the "glue" between the data you define in custom
/// widgets and the behavior suzy defines for widgets.  There are three
/// required methods: `init`, `children`, and `graphics`.
///
/// The `init` method is the primary point for registering the `watch`
/// functions that define the behavior of a widget. See the
/// [watch](../watch/index.html) module for more information.
///
/// The methods `children` and `graphics` should be fairly straightforward
/// to implement: they provide a simple "internal iterator" format which
/// allows suzy to iterate over the children and graphics a custom widget
/// contains.
///
/// Custom widgets may contain any number of graphics and child widgets, or
/// none of either.
///
/// For example, if a custom widget contains two buttons as children:
///
/// ```rust
/// # use suzy::widget::*;
/// # use suzy::selectable::SelectableIgnored;
/// # type ButtonContent = SelectableIgnored<()>;
/// use suzy::widgets::Button;
///
/// struct MyWidgetData {
///     button_one: Button<ButtonContent>,
///     button_two: Button<ButtonContent>,
/// }
///
/// impl WidgetContent for MyWidgetData {
///     // ...
/// #   fn init(_init: impl WidgetInit<Self>) {}
/// #   fn graphics(&mut self, _receiver: impl WidgetGraphicReceiver) {}
///
///     fn children(&mut self, mut receiver: impl WidgetChildReceiver) {
///         receiver.child(&mut self.button_one);
///         receiver.child(&mut self.button_two);
///     }
/// }
/// ```
///
/// Or, if the custom widget only has a single graphic:
///
/// ```rust
/// # use suzy::widget::*;
/// # type MyGraphic = ();
///
/// struct MyWidgetData {
///     graphic: MyGraphic,
/// }
///
/// impl WidgetContent for MyWidgetData {
///     // ...
/// #   fn init(_init: impl WidgetInit<Self>) {}
/// #   fn children(&mut self, _receiver: impl WidgetChildReceiver) {}
///
///     fn graphics(&mut self, mut receiver: impl WidgetGraphicReceiver) {
///         receiver.graphic(&mut self.graphic);
///     }
/// }
/// ```
///
pub trait WidgetContent<P = DefaultRenderPlatform>
where
    P: RenderPlatform + ?Sized,
    Self: 'static,
{
    /// This method provides a convient place to register functions which
    /// watch values and update parts of the widget when they change.
    fn init(init: impl WidgetInit<Self, P>);

    /// Use this method to specify the children a custom widget contains.
    ///
    /// Call `receiver.child` for each child.
    fn children(&mut self, receiver: impl WidgetChildReceiver<P>);

    /// Use this method to specify the graphics a custom widget contains.
    ///
    /// Call `receiver.graphic` for each graphic.
    fn graphics(&mut self, receiver: impl WidgetGraphicReceiver<P>);

    /// Override this method to define a custom shape for the widget.
    ///
    /// This is used by e.g. Button to test if it should handle a pointer
    /// event.  The default is a standard rectangular test.
    fn hittest(&self, extra: &mut WidgetExtra<'_>, point: (f32, f32)) -> bool {
        extra.contains(point)
    }

    /// Override this method to handle pointer events directly by a custom
    /// widget.
    ///
    /// Return true if this successfully handled the event.
    fn pointer_event(
        &mut self,
        extra: &mut WidgetExtra<'_>,
        event: &mut PointerEvent,
    ) -> bool {
        let _unused = (extra, event);
        false
    }

    /// This is the same as `pointer_event`, except that it runs before
    /// passing the event to children, rather than after.  This is only
    /// recomended for special cases.
    fn pointer_event_before(
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
    fn init(_init: impl WidgetInit<Self, P>) {}
    fn children(&mut self, _receiver: impl WidgetChildReceiver<P>) {}
    fn graphics(&mut self, _receiver: impl WidgetGraphicReceiver<P>) {}
}

fn run_widget_as_app<T>() -> !
where
    T: Default + WidgetContent<DefaultRenderPlatform>,
{
    use crate::app::{App, AppBuilder};
    use crate::window::WindowSettings;

    let name = std::any::type_name::<T>()
        .rsplit("::")
        .next()
        .expect("Iterator Returnned by str::rsplit was empty.");
    let (_, title) =
        name.chars()
            .fold((false, String::new()), |(prev, mut title), ch| {
                if prev && ch.is_uppercase() {
                    title.push(' ');
                }
                title.push(ch);
                (ch.is_lowercase(), title)
            });
    let mut builder = AppBuilder::default();
    builder.set_title(title);
    let app: App<DefaultPlatform> = builder.build();
    let (app, _) = app.with(|current| {
        current.add_root(super::Widget::<T>::default);
    });
    app.run();
}
