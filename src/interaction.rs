
#[derive(Copy, Clone, Debug)]
pub struct Touch {}

pub trait InteractionReceiver {
    #[allow(unused_variables)]
    fn on_touch_down(&mut self, touch: Touch) -> bool { false }
    #[allow(unused_variables)]
    fn on_touch_move(&mut self, touch: Touch) -> bool { false }
    #[allow(unused_variables)]
    fn on_touch_up(&mut self, touch: Touch) -> bool { false }
}
