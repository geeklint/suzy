//! This module contains the built-in implementations of the Suzy platform.

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
pub use self::sdl2::SdlPlatform as DefaultPlatform;

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
pub use self::sdl2::SdlPlatform as TestPlatform;

/// The default Platform used for tests is determined by the cargo features
/// enabled on this crate.
#[cfg(any(
    feature = "platform_osmesa",
    all(test, feature = "platform_osmesa_test"),
))]
pub use self::osmesa::OsMesaPlatform as TestPlatform;

/// The default RenderPlatform is determined by the cargo features enabled on
/// this crate.
pub type DefaultRenderPlatform =
    <DefaultPlatform as crate::platform::Platform>::Renderer;

/// The default RenderPlatform used for tests is determined by the cargo
/// features enabled on this crate.
pub type TestRenderPlatform =
    <TestPlatform as crate::platform::Platform>::Renderer;
