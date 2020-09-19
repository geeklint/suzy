/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

extern crate suzy;

use suzy::dims::{Rect, SimplePadding2d};
use suzy::widget::*;
use suzy::platform::opengl::Texture;
use suzy::platform::opengl::SlicedImage;

const IMAGE: &[u8] = include_bytes!("cute.data");
const IMAGE_WIDTH: u16 = 384;
const IMAGE_HEIGHT: u16 = 512;
const IMAGE_ASPECT: f32 = 384.0 / 512.0;

// Root widget data
#[derive(Default)]
struct ImageViewer {
    image: SlicedImage,
}

impl WidgetContent for ImageViewer {
    fn init<I: WidgetInit<Self>>(mut init: I) {
        init.watch(|this, _rect| {
            this.image.set_image(
                Texture::from_rgb(IMAGE_WIDTH, IMAGE_HEIGHT, 1, IMAGE),
                &SimplePadding2d::zero(),
            );
        });
        init.watch(|this, rect| {
            // fill the screen with the image
            this.image.set_fill(&rect, &SimplePadding2d::zero());
            // but shrink it so it stays the same aspect ratio
            this.image.shrink_to_aspect(IMAGE_ASPECT);
        });
    }

    fn children<R: WidgetChildReceiver>(&mut self, _receiver: R) {
        // no widget children
    }

    fn graphics<R: WidgetGraphicReceiver>(&mut self, mut receiver: R) {
        receiver.graphic(&mut self.image);
    }
}

fn main() {
    ImageViewer::run_as_app();
}
