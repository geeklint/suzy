/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use crate::{
    pointer::PointerEvent,
    widget::{self, WidgetRect},
};

with_default_render_platform! {
    /// This trait provides the "glue" between the data you define in custom
    /// widgets and the behavior suzy defines for widgets.
    ///
    /// The desc method allows you to provide the description of your custom
    /// widget, which includes the child widgets, graphics, and `watch`
    /// functions that define the behavior of a widget. See the
    /// [watch](crate::watch) module for more information on `watch`
    /// functions.
    ///
    /// Custom widgets may contain any number of graphics and child widgets, or
    /// none of either.
    ///
    /// For example, if a custom widget contains two buttons as children:
    ///
    #[cfg_attr(
        feature = "platform-opengl",
        doc = "```rust
# use suzy::widget;
# use suzy::selectable::SelectableIgnored;
# type ButtonContent = SelectableIgnored<()>;
use suzy::widgets::Button;

struct MyWidgetData {
    button_one: Button<ButtonContent>,
    button_two: Button<ButtonContent>,
}

impl widget::Content for MyWidgetData {
    fn desc(mut desc: impl widget::Desc<Self>) {
        desc.child(|this| &mut this.button_one);
        desc.child(|this| &mut this.button_two);
    }
}
```"
    )]
    ///
    /// Or, if the custom widget only has a single graphic:
    ///
    #[cfg_attr(
        feature = "platform-opengl",
        doc = "```rust
# use suzy::widget::{self, *};
# type MyGraphic = ();

struct MyWidgetData {
    graphic: MyGraphic,
}

impl widget::Content for MyWidgetData {
    fn desc(mut desc: impl widget::Desc<Self>) {
        desc.graphic(|this| &mut this.graphic);
    }
}
```"
    )]
    pub trait Content<P>
    where
        Self: 'static,
    {
        /// This method should be implemented to describe a custom widget,
        /// including watch functions, children, graphics, and more.
        fn desc(desc: impl widget::Desc<Self, P>);

        /// Override this method to define a custom shape for the widget.
        ///
        /// This is used by e.g. Button to test if it should handle a pointer
        /// event.  The default is a standard rectangular test.
        fn hittest(&self, rect: &WidgetRect, point: [f32; 2]) -> bool {
            use crate::dims::Rect;
            rect.contains(point)
        }

        /// Override this method to handle pointer events directly by a custom
        /// widget.
        ///
        /// Return true if this successfully handled the event.
        fn pointer_event(
            &mut self,
            rect: &WidgetRect,
            event: &mut PointerEvent<'_>,
        ) -> bool {
            let _unused = (rect, event);
            false
        }

        /// This is the same as `pointer_event`, except that it runs before
        /// passing the event to children, rather than after.  This is only
        /// recomended for special cases.
        fn pointer_event_before(
            &mut self,
            rect: &WidgetRect,
            event: &mut PointerEvent<'_>,
        ) -> bool {
            let _unused = (rect, event);
            false
        }

        /// If this widget is used with RunAsApp, defines the window title.
        #[must_use]
        fn app_title() -> String {
            let name = std::any::type_name::<Self>()
                .rsplit("::")
                .next()
                .expect("Iterator Returned by str::rsplit was empty.");
            let (_accum, title) = name.chars().fold(
                (false, String::new()),
                |(prev, mut title), ch| {
                    if prev && ch.is_uppercase() {
                        title.push(' ');
                    }
                    title.push(ch);
                    (ch.is_lowercase(), title)
                },
            );
            title
        }
    }
}

impl<P> Content<P> for () {
    fn desc(_desc: impl widget::Desc<Self, P>) {}
}

/// This is a convience function to create and run an App with this
/// content as the only initial root widget.
///
/// Note: this is intended to be a quick way to run a Suzy app, but it may be
/// unsound in the presence of other crates which interact with the OpenGL
/// context. This isn't a typical use-case, and users may be mislead if it was
/// an unsafe fn, but you may avoid this unsoundness by using the more explict
/// API(s).
#[cfg(feature = "quickstart")]
pub trait RunAsApp {
    /// Run an app.
    ///
    /// See [trait-level documentation](RunAsApp) for more information.
    fn run_as_app() -> !;
}

#[cfg(feature = "quickstart")]
impl<T> RunAsApp for T
where
    T: Default + Content<crate::platforms::DefaultRenderPlatform>,
{
    fn run_as_app() -> ! {
        use crate::{
            app::App,
            platforms::sdl2::{Window, WindowSettings},
        };

        let platform = crate::platforms::sdl2::SdlPlatform {
            sdl: sdl2::init().expect("Failed to initialize SDL2"),
        };
        let mut window = Window::new_window(
            &platform.sdl,
            WindowSettings {
                title: &T::app_title(),
                ..WindowSettings::default()
            },
        )
        .expect("Failed to open window");
        let [width, height] = window.logical_size();
        let mut app = App::new(width, height);
        app.add_root(widget::Widget::<T>::default());
        let code: i32 = match platform.run(&mut window, &mut app) {
            Ok(()) => 0,
            Err(_) => 1,
        };
        std::process::exit(code)
    }
}
