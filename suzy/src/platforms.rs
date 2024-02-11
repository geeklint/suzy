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

#[cfg(feature = "platform_opengl")]
pub trait TestEnvironment {
    /// # Safety
    ///
    /// OpenGL uses global state which can cause unsoundness if user code or
    /// other crates adjust things that Suzy cannot account for. The safety
    /// precondition of this function is: if any other crate is making OpenGL
    /// calls, you have reviewed the semantics of all of the OpenGL code for
    /// potential errors.
    ///
    /// This function is intended to be safe when not combined with sources of
    /// interaction with the OpenGL state from outside Suzy.
    unsafe fn initialize(
        &self,
        width: u16,
        height: u16,
    ) -> Box<dyn std::ops::DerefMut<Target = opengl::Window>>;
}

#[cfg(any(feature = "platform_osmesa", feature = "platform_sdl"))]
#[allow(unreachable_code)]
pub const TEST_ENV: &dyn TestEnvironment = 'test_env: {
    #[cfg(feature = "platform_osmesa")]
    {
        break 'test_env &osmesa::TestEnvironment;
    }
    #[cfg(feature = "platform_sdl")]
    {
        break 'test_env &sdl2::TestEnvironment;
    }
};

#[cfg(feature = "platform_opengl")]
macro_rules! with_default_render_platform {
    ($(#[$Attr:meta])* pub $Def:ident $Item:ident < $T:ident, $P:ident > $($body:tt)* ) => {
        $(#[$Attr])*
        pub $Def $Item < $T, $P = crate::platforms::DefaultRenderPlatform >
            $($body)*
    };
    ($(#[$Attr:meta])* pub $Def:ident $Item:ident < $P:ident > $($body:tt)* ) => {
        $(#[$Attr])*
        pub $Def $Item < $P = crate::platforms::DefaultRenderPlatform >
            $($body)*
    };
}

#[cfg(not(feature = "platform_opengl"))]
macro_rules! with_default_render_platform {
    ($(#[$Attr:meta])* pub $Def:ident $Item:ident < $T:ident, $P:ident > $($body:tt)* ) => {
        $(#[$Attr])*
        pub $Def $Item < $T, $P >
            $($body)*
    };
    ($(#[$Attr:meta])* pub $Def:ident $Item:ident < $P:ident > $($body:tt)* ) => {
        $(#[$Attr])*
        pub $Def $Item < $P >
            $($body)*
    };
}

#[cfg(feature = "platform_opengl")]
#[derive(Clone, Copy, Debug, Default)]
pub struct TestPlatform;

#[cfg(feature = "platform_opengl")]
impl crate::platform::Platform for TestPlatform {
    type Renderer = opengl::OpenGlRenderPlatform;
}
