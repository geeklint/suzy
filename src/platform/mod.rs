
#[cfg(feature = "tui")]
mod tui;
#[cfg(feature = "tui")]
pub use tui::{};

pub enum WindowEvent {
    Resize(f32, f32),
    KeyDown(i32),
}
