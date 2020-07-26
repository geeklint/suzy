
mod shared;
pub use shared::{
    Platform,
    RenderPlatform,
    Event,
    EventLoopState,
};

// Platforms

#[cfg(feature = "tui")]
pub mod tui;

#[cfg(feature = "opengl")]
pub mod opengl;

#[cfg(feature = "sdl")]
pub mod sdl2;

#[cfg(feature = "opengl")]
pub use opengl::OpenGlRenderPlatform as DefaultRenderPlatform;

#[cfg(feature = "sdl")]
pub use self::sdl2::SDLPlatform as DefaultPlatform;

/*
#[cfg(feature = "opengl")]
pub use opengl::{
    graphics,
    Text,
    Font,
};
*/

