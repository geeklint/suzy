/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use std::rc::Rc;

use suzy::{
    dims::{Padding2d, Rect},
    platforms::opengl::{
        self, PopulateTexture, PopulateTextureUtil, SlicedImage, Texture,
    },
    widget::{self, *},
};

const IMAGE: &[u8] = include_bytes!("cute.data");
const IMAGE_WIDTH: u16 = 384;
const IMAGE_HEIGHT: u16 = 512;
const IMAGE_ASPECT: f32 = 384.0 / 512.0;

fn main() {
    ImageViewer::run_as_app();
}

// Root widget data
#[derive(Default)]
struct ImageViewer {
    image: SlicedImage,
}

impl widget::Content for ImageViewer {
    fn desc(mut desc: impl widget::Desc<Self>) {
        desc.watch(|this, _rect| {
            this.image.texture = Texture::new(Rc::new(Populator));
            this.image.padding = Padding2d::zero();
        });
        desc.watch(|this, rect| {
            // fill the screen with the image
            this.image.set_fill(&rect, &Padding2d::zero());
            // but shrink it so it stays the same aspect ratio
            this.image.shrink_to_aspect(IMAGE_ASPECT);
        });
        desc.graphic(|this| &mut this.image);
    }
}

struct Populator;

impl PopulateTexture for Populator {
    fn populate(
        &self,
        gl: &opengl::OpenGlBindings,
        target: opengl::opengl_bindings::types::GLenum,
    ) -> Result<opengl::TextureSize, String> {
        Ok(PopulateTextureUtil::populate_color_rgb(
            gl,
            target,
            IMAGE_WIDTH,
            IMAGE_HEIGHT,
            1,
            IMAGE,
        ))
    }

    fn texture_key(&self) -> &[u8] {
        "cute".as_bytes()
    }
}
