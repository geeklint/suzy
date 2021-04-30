/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

extern crate suzy;

use suzy::dims::{Rect, SimplePadding2d};
use suzy::platforms::opengl::SimpleImage;
use suzy::platforms::opengl::Texture;
use suzy::widget::*;

const IMAGE: &[u8] = include_bytes!("cute.data");
const IMAGE_WIDTH: u16 = 384;
const IMAGE_HEIGHT: u16 = 512;
const IMAGE_ASPECT: f32 = 384.0 / 512.0;

// Root widget data
#[derive(Default)]
struct ImageViewer {
    image: SimpleImage,
}

impl WidgetContent for ImageViewer {
    fn init(mut init: impl WidgetInit<Self>) {
        init.watch(|this, _rect| {
            this.image.set_image(Texture::from_rgb(
                IMAGE_WIDTH,
                IMAGE_HEIGHT,
                1,
                IMAGE,
            ));
        });
        init.watch(|this, rect| {
            // fill the screen with the image
            this.image.set_fill(&rect, &SimplePadding2d::zero());
            // but shrink it so it stays the same aspect ratio
            this.image.shrink_to_aspect(IMAGE_ASPECT);
        });
    }

    fn children(&mut self, _receiver: impl WidgetChildReceiver) {
        // no widget children
    }

    fn graphics(&mut self, mut receiver: impl WidgetGraphicReceiver) {
        receiver.graphic(&mut self.image);
    }
}

fn main() {
    ImageViewer::run_as_app();
}
