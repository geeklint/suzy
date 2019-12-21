
#[cfg(feature = "tui")]
mod tui;
#[cfg(feature = "tui")]
pub use tui::*;

#[cfg(feature = "sdl")]
mod sdl;
#[cfg(feature = "sdl")]
pub use sdl::*;

#[cfg(not(any(feature = "tui", feature = "sdl")))]
mod dummy;
#[cfg(not(any(feature = "tui", feature = "sdl")))]
pub use dummy::*;

pub enum WindowEvent {
    Resize(f32, f32),
    KeyDown(i32),
}
