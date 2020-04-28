use std::convert::TryFrom;

use crate::graphics::DrawContext;
use crate::platform::RenderPlatform;
use crate::pointer::PointerEventData;

pub enum WindowEvent {
    Resize(f32, f32),
    DpScaleChange(f32),
    KeyDown(i32),
    Quit,
    Pointer(PointerEventData),
}

pub trait WindowSettings {
    /// Get the size of the window in dp
    fn size(&self) -> (f32, f32);
    
    /// Set the size of the window in dp
    fn set_size(&mut self, size: (f32, f32));

    /// Get the window title
    fn title(&self) -> &str;

    /// Set the window title
    fn set_title(&mut self, title: String);

    /// Get the window fullscreen state
    fn fullscreen(&self) -> bool;

    /// Set the fullscreen state
    fn set_fullscreen(&mut self, fullscreen: bool);
}

/// A structure which defines the initial creation parameters for a window
pub struct WindowBuilder {
    size: (f32, f32),
    title: String,
    fullscreen: bool,
}

impl WindowBuilder {
    pub fn into_title(self) -> String { self.title }
}

impl Default for WindowBuilder {
    fn default() -> Self {
        Self {
            size: (1366.0, 768.0),
            title: "Suzy Window".to_string(),
            fullscreen: false,
        }
    }
}

impl WindowSettings for WindowBuilder {
    fn size(&self) -> (f32, f32) { self.size }
    
    fn set_size(&mut self, size: (f32, f32)) {
        self.size = size;
    }

    fn title(&self) -> &str { &self.title }

    fn set_title(&mut self, title: String) {
        self.title = title;
    }

    fn fullscreen(&self) -> bool { self.fullscreen }

    fn set_fullscreen(&mut self, fullscreen: bool) {
        self.fullscreen = fullscreen;
    }
}

pub trait Window<P: RenderPlatform> : WindowSettings + TryFrom<WindowBuilder> {
    /// Get the pixel density of the window as displayed
    fn pixels_per_dp(&self) -> f32;

    /// Clear the screen
    fn clear(&mut self);

    /// Do some sort of synchonization - this function is expected to block
    /// for some period of time. In a double buffered context, this will
    /// usually cause the back buffer to be displayed.
    fn flip(&mut self);

    fn prepare_draw(&mut self) -> DrawContext<P>;

    /// Check for a new event on the window
    fn next_event(&mut self) -> Option<WindowEvent>;
}
