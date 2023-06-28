/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

//! This module contains the built-in implementations of the Suzy platform.

pub mod no_graphics;

#[cfg(feature = "platform_opengl")]
pub mod opengl;

#[cfg(feature = "platform_sdl")]
pub mod sdl2;

#[cfg(feature = "platform_osmesa")]
pub mod osmesa;

#[cfg(feature = "platform_sdl")]
pub type DefaultPlatform = self::sdl2::SdlPlatform;

#[cfg(feature = "platform_opengl")]
pub type DefaultRenderPlatform = self::opengl::OpenGlRenderPlatform;

#[cfg(feature = "platform_osmesa")]
pub type TestPlatform = self::osmesa::OsMesaPlatform;

#[cfg(all(not(feature = "platform_osmesa"), feature = "platform_sdl",))]
pub type TestPlatform = self::sdl2::SdlPlatform;

#[cfg(feature = "platform_opengl")]
macro_rules! with_default_render_platform {
    ($(#[$Attr:meta])* pub trait $Trait:ident < $T:ident, $P:ident > $($body:tt)* ) => {
        $(#[$Attr])*
        pub trait $Trait < $T, $P = crate::platforms::DefaultRenderPlatform >
            $($body)*
    };
    ($(#[$Attr:meta])* pub trait $Trait:ident < $P:ident > $($body:tt)* ) => {
        $(#[$Attr])*
        pub trait $Trait < $P = crate::platforms::DefaultRenderPlatform >
            $($body)*
    };
}

#[cfg(not(feature = "platform_opengl"))]
macro_rules! with_default_render_platform {
    ($(#[$Attr:meta])* pub trait $Trait:ident < $T:ident, $P:ident > $($body:tt)* ) => {
        $(#[$Attr])*
        pub trait $Trait < $T, $P >
            $($body)*
    };
    ($(#[$Attr:meta])* pub trait $Trait:ident < $P:ident > $($body:tt)* ) => {
        $(#[$Attr])*
        pub trait $Trait < $P >
            $($body)*
    };
}
