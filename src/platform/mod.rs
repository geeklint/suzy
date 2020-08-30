/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

mod shared;
pub use shared::{
    Platform,
    RenderPlatform,
    Event,
    EventLoopState,
    SimpleEventLoopState,
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

