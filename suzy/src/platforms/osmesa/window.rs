/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use crate::window::Window;

use crate::platforms::opengl;

pub struct OsMesaWindow {
    width: u16,
    height: u16,
}

impl Window<opengl::OpenGlRenderPlatform> for OsMesaWindow {
    fn size(&self) -> [f32; 2] {
        [self.width.into(), self.height.into()]
    }
}
