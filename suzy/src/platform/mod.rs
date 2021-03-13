/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A flexible interface for the low-level aspects of the GUI system.
//!
//! One render platform is currently included, based on OpenGL ES 2.0
//!
//! Two windowing platforms are currently included; the primary is based on
//! SDL2.  A windowless OSMesa interface is also included, intended for
//! automated testing.
//!
//! If no platforms are enabled (using features) the default platform is
//! a "stub" platform which has no functionality (all methods panic).

mod shared;
pub use shared::{
    Event, EventLoopState, Platform, RenderPlatform, SimpleEventLoopState,
};

// Platforms

// stub types are used in case no other defaults are available - all their
// methods panic
mod stub;

#[cfg(feature = "platform_opengl")]
pub mod opengl;

#[cfg(feature = "sdl")]
pub mod sdl2;

#[cfg(any(
    feature = "platform_osmesa",
    all(test, feature = "platform_osmesa_test"),
))]
pub mod osmesa;

// Default Platform

/// The default Platform is determined by the cargo features enabled on
/// this crate.
#[cfg(not(feature = "platform_opengl"))]
pub use self::stub::StubPlatform as DefaultPlatform;

/// The default Platform is determined by the cargo features enabled on
/// this crate.
#[cfg(all(feature = "platform_opengl", not(feature = "sdl")))]
pub use self::stub::StubOpenglPlatform as DefaultPlatform;

/// The default Platform is determined by the cargo features enabled on
/// this crate.
#[cfg(feature = "sdl")]
pub use self::sdl2::SDLPlatform as DefaultPlatform;

// Platform used for tests

/// The default Platform used for tests is determined by the cargo features
/// enabled on this crate.
#[cfg(not(feature = "platform_opengl"))]
pub use self::stub::StubPlatform as TestPlatform;

/// The default Platform used for tests is determined by the cargo features
/// enabled on this crate.
#[cfg(all(
    feature = "platform_opengl",
    not(feature = "sdl"),
    not(feature = "platform_osmesa"),
    not(all(test, feature = "platform_osmesa_test")),
))]
pub use self::stub::StubOpenglPlatform as TestPlatform;

/// The default Platform used for tests is determined by the cargo features
/// enabled on this crate.
#[cfg(all(
    feature = "sdl",
    not(feature = "platform_osmesa"),
    not(all(test, feature = "platform_osmesa_test")),
))]
pub use self::sdl2::SDLPlatform as TestPlatform;

/// The default Platform used for tests is determined by the cargo features
/// enabled on this crate.
#[cfg(any(
    feature = "platform_osmesa",
    all(test, feature = "platform_osmesa_test"),
))]
pub use self::osmesa::OSMesaPlatform as TestPlatform;

/// The default RenderPlatform is determined by the cargo features enabled on
/// this crate.
pub type DefaultRenderPlatform = <DefaultPlatform as Platform>::Renderer;

/// The default RenderPlatform used for tests is determined by the cargo
/// features enabled on this crate.
pub type TestRenderPlatform = <TestPlatform as Platform>::Renderer;
