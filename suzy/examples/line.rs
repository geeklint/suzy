/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2025 Violet Leonard */

use suzy::{
    dims::Rect,
    graphics::Color,
    platforms::opengl::Line,
    widget::{self, RunAsApp},
};

#[derive(Default)]
struct LineExample {
    line: Line,
}

impl widget::Content for LineExample {
    fn desc(mut desc: impl widget::Desc<Self>) {
        desc.graphic(|this| &mut this.line);
        desc.watch(|this, rect| {
            let small_dim = rect.width().min(rect.height());
            let star_radius = small_dim * 0.33;
            this.line.points = [0.0_f32, 2.0, 4.0, 1.0, 3.0]
                .map(|arm| {
                    let angle = (360.0 * arm / 5.0 + 90.0).to_radians();
                    let x = star_radius * angle.cos() + rect.center_x();
                    let y = star_radius * angle.sin() + rect.center_y();
                    [x, y]
                })
                .to_vec();
            this.line.close_loop = true;
            this.line.width = 10.0;
            this.line.color = Color::from_rgba(1.0, 0.0, 1.0, 0.75);
        });
    }
}

fn main() {
    LineExample::run_as_app();
}
