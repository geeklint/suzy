/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use crate::{
    pointer::PointerId,
    watch::WatchedMeta,
    widget::{self, UniqueHandle},
};

use super::{layout::AdapterLayoutData, Adaptable, AdapterLayout};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum PointerStatus {
    Grabbed,
    Pending,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct PointerEntry {
    status: PointerStatus,
    pointer: PointerId,
}

#[derive(Clone, Default)]
struct PointerSet {
    data: Vec<PointerEntry>,
}

impl PointerSet {
    fn primary_pointer(&self) -> Option<PointerId> {
        self.data.iter().find_map(|entry| {
            (entry.status == PointerStatus::Grabbed).then_some(entry.pointer)
        })
    }

    fn status(&self, pointer: PointerId) -> Option<PointerStatus> {
        self.data.iter().find_map(|entry| {
            (entry.pointer == pointer).then_some(entry.status)
        })
    }

    fn add_pending(&mut self, pointer: PointerId) {
        self.data.push(PointerEntry {
            status: PointerStatus::Pending,
            pointer,
        })
    }

    fn add_grabbed(&mut self, pointer: PointerId) {
        self.remove(pointer);
        self.data.push(PointerEntry {
            status: PointerStatus::Grabbed,
            pointer,
        })
    }

    fn remove(&mut self, pointer: PointerId) {
        self.data.retain(|entry| entry.pointer != pointer);
    }
}

/// Base adapter view.
pub struct AdapterView<Layout, Content>
where
    Layout: AdapterLayout,
{
    inner: AdapterLayoutData<Layout::ElementKey, Content>,
    data_flag: WatchedMeta<'static>,
    position_flag: WatchedMeta<'static>,
    layout: Layout,
    current_pointers: PointerSet,
    handle: UniqueHandle,
}

impl<Layout, Content> AdapterView<Layout, Content>
where
    Layout: AdapterLayout,
{
    /// Get the data collection stored by the layout.
    pub fn data(&self) -> &Layout::Collection {
        self.data_flag.watched_auto();
        self.layout.data()
    }

    /// Get a mutable reference to the collection stored by the layout.
    pub fn data_mut(&mut self) -> &mut Layout::Collection {
        // if data is modified, flush the active children
        self.inner.clear_active_children();
        self.data_flag.trigger_auto();
        self.data_flag.watched_auto();
        self.layout.data_mut()
    }

    /// This provides a Watched iterator of every Widget the [`AdapterView`]
    /// has instantiated.  This allows the parent widget of the [`AdapterView`]
    /// to listen to events from the content Widgets.
    pub fn watch_each_child(
        &self,
    ) -> impl Iterator<Item = &widget::Ephemeral<Content>> {
        self.inner.watch_each_child()
    }
}

impl<Layout, Content> Default for AdapterView<Layout, Content>
where
    Layout: AdapterLayout + Default,
{
    fn default() -> Self {
        let layout = Layout::default();
        AdapterView {
            inner: AdapterLayoutData::default(),
            data_flag: WatchedMeta::default(),
            position_flag: WatchedMeta::default(),
            layout,
            current_pointers: PointerSet::default(),
            handle: UniqueHandle::default(),
        }
    }
}

impl<Layout, Content, Platform> widget::Content<Platform>
    for AdapterView<Layout, Content>
where
    Self: 'static,
    Layout: AdapterLayout,
    Content: widget::Content<Platform> + Adaptable<Layout::ElementData>,
{
    fn desc(mut desc: impl widget::Desc<Self, Platform>) {
        desc.watch(|this, rect| {
            this.position_flag.watched_auto();
            this.data_flag.watched_auto();
            this.layout.layout(this.inner.get_interface(rect));
        });
        desc.watch(|this, _rect| {
            let current_pointers = &mut this.current_pointers;
            this.handle.handle_pointer_grab_stolen(|pointer_id| {
                current_pointers.remove(pointer_id);
            });
        });
        desc.iter_children(|this| Box::new(this.inner.active_children()));
    }

    fn pointer_event_before(
        &mut self,
        rect: &crate::widget::WidgetRect,
        event: &mut crate::pointer::PointerEvent<'_>,
    ) -> bool {
        use crate::pointer::PointerAction;
        match event.action() {
            PointerAction::Down => {
                if self.hittest(rect, event.pos()) {
                    self.current_pointers.add_pending(event.id());
                }
                false
            }
            PointerAction::Move(x, y) => {
                if self.current_pointers.status(event.id())
                    == Some(PointerStatus::Pending)
                {
                    self.current_pointers.add_grabbed(event.id());
                    if self.current_pointers.primary_pointer()
                        == Some(event.id())
                    {
                        self.inner.move_content(*x, *y);
                        self.position_flag.trigger_auto();
                    }
                    event.force_grab(self.handle.id());
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn pointer_event(
        &mut self,
        rect: &crate::widget::WidgetRect,
        event: &mut crate::pointer::PointerEvent<'_>,
    ) -> bool {
        use crate::pointer::PointerAction;
        match event.action() {
            PointerAction::Down => {
                let grabbed = self.hittest(rect, event.pos())
                    && event.try_grab(self.handle.id());
                if grabbed {
                    self.current_pointers.add_grabbed(event.id());
                }
                grabbed
            }
            PointerAction::Move(x, y) => {
                if Some(event.id()) == self.current_pointers.primary_pointer()
                {
                    self.inner.move_content(*x, *y);
                    self.position_flag.trigger_auto();
                    true
                } else {
                    false
                }
            }
            PointerAction::Wheel(x, y) => {
                if self.current_pointers.primary_pointer().is_none()
                    && self.hittest(rect, event.pos())
                {
                    self.inner.move_content(*x, *y);
                    self.position_flag.trigger_auto();
                    true
                } else {
                    false
                }
            }
            PointerAction::Up => {
                let ungrabbed = event.try_ungrab(self.handle.id());
                self.current_pointers.remove(event.id());
                ungrabbed
            }
            _ => false,
        }
    }
}
