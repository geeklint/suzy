/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

#![allow(missing_docs)]

use crate::{
    graphics::Color,
    platforms::opengl::OpenGlRenderPlatform,
    pointer::{AltMouseButton, PointerAction, PointerEventData, PointerId},
    watch::WatchedValueCore,
};

mod window;

pub use window::{Window, WindowSettings};

pub struct SdlPlatform {
    pub sdl: sdl2::Sdl,
}

impl crate::platform::Platform for SdlPlatform {
    type Renderer = OpenGlRenderPlatform;
}

impl SdlPlatform {
    pub fn new() -> Self {
        SdlPlatform {
            sdl: sdl2::init().expect("Failed to initialize SDL2"),
        }
    }

    pub fn run(
        self,
        window: &mut window::Window,
        app: &mut crate::app::App<Self>,
    ) -> Result<(), String> {
        let mut event_pump = self.sdl.event_pump()?;
        loop {
            use sdl2::event::{Event, WindowEvent};
            app.start_frame(std::time::Instant::now());
            while let Some(event) = event_pump.poll_event() {
                match event {
                    Event::Quit { .. }
                    | Event::Window {
                        win_event: WindowEvent::Close,
                        ..
                    } => {
                        return Ok(());
                    }
                    event => app.handle_event(window, event, || {
                        let state = event_pump.mouse_state();
                        (state.x() as f32, state.y() as f32)
                    }),
                }
            }
            app.update_watches();
            window.gl_win.draw_app(app);
            window.flip();
        }
    }
}

impl Default for SdlPlatform {
    fn default() -> Self {
        Self::new()
    }
}

pub trait AppHandleSdlEvent {
    fn handle_event(
        &mut self,
        window: &mut window::Window,
        event: sdl2::event::Event,
        mouse_pos: impl FnOnce() -> (f32, f32),
    );
}

impl AppHandleSdlEvent for crate::app::App<SdlPlatform> {
    fn handle_event(
        &mut self,
        window: &mut window::Window,
        event: sdl2::event::Event,
        mouse_pos: impl FnOnce() -> (f32, f32),
    ) {
        use sdl2::event::{Event, WindowEvent};
        match event {
            Event::Window { win_event, .. } => {
                match win_event {
                    WindowEvent::SizeChanged(_, _)
                    | WindowEvent::Moved { .. } => {
                        let [width, height] = window.logical_size();
                        self.resize(width, height);
                        self.update_dpi(window.dpi());
                        window.recalculate_viewport();
                    }
                    WindowEvent::Leave => {
                        self.pointer_event(PointerEventData {
                            id: PointerId::Mouse,
                            action: PointerAction::Hover(f32::NAN, f32::NAN),
                            x: f32::NAN,
                            y: f32::NAN,
                        });
                    }
                    _ => {}
                };
            }
            Event::MouseButtonDown {
                mouse_btn, x, y, ..
            } => {
                let height = self.state().window_height().get_unwatched();
                let [x, y] = [x as f32, height - y as f32];
                let action = match mouse_btn.to_suzy_mouse_button() {
                    AltMouseButtonResult::Primary => PointerAction::Down,
                    AltMouseButtonResult::Alt(btn) => {
                        PointerAction::AltDown(btn)
                    }
                    AltMouseButtonResult::Unknown => return,
                };
                self.pointer_event(PointerEventData {
                    id: PointerId::Mouse,
                    action,
                    x,
                    y,
                });
            }
            Event::MouseButtonUp {
                mouse_btn, x, y, ..
            } => {
                let height = self.state().window_height().get_unwatched();
                let [x, y] = [x as f32, height - y as f32];
                let action = match mouse_btn.to_suzy_mouse_button() {
                    AltMouseButtonResult::Primary => PointerAction::Up,
                    AltMouseButtonResult::Alt(btn) => {
                        PointerAction::AltUp(btn)
                    }
                    AltMouseButtonResult::Unknown => return,
                };
                self.pointer_event(PointerEventData {
                    id: PointerId::Mouse,
                    action,
                    x,
                    y,
                });
            }
            Event::MouseMotion {
                mousestate,
                x,
                y,
                xrel,
                yrel,
                ..
            } => {
                let height = self.state().window_height().get_unwatched();
                let [x, y] = [x as f32, height - y as f32];
                let [xrel, yrel] = [xrel as f32, -(yrel as f32)];
                let pointer_event = if mousestate.left() {
                    PointerEventData {
                        id: PointerId::Mouse,
                        action: PointerAction::Move(xrel, yrel),
                        x,
                        y,
                    }
                } else {
                    PointerEventData {
                        id: PointerId::Mouse,
                        action: PointerAction::Hover(xrel, yrel),
                        x,
                        y,
                    }
                };
                self.pointer_event(pointer_event);
            }
            Event::MouseWheel { x, y, .. } => {
                let height = self.state().window_height().get_unwatched();
                let (mouse_x, mouse_y) = mouse_pos();
                let xrel = x as f32 * 125.0;
                let yrel = -(y as f32 * 125.0);
                let x = mouse_x;
                let y = height - mouse_y;
                self.pointer_event(PointerEventData {
                    id: PointerId::Mouse,
                    action: PointerAction::Wheel(xrel, yrel),
                    x,
                    y,
                });
            }
            _ => {}
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AltMouseButtonResult {
    Primary,
    Alt(AltMouseButton),
    Unknown,
}

pub trait ToSuzyMouseButton {
    fn to_suzy_mouse_button(self) -> AltMouseButtonResult;
}

impl ToSuzyMouseButton for sdl2::mouse::MouseButton {
    fn to_suzy_mouse_button(self) -> AltMouseButtonResult {
        use AltMouseButtonResult::*;
        match self {
            sdl2::mouse::MouseButton::Unknown => Unknown,
            sdl2::mouse::MouseButton::Left => Primary,
            sdl2::mouse::MouseButton::Middle => Alt(AltMouseButton::Middle),
            sdl2::mouse::MouseButton::Right => Alt(AltMouseButton::Right),
            sdl2::mouse::MouseButton::X1 => Alt(AltMouseButton::X1),
            sdl2::mouse::MouseButton::X2 => Alt(AltMouseButton::X2),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct TestEnvironment;

impl super::TestEnvironment for TestEnvironment {
    unsafe fn initialize(
        &self,
        width: u16,
        height: u16,
    ) -> Box<dyn AsMut<super::opengl::Window>> {
        let sdl = sdl2::init().expect("Failed to initialize SDL2");
        let window = Window::new_window(
            &sdl,
            WindowSettings {
                title: "Suzy Test",
                width: width.into(),
                height: height.into(),
                background_color: Color::BLACK,
            },
        )
        .expect("failed to open window");
        struct Wrapper {
            window: Window,
        }
        impl AsMut<super::opengl::Window> for Wrapper {
            fn as_mut(&mut self) -> &mut super::opengl::Window {
                &mut self.window.gl_win
            }
        }
        Box::new(Wrapper { window })
    }
}
