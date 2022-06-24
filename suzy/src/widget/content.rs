/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use crate::dims::Rect;
use crate::platform::{DefaultPlatform, DefaultRenderPlatform};
use crate::pointer::PointerEvent;

use super::{WidgetDescReceiver, WidgetExtra, WidgetInit};

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
/// # use suzy::widget::{self, *};
/// # use suzy::selectable::SelectableIgnored;
/// # type ButtonContent = SelectableIgnored<()>;
/// use suzy::widgets::Button;
///
/// struct MyWidgetData {
///     button_one: Button<ButtonContent>,
///     button_two: Button<ButtonContent>,
/// }
///
/// impl widget::Content for MyWidgetData {
///     // ...
/// #   fn init(_init: impl WidgetInit<Self>) {}
///
///     fn desc(mut receiver: impl WidgetDescReceiver<Self>) {
///         receiver.child(|this| &mut this.button_one);
///         receiver.child(|this| &mut this.button_two);
///     }
/// }
/// ```
///
/// Or, if the custom widget only has a single graphic:
///
/// ```rust
/// # use suzy::widget::{self, *};
/// # type MyGraphic = ();
///
/// struct MyWidgetData {
///     graphic: MyGraphic,
/// }
///
/// impl widget::Content for MyWidgetData {
///     // ...
/// #   fn init(_init: impl WidgetInit<Self>) {}
///
///     fn desc(mut receiver: impl WidgetDescReceiver<Self>) {
///         receiver.graphic(|this| &mut this.graphic);
///     }
/// }
/// ```
///
pub trait Content<P = DefaultRenderPlatform>
where
    Self: 'static,
{
    /// This method provides a convient place to register functions which
    /// watch values and update parts of the widget when they change.
    fn init(init: impl WidgetInit<Self, P>);

    /// Use this method to specify the children a custom widget contains.
    ///
    /// Call `receiver.child` for each child.
    fn desc(receiver: impl WidgetDescReceiver<Self, P>);

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
}

impl<P> Content<P> for () {
    fn init(_init: impl WidgetInit<Self, P>) {}
    fn desc(_receiver: impl WidgetDescReceiver<Self, P>) {}
}

/// This is a convience function to create and run an App with this
/// content as the only initial root widget.
pub trait RunAsApp {
    fn run_as_app() -> !;
}

impl<T> RunAsApp for T
where
    T: Default + Content<DefaultRenderPlatform>,
{
    fn run_as_app() -> ! {
        use crate::app::{App, AppBuilder};
        use crate::window::WindowSettings;

        let name = std::any::type_name::<T>()
            .rsplit("::")
            .next()
            .expect("Iterator Returned by str::rsplit was empty.");
        let (_, title) = name.chars().fold(
            (false, String::new()),
            |(prev, mut title), ch| {
                if prev && ch.is_uppercase() {
                    title.push(' ');
                }
                title.push(ch);
                (ch.is_lowercase(), title)
            },
        );
        let mut builder = AppBuilder::default();
        builder.set_title(title);
        let app: App<DefaultPlatform> = builder.build();
        let (app, _) = app.with(|current| {
            current.add_root(super::Widget::<T>::default);
        });
        app.run();
    }
}
