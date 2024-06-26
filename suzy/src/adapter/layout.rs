/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use std::{collections::HashMap, hash::Hash};

use crate::{
    dims::Rect,
    watch::WatchedMeta,
    widget::{self, WidgetRect},
};

use super::Adaptable;

/// An implementation of this trait is passed to [`AdapterLayout`] implementations.
///
/// It provides the utilities an adapter layout uses to control the view.
pub trait AdapterLayoutInterface<Key, Data> {
    /// A rectangle which represents the extants of the view. Returned by
    /// the `bounds` method.
    type Bounds: Rect;

    /// A rectangle which represents an individual element to be positioned
    /// by the layout.
    type Element: Rect;

    /// Get the position last set with `update_reference`, offset by any
    /// scrolling that has happened since.
    fn reference_position(&self) -> [f32; 2];

    /// Get the rectangle which represents the extants of the view.
    fn bounds(&self) -> &Self::Bounds;

    /// Update the reference and rest positions.
    ///
    /// The rest position can be used in the case of overscroll to return the
    /// list to a preferred position.
    fn update_positions(
        &mut self,
        reference_position: [f32; 2],
        rest_position: [f32; 2],
    );

    /// Get the number of times [`Self::get_element`] has been called during this
    /// layout operation.
    fn num_active_elements(&self) -> usize;

    /// Get or construct a widget from some data.
    fn get_element(&mut self, key: Key, data: &Data) -> &mut Self::Element;
}

/// An adapter layout defines how elements in the adapter view are organized.
pub trait AdapterLayout {
    /// The key for looking up existing widgets.
    type ElementKey: Hash + Eq;

    /// The type of the collection the layout understands.
    type Collection: ?Sized;

    /// The type of the data widgets will need to adapt to.
    type ElementData;

    /// Get a reference to the collection.
    fn data(&self) -> &Self::Collection;

    /// Get a mutable reference to the collection.
    fn data_mut(&mut self) -> &mut Self::Collection;

    /// Execute the layout, using the provided interface to position elements.
    fn layout(
        &mut self,
        interface: impl AdapterLayoutInterface<Self::ElementKey, Self::ElementData>,
    );

    /// Get an approximate location of an element that is not currently in
    /// view.
    fn element_location(
        &mut self,
        item: &Self::ElementKey,
        reference_position: [f32; 2],
    ) -> Option<[f32; 2]>;
}

pub(super) struct AdapterLayoutData<ElementKey, Content> {
    active: HashMap<ElementKey, widget::Ephemeral<Content>>,
    inactive: Vec<widget::Ephemeral<Content>>,
    child_flag: WatchedMeta<'static>,
    position: [f32; 2],
    rest_position: [f32; 2],
}

impl<ElementKey, Content> Default for AdapterLayoutData<ElementKey, Content> {
    fn default() -> Self {
        AdapterLayoutData {
            active: HashMap::default(),
            inactive: Vec::default(),
            child_flag: WatchedMeta::default(),
            position: [0.0, 0.0],
            rest_position: [0.0, 0.0],
        }
    }
}

impl<ElementKey, Content> AdapterLayoutData<ElementKey, Content> {
    pub fn clear_active_children(&mut self) {
        let old = std::mem::take(&mut self.active);
        self.inactive.extend(old.into_values());
    }

    pub fn watch_each_child(
        &self,
    ) -> impl Iterator<Item = &widget::Ephemeral<Content>> {
        self.child_flag.watched_auto();
        self.active.values().chain(&self.inactive)
    }

    pub fn get_interface<'a, Data>(
        &'a mut self,
        rect: &'a WidgetRect,
    ) -> impl AdapterLayoutInterface<ElementKey, Data> + 'a
    where
        ElementKey: Hash + Eq,
        Content: Adaptable<Data>,
    {
        let prev = std::mem::take(&mut self.active);
        Interface {
            rect,
            data: self,
            prev,
        }
    }

    pub fn active_children(
        &mut self,
    ) -> impl Iterator<Item = &mut widget::Ephemeral<Content>> {
        self.child_flag.watched_auto();
        self.active.iter_mut().map(|(_k, child)| child)
    }

    pub fn move_content(&mut self, dx: f32, dy: f32) {
        let [px, py] = &mut self.position;
        *px += dx;
        *py += dy;
    }
}

struct Interface<'a, Key, Content> {
    rect: &'a WidgetRect,
    data: &'a mut AdapterLayoutData<Key, Content>,
    prev: HashMap<Key, widget::Ephemeral<Content>>,
}

impl<'a, Key, Data, Content> AdapterLayoutInterface<Key, Data>
    for Interface<'a, Key, Content>
where
    Key: Hash + Eq,
    Content: Adaptable<Data>,
{
    type Bounds = WidgetRect;
    type Element = widget::Ephemeral<Content>;

    fn reference_position(&self) -> [f32; 2] {
        self.data.position
    }

    fn bounds(&self) -> &WidgetRect {
        self.rect
    }

    fn update_positions(
        &mut self,
        position: [f32; 2],
        rest_position: [f32; 2],
    ) {
        self.data.position = position;
        self.data.rest_position = rest_position;
    }

    fn num_active_elements(&self) -> usize {
        self.data.active.len()
    }

    fn get_element(&mut self, key: Key, data: &Data) -> &mut Self::Element {
        use std::collections::hash_map::Entry;
        match self.data.active.entry(key) {
            Entry::Occupied(bucket) => bucket.into_mut(),
            Entry::Vacant(bucket) => {
                let inactive = &mut self.data.inactive;
                let child_flag = &mut self.data.child_flag;
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
                    .unwrap_or_else(|| {
                        child_flag.trigger_auto();
                        Adaptable::from(data)
                    });
                bucket.insert(element)
            }
        }
    }
}

impl<'a, Key, Content> Drop for Interface<'a, Key, Content> {
    fn drop(&mut self) {
        let remaining = std::mem::take(&mut self.prev);
        self.data.inactive.extend(remaining.into_values());
    }
}
