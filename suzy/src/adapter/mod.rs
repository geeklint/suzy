/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

//! Adapters enable an arbitrarily large scrolling list of widgets to be
//! created using finite memory.
//!
//! See `examples/adapter.rs` for an example.
//!
//! An Adapter view maintains a collection of widgets which are re-used as
//! the view is scrolled to display new content.
//!
//! The Adaptable trait is the primary way for a Widget to update it's visuals
//! in response to a change in an external data source.

use crate::dims::Rect;
use crate::platform::DefaultRenderPlatform;
use crate::widget::Widget;

mod layout;
mod view;

pub use layout::{AdapterLayout, AdapterLayoutInterface};
pub use view::AdapterView;

/// Trait representing some view which may "adapt" to a specific change in
/// external data.
///
/// See the [module-level documentation](./index.html) for more details.
pub trait Adaptable<T: ?Sized> {
    /// Update `self` in acordance with the provided `data`
    fn adapt(&mut self, data: &T);

    /// Create a new instance from provided `data`
    fn from(data: &T) -> Self;
}

/// An adapter view which displays the contents of a Vec growing downwards.
pub type DownwardVecAdapter<T, W, P = DefaultRenderPlatform> =
    Widget<AdapterView<DownwardVecLayout<T>, W, P>, P>;

/// An adapter layout which lays out elements from a Vec growing downwards.
#[derive(Default)]
pub struct DownwardVecLayout<T> {
    data: Vec<T>,
    reference_index: usize,
    avg_size: f32,
}

impl<T> AdapterLayout for DownwardVecLayout<T> {
    type ElementKey = usize;
    type Collection = Vec<T>;
    type ElementData = T;

    fn data(&self) -> &Vec<T> {
        &self.data
    }

    fn data_mut(&mut self) -> &mut Vec<T> {
        &mut self.data
    }

    fn layout(&mut self, mut interface: impl AdapterLayoutInterface<Self>) {
        let (top, bottom) = {
            let bounds = interface.bounds();
            (bounds.top(), bounds.bottom())
        };
        let middle = (top + bottom) / 2.0;
        let mut cursor_back = top + interface.reference_position().1;
        let ref_index = self.reference_index;
        let prev_index = ref_index.saturating_sub(1);
        let mut cursor_fwd;
        let drawn_index;
        if self.data.is_empty() {
            // if the list is empty, don't render or change anything
            return;
        } else if let Some(value) = self.data.get(ref_index) {
            // position reference element
            let el = interface.get_element(ref_index, value);
            el.set_top(cursor_back);
            cursor_fwd = el.bottom();
            drawn_index = ref_index;
        } else if let Some(value) = self.data.get(prev_index) {
            // if we couldn't get the ref element, the list has shrunk
            // but if ref-1 exists, we can still position backwards
            cursor_fwd = cursor_back;
            let el = interface.get_element(prev_index, value);
            el.set_bottom(cursor_fwd);
            cursor_back = el.top();
            drawn_index = prev_index;
        } else {
            // we have nothing to go off of, so just update the rest position
            self.avg_size = 100.0;
            self.reference_index = self.data.len();
            let reference_update = (0.0, 0.0);
            let rest_update = (0.0, bottom - top);
            interface.update_positions(reference_update, rest_update);
            return;
        }
        struct Nearest {
            index: usize,
            pos: f32,
            dist: f32,
        }
        let mut nearest = Nearest {
            index: drawn_index,
            pos: cursor_back,
            dist: (cursor_back - middle).abs(),
        };
        // draw elements before the initially drawn one
        let mut index_back = drawn_index;
        while cursor_back < top && index_back > 0 {
            index_back -= 1;
            let el = interface.get_element(index_back, &self.data[index_back]);
            el.set_bottom(cursor_back);
            cursor_back = el.top();
            let dist = (cursor_back - middle).abs();
            if dist < nearest.dist {
                nearest = Nearest {
                    index: index_back,
                    pos: cursor_back,
                    dist,
                };
            }
        }
        let mut index_fwd = drawn_index;
        // draw elements after the initially drawn one
        while cursor_fwd > bottom && index_fwd < (self.data.len() - 1) {
            index_fwd += 1;
            let el = interface.get_element(index_fwd, &self.data[index_fwd]);
            el.set_top(cursor_fwd);
            let dist = (cursor_fwd - middle).abs();
            if dist < nearest.dist {
                nearest = Nearest {
                    index: index_fwd,
                    pos: cursor_fwd,
                    dist,
                };
            }
            cursor_fwd = el.bottom();
        }
        let count = interface.num_active_elements() as f32;
        self.avg_size = (cursor_back - cursor_fwd).abs() / count;
        let rest_pos = if cursor_back < top {
            nearest.pos + (top - cursor_back)
        } else if cursor_fwd > bottom {
            let limit = if index_back == 0 {
                // prevent wiggling if the whole list is smaller than the view
                cursor_back - top
            } else {
                f32::INFINITY
            };
            nearest.pos - (cursor_fwd - bottom).min(limit)
        } else {
            nearest.pos
        };
        self.reference_index = nearest.index;
        let reference_update = (0.0, nearest.pos - top);
        let rest_update = (0.0, rest_pos - top);
        interface.update_positions(reference_update, rest_update);
    }

    fn element_location(
        &mut self,
        item: &Self::ElementKey,
        reference_position: (f32, f32),
    ) -> Option<(f32, f32)> {
        let item = *item;
        let dist = if item >= self.data.len() {
            return None;
        } else if item > self.reference_index {
            (item - self.reference_index) as f32 * -1.0
        } else {
            (self.reference_index - item) as f32
        };
        let pos = reference_position.1 + (dist * self.avg_size);
        Some((reference_position.0, pos))
    }
}
