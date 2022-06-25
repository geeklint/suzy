/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

extern crate suzy;

use suzy::dims::{Rect, SimplePadding2d};
use suzy::platforms::opengl::SlicedImage;
use suzy::platforms::opengl::Texture;
use suzy::widget::{self, *};

const IMAGE: &[u8] = include_bytes!("cute.data");
const IMAGE_WIDTH: u16 = 384;
const IMAGE_HEIGHT: u16 = 512;
const IMAGE_ASPECT: f32 = 384.0 / 512.0;

// Root widget data
#[derive(Default)]
struct ImageViewer {
    image: SlicedImage,
}

impl widget::Content for ImageViewer {
    fn init(mut init: impl widget::Desc<Self>) {
        init.watch(|this, _rect| {
            this.image.set_image(
                Texture::from_rgb(IMAGE_WIDTH, IMAGE_HEIGHT, 1, IMAGE),
                SimplePadding2d::zero(),
            );
        });
        init.watch(|this, rect| {
            // fill the screen with the image
            this.image.set_fill(&rect, &SimplePadding2d::zero());
            // but shrink it so it stays the same aspect ratio
            this.image.shrink_to_aspect(IMAGE_ASPECT);
        });
    }

    fn desc(mut receiver: impl WidgetDescReceiver<Self>) {
        receiver.graphic(|this| &mut this.image);
    }
}

fn main() {
    ImageViewer::run_as_app();
}
