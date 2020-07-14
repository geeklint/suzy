use std::cell::Ref;

use drying_paint::{
    Watched,
    WatchedEvent,
};

use crate::pointer::{
    PointerEvent,
    PointerAction,
};
use crate::platform::{
    RenderPlatform,
    DefaultRenderPlatform,
};
use crate::widget::{
    Widget,
    WidgetContent,
    WidgetInit,
    WidgetChildReceiver,
    WidgetMutChildReceiver,
    WidgetGraphicReceiver,
    WidgetExtra,
};
use crate::selectable::{
    Selectable,
    SelectionState,
    SelectionStateV1,
};

struct ButtonContent<T> {
    on_click: WatchedEvent<()>,
    state: Watched<SelectionState>,
    interactable: Watched<bool>,
    pointers_down: usize,
    content: T,
}

impl<T, P> WidgetContent<P> for ButtonContent<T>
where
    P: RenderPlatform,
    T: Selectable + WidgetContent<P>,
{
    fn init<I: WidgetInit<Self, P>>(mut init: I) {
        init.init_child_inline(|button| &mut button.content);
        init.watch(|button, _rect| {
            button.content.selection_changed(*button.state);
        });
        init.watch(|button, _rect| {
            if !*button.interactable {
                *button.state = SelectionState::normal();
            }
        });
    }

    fn children<R: WidgetChildReceiver<P>>(&self, receiver: R) {
        self.content.children(receiver);
    }

    fn children_mut<R: WidgetMutChildReceiver<P>>(&mut self, receiver: R) {
        self.content.children_mut(receiver);
    }

    fn graphics<R: WidgetGraphicReceiver<P>>(&mut self, receiver: R) {
        self.content.graphics(receiver);
    }

    fn hittest(&self, extra: &mut WidgetExtra<'_>, point: (f32, f32)) -> bool {
        self.content.hittest(extra, point)
    }

    fn pointer_event(
        &mut self,
        extra: &mut WidgetExtra<'_>,
        event: &mut PointerEvent,
    ) -> bool {
        match event.action() {
            PointerAction::Down => {
                let grabbed = self.hittest(extra, event.pos())
                    && event.try_grab(extra);
                if grabbed {
                    self.pointers_down += 1;
                    if *self.interactable {
                        *self.state = SelectionState::active();
                    }
                }
                grabbed
            },
            PointerAction::Move(_, _) => {
                let ungrabbed = !self.hittest(extra, event.pos())
                    && event.try_ungrab(extra);
                if ungrabbed {
                    self.pointers_down -= 1;
                    if self.pointers_down == 0 {
                        *self.state = SelectionState::normal();
                    }
                }
                ungrabbed
            },
            PointerAction::GrabStolen => {
                self.pointers_down -= 1;
                if self.pointers_down == 0 {
                    *self.state = SelectionState::normal();
                }
                true
            },
            PointerAction::Up => {
                let ungrabbed = event.try_ungrab(extra);
                if ungrabbed {
                    self.pointers_down -= 1;
                    if self.pointers_down == 0 {
                        *self.state = SelectionState::normal();
                        self.on_click.dispatch(());
                    }
                }
                ungrabbed
            },
            PointerAction::Hover => {
                match (self.state.v1(), self.hittest(extra, event.pos())) {
                    (SelectionStateV1::Normal, true) => {
                        let grabbed = event.try_grab(extra);
                        if grabbed && *self.interactable {
                            *self.state = SelectionState::hover();
                        }
                        grabbed
                    }
                    (SelectionStateV1::Hover, false) => {
                        let ungrabbed = event.try_ungrab(extra);
                        if ungrabbed {
                            *self.state = SelectionState::normal();
                        }
                        ungrabbed
                    }
                    _ => false,
                }
            },
            _ => false,
        }
    }
}


pub struct Button<T, P = DefaultRenderPlatform>
where
    P: RenderPlatform,
    T: Selectable + WidgetContent<P>,
{
    widget: Widget<ButtonContent<T>, P>,
}

impl<T, P> Button<T, P>
where
    P: RenderPlatform,
    T: Selectable + WidgetContent<P>,
{
    pub fn content(&self) -> Ref<T> {
        Ref::map(
            self.widget.content(),
            |button| &button.content,
        )
    }

    pub fn content_mut(&self) -> Ref<T> {
        Ref::map(
            self.widget.content(),
            |button| &button.content,
        )
    }

    pub fn on_click<F: FnOnce()>(&mut self, handler: F) {
        let borrow = self.widget.content_mut();
        borrow.on_click.bind(move |()| handler());
    }

    pub fn state(&self) -> SelectionState {
        *self.widget.content().state
    }
}
