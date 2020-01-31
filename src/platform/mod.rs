
// Platforms

pub mod dummy;

#[cfg(feature = "tui")]
pub mod tui;

#[cfg(feature = "opengl")]
pub mod opengl;

#[cfg(feature = "sdl")]
pub mod sdl2;

// Re-exports

#[cfg(feature = "opengl")]
pub use opengl::DrawParams;
#[cfg(feature = "opengl")]
pub use opengl::graphics;

#[cfg(feature = "sdl")]
pub use self::sdl2::Window as DefaultWindow;
