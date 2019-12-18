
#[cfg(feature = "tui")]
mod tui;
#[cfg(feature = "tui")]
pub use tui::*;

#[cfg(not(any(feature = "tui", )))]
mod dummy;
#[cfg(not(any(feature = "tui", )))]
pub use dummy::*;

pub enum WindowEvent {
    Resize(f32, f32),
    KeyDown(i32),
}
