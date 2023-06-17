/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use crate::dims::Rect;

#[derive(Clone, Copy, Debug)]
pub struct BoundingBox {
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
}

impl Default for BoundingBox {
    fn default() -> Self {
        Self {
            left: f32::NAN,
            right: f32::NAN,
            bottom: f32::NAN,
            top: f32::NAN,
        }
    }
}

impl<R: Rect> From<&R> for BoundingBox {
    fn from(value: &R) -> Self {
        Self {
            left: value.left(),
            right: value.right(),
            bottom: value.bottom(),
            top: value.top(),
        }
    }
}

impl BoundingBox {
    pub fn area(&self) -> f32 {
        let area = (self.right - self.left) * (self.top - self.bottom);
        if area.is_nan() {
            0.0
        } else {
            area
        }
    }

    pub fn overlaps(&self, other: &Self) -> bool {
        let x_overlaps = self.left < other.right && other.left < self.right;
        let y_overlaps = self.bottom < other.top && other.bottom < self.top;
        x_overlaps && y_overlaps
    }

    pub fn merge(&self, other: &Self) -> Self {
        Self {
            left: self.left.min(other.left),
            right: self.right.max(other.right),
            bottom: self.bottom.min(other.bottom),
            top: self.top.max(other.top),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct CoveredArea {
    boxes: [BoundingBox; 2],
}

impl CoveredArea {
    pub fn overlaps(&self, bb: &BoundingBox) -> bool {
        let [a, b] = &self.boxes;
        a.overlaps(bb) || b.overlaps(bb)
    }

    pub fn add_covered(&mut self, new: &BoundingBox) {
        let [a, b] = &self.boxes;
        let possible =
            [[a.merge(b), *new], [a.merge(new), *b], [*a, b.merge(new)]];
        self.boxes = <[_; 3]>::into_iter(possible)
            .min_by(|[a, b], [c, d]| {
                let left = a.area() + b.area();
                let right = c.area() + d.area();
                left.total_cmp(&right)
            })
            .expect("array iterator can't be empty");
    }
}
