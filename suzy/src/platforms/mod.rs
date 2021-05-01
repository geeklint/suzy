/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This module contains the built-in implementations of the Suzy platform.

// stub types are used in case no other defaults are available - all their
// methods panic
mod stub;

#[cfg(feature = "platform_opengl")]
pub mod opengl;

#[cfg(feature = "platform_sdl")]
pub mod sdl2;

#[cfg(feature = "platform_osmesa")]
pub mod osmesa;

/// The default Platform is determined by the cargo features enabled on
/// this crate.
pub type DefaultPlatform = _DefaultPlatform;

/// The default Platform used for tests is determined by the cargo features
/// enabled on this crate.
pub type TestPlatform = _TestPlatform;

/// The default RenderPlatform is determined by the cargo features enabled on
/// this crate.
pub type DefaultRenderPlatform =
    <DefaultPlatform as crate::platform::Platform>::Renderer;

/// The default RenderPlatform used for tests is determined by the cargo
/// features enabled on this crate.
pub type TestRenderPlatform =
    <TestPlatform as crate::platform::Platform>::Renderer;

// Determe DefaultPlatform by checking cargo features

#[cfg(feature = "platform_sdl")]
use self::sdl2::SdlPlatform as _DefaultPlatform;

#[cfg(all(not(feature = "platform_sdl"), feature = "platform_opengl"))]
use self::stub::StubOpenglPlatform as _DefaultPlatform;

#[cfg(all(
    not(feature = "platform_sdl"),
    not(feature = "platform_opengl")
))]
use self::stub::StubPlatform as _DefaultPlatform;

// Determe TestPlatform by checking cargo features

#[cfg(feature = "platform_osmesa")]
pub use self::osmesa::OsMesaPlatform as _TestPlatform;

#[cfg(all(not(feature = "platform_osmesa"), feature = "platform_sdl",))]
pub use self::sdl2::SdlPlatform as _TestPlatform;

#[cfg(all(
    not(feature = "platform_osmesa"),
    not(feature = "platform_sdl"),
    feature = "platform_opengl",
))]
pub use self::stub::StubOpenglPlatform as _TestPlatform;

#[cfg(all(
    not(feature = "platform_osmesa"),
    not(feature = "platform_sdl"),
    not(feature = "platform_opengl"),
))]
pub use self::stub::StubPlatform as _TestPlatform;
