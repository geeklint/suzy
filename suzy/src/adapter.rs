/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

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

use std::collections::HashMap;

use crate::dims::Rect;
use crate::platform::{DefaultRenderPlatform, RenderPlatform};
use crate::pointer::PointerId;
use crate::watch::WatchedMeta;
use crate::widget::{
    Widget, WidgetChildReceiver, WidgetContent, WidgetGraphicReceiver,
    WidgetInit, WidgetRect,
};

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

/// An implementation of this trait is passed to AdapterLayout implementations.
///
/// It provides the utilities an adapter layout uses to control the view.
pub trait AdapterLayoutInterface<Layout: AdapterLayout + ?Sized> {
    /// A rectangle which represents the extants of the view. Returned by
    /// the `bounds` method.
    type Bounds: Rect;

    /// A rectangle which represents an individual element to be positioned
    /// by the layout.
    type Element: Rect;

    /// Get the position last set with `update_reference`, offset by any
    /// scrolling that has happened since.
    fn reference_position(&self) -> (f32, f32);

    /// Get the rectangle which represents the extants of the view.
    fn bounds(&self) -> &Self::Bounds;

    /// Update the reference and rest positions.
    ///
    /// The rest position can be used in the case of overscroll to return the
    /// list to a preferred position.
    fn update_positions(
        &mut self,
        reference_position: (f32, f32),
        rest_position: (f32, f32),
    );

    /// Get the number of times get_element has been called during this
    /// layout operation.
    fn num_active_elements(&self) -> usize;

    /// Get or construct a widget from some data.
    fn get_element(
        &mut self,
        key: Layout::ElementKey,
        data: &Layout::ElementData,
    ) -> &mut Self::Element;
}

/// An adapter layout defines how elements in the adapter view are organized.
pub trait AdapterLayout {
    /// The key for looking up existing widgets.
    type ElementKey: std::hash::Hash + Eq;

    /// The type of the collection the layout understands.
    type Collection: ?Sized;

    /// The type of the data widgets will need to adapt to.
    type ElementData;

    /// Get a reference to the collection.
    fn data(&self) -> &Self::Collection;

    /// Get a mutable reference to the collection.
    fn data_mut(&mut self) -> &mut Self::Collection;

    /// Execute the layout, using the provided interface to position elements.
    fn layout(&mut self, interface: impl AdapterLayoutInterface<Self>);

    /// Get an approximate location of an element that is not currently in
    /// view.
    fn element_location(
        &mut self,
        item: &Self::ElementKey,
        reference_position: (f32, f32),
    ) -> Option<(f32, f32)>;
}

struct AdapterData<Layout, Content, Platform>
where
    Layout: AdapterLayout,
    Content: WidgetContent<Platform> + Adaptable<Layout::ElementData>,
    Platform: RenderPlatform,
{
    active: HashMap<Layout::ElementKey, Widget<Content, Platform>>,
    inactive: Vec<Widget<Content, Platform>>,
    position: (f32, f32),
    rest_position: (f32, f32),
}

struct Interface<'a, Layout, Content, Platform>
where
    Layout: AdapterLayout,
    Content: WidgetContent<Platform> + Adaptable<Layout::ElementData>,
    Platform: RenderPlatform,
{
    rect: &'a WidgetRect,
    data: &'a mut AdapterData<Layout, Content, Platform>,
    prev: HashMap<Layout::ElementKey, Widget<Content, Platform>>,
}

impl<'a, Layout, Content, Platform> AdapterLayoutInterface<Layout>
    for Interface<'a, Layout, Content, Platform>
where
    Layout: AdapterLayout,
    Content: WidgetContent<Platform> + Adaptable<Layout::ElementData>,
    Platform: RenderPlatform,
{
    type Bounds = WidgetRect;
    type Element = Widget<Content, Platform>;

    fn reference_position(&self) -> (f32, f32) {
        self.data.position
    }

    fn bounds(&self) -> &WidgetRect {
        &self.rect
    }

    fn update_positions(
        &mut self,
        position: (f32, f32),
        rest_position: (f32, f32),
    ) {
        self.data.position = position;
        self.data.rest_position = rest_position;
    }

    fn num_active_elements(&self) -> usize {
        self.data.active.len()
    }

    fn get_element(
        &mut self,
        key: Layout::ElementKey,
        data: &Layout::ElementData,
    ) -> &mut Self::Element {
        use std::collections::hash_map::Entry;
        match self.data.active.entry(key) {
            Entry::Occupied(bucket) => bucket.into_mut(),
            Entry::Vacant(bucket) => {
                let inactive = &mut self.data.inactive;
                // try to get from previous
                let element = self
                    .prev
                    .remove(bucket.key())
                    // otherwise try to get anything existing and adapt it
                    .or_else(|| {
                        inactive.pop().map(|mut el| {
                            el.adapt(data);
                            el
                        })
                    })
                    // otherwise, create a new widget
                    .unwrap_or_else(|| Adaptable::from(data));
                bucket.insert(element)
            }
        }
    }
}

impl<'a, Layout, Content, Platform> Drop
    for Interface<'a, Layout, Content, Platform>
where
    Layout: AdapterLayout,
    Content: WidgetContent<Platform> + Adaptable<Layout::ElementData>,
    Platform: RenderPlatform,
{
    fn drop(&mut self) {
        let remaining = std::mem::take(&mut self.prev);
        self.data
            .inactive
            .extend(remaining.into_iter().map(|(_k, v)| v));
    }
}

/// Base adapter view.
pub struct AdapterView<Layout, Content, Platform>
where
    Layout: AdapterLayout,
    Content: WidgetContent<Platform> + Adaptable<Layout::ElementData>,
    Platform: RenderPlatform,
{
    inner: AdapterData<Layout, Content, Platform>,
    data_flag: WatchedMeta,
    position_flag: WatchedMeta,
    layout: Layout,
    primary_pointer: Option<PointerId>,
}

impl<Layout, Content, Platform> AdapterView<Layout, Content, Platform>
where
    Layout: 'static + AdapterLayout,
    Content: WidgetContent<Platform> + Adaptable<Layout::ElementData>,
    Platform: RenderPlatform,
{
    /// Get the data collection stored by the layout.
    pub fn data(&self) -> &Layout::Collection {
        self.data_flag.watched();
        self.layout.data()
    }

    /// Get a mutable reference to the collection stored by the layout.
    pub fn data_mut(&mut self) -> &mut Layout::Collection {
        // if data is modified, flush the active children
        let old = std::mem::take(&mut self.inner.active);
        self.inner.inactive.extend(old.into_iter().map(|(_k, v)| v));
        self.data_flag.trigger();
        self.data_flag.watched();
        self.layout.data_mut()
    }
}

impl<Layout, Content, Platform> Default
    for AdapterView<Layout, Content, Platform>
where
    Layout: 'static + AdapterLayout + Default,
    Content: WidgetContent<Platform> + Adaptable<Layout::ElementData>,
    Platform: RenderPlatform,
{
    fn default() -> Self {
        let layout = Layout::default();
        AdapterView {
            inner: AdapterData {
                active: HashMap::default(),
                inactive: Vec::default(),
                position: (0.0, 0.0),
                rest_position: (0.0, 0.0),
            },
            data_flag: WatchedMeta::default(),
            position_flag: WatchedMeta::default(),
            layout,
            primary_pointer: None,
        }
    }
}

impl<Layout, Content, Platform> WidgetContent<Platform>
    for AdapterView<Layout, Content, Platform>
where
    Layout: 'static + AdapterLayout,
    Content: WidgetContent<Platform> + Adaptable<Layout::ElementData>,
    Platform: RenderPlatform,
{
    fn init(mut init: impl WidgetInit<Self, Platform>) {
        init.watch(|this, rect| {
            this.position_flag.watched();
            this.data_flag.watched();
            let prev = std::mem::take(&mut this.inner.active);
            let interface = Interface {
                rect: &rect,
                data: &mut this.inner,
                prev,
            };
            this.layout.layout(interface);
        });
    }

    fn children(&mut self, mut receiver: impl WidgetChildReceiver<Platform>) {
        for (_k, child) in self.inner.active.iter_mut() {
            receiver.child(child);
        }
    }

    fn graphics(&mut self, _receiver: impl WidgetGraphicReceiver<Platform>) {
        // no graphics
    }

    fn pointer_event(
        &mut self,
        extra: &mut crate::widget::WidgetExtra<'_>,
        event: &mut crate::pointer::PointerEvent,
    ) -> bool {
        use crate::pointer::PointerAction;
        match event.action() {
            PointerAction::Down => {
                let grabbed =
                    self.hittest(extra, event.pos()) && event.try_grab(extra);
                if grabbed {
                    self.primary_pointer.get_or_insert(event.id());
                }
                grabbed
            }
            PointerAction::Move(x, y) => {
                if Some(event.id()) == self.primary_pointer {
                    self.inner.position.0 += x;
                    self.inner.position.1 += y;
                    self.position_flag.trigger();
                    true
                } else {
                    false
                }
            }
            PointerAction::Wheel(x, y) => {
                if self.primary_pointer.is_none()
                    && self.hittest(extra, event.pos())
                {
                    self.inner.position.0 += x;
                    self.inner.position.1 += y;
                    self.position_flag.trigger();
                    true
                } else {
                    false
                }
            }
            PointerAction::GrabStolen => {
                if Some(event.id()) == self.primary_pointer {
                    self.primary_pointer = None;
                }
                true
            }
            PointerAction::Up => {
                let ungrabbed = event.try_ungrab(extra.id());
                if ungrabbed && Some(event.id()) == self.primary_pointer {
                    self.primary_pointer = None;
                }
                ungrabbed
            }
            _ => false,
        }
    }
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
