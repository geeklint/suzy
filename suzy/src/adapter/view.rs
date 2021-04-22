use crate::platform::RenderPlatform;
use crate::pointer::PointerId;
use crate::watch::WatchedMeta;
use crate::widget::{
    Widget, WidgetChildReceiver, WidgetContent, WidgetGraphicReceiver,
    WidgetInit,
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
        self.data
            .iter()
            .filter_map(|entry| {
                (entry.status == PointerStatus::Grabbed).then(|| entry.pointer)
            })
            .next()
    }

    fn contains(&self, pointer: PointerId) -> bool {
        self.data.iter().any(|entry| entry.pointer == pointer)
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
pub struct AdapterView<Layout, Content, Platform>
where
    Layout: AdapterLayout,
    Content: WidgetContent<Platform> + Adaptable<Layout::ElementData>,
    Platform: RenderPlatform,
{
    inner: AdapterLayoutData<Layout, Content, Platform>,
    data_flag: WatchedMeta,
    position_flag: WatchedMeta,
    layout: Layout,
    current_pointers: PointerSet,
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
        self.inner.clear_active_children();
        self.data_flag.trigger();
        self.data_flag.watched();
        self.layout.data_mut()
    }

    /// This provides a Watched iterator of every Widget the AdapterView
    /// has instantiated.  This allows the parent widget of the AdapterView
    /// to listen to events from the content Widgets.
    pub fn watch_each_child(
        &self,
    ) -> impl Iterator<Item = &Widget<Content, Platform>> {
        self.inner.watch_each_child()
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
            inner: AdapterLayoutData::default(),
            data_flag: WatchedMeta::default(),
            position_flag: WatchedMeta::default(),
            layout,
            current_pointers: PointerSet::default(),
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
            this.layout.layout(this.inner.get_interface(&rect));
        });
    }

    fn children(&mut self, mut receiver: impl WidgetChildReceiver<Platform>) {
        for child in self.inner.active_children() {
            receiver.child(child);
        }
    }

    fn graphics(&mut self, _receiver: impl WidgetGraphicReceiver<Platform>) {
        // no graphics
    }

    fn pointer_event_before(
        &mut self,
        extra: &mut crate::widget::WidgetExtra<'_>,
        event: &mut crate::pointer::PointerEvent,
    ) -> bool {
        use crate::pointer::PointerAction;
        match event.action() {
            PointerAction::Down => {
                if self.hittest(extra, event.pos()) {
                    self.current_pointers.add_pending(event.id());
                }
                false
            }
            PointerAction::Move(x, y) => {
                if self.current_pointers.contains(event.id()) {
                    self.current_pointers.add_grabbed(event.id());
                    if self.current_pointers.primary_pointer()
                        == Some(event.id())
                    {
                        self.inner.move_content(*x, *y);
                        self.position_flag.trigger();
                    }
                    event.force_grab(extra);
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
        extra: &mut crate::widget::WidgetExtra<'_>,
        event: &mut crate::pointer::PointerEvent,
    ) -> bool {
        use crate::pointer::PointerAction;
        match event.action() {
            PointerAction::Down => {
                let grabbed =
                    self.hittest(extra, event.pos()) && event.try_grab(extra);
                if grabbed {
                    self.current_pointers.add_grabbed(event.id());
                }
                grabbed
            }
            PointerAction::Move(x, y) => {
                if Some(event.id()) == self.current_pointers.primary_pointer()
                {
                    self.inner.move_content(*x, *y);
                    self.position_flag.trigger();
                    true
                } else {
                    false
                }
            }
            PointerAction::Wheel(x, y) => {
                if self.current_pointers.primary_pointer().is_none()
                    && self.hittest(extra, event.pos())
                {
                    self.inner.move_content(*x, *y);
                    self.position_flag.trigger();
                    true
                } else {
                    false
                }
            }
            PointerAction::GrabStolen => {
                self.current_pointers.remove(event.id());
                true
            }
            PointerAction::Up => {
                let ungrabbed = event.try_ungrab(extra.id());
                self.current_pointers.remove(event.id());
                ungrabbed
            }
            _ => false,
        }
    }
}