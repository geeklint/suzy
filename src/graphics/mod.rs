
pub mod image;

pub trait Graphic {
    fn draw(&self);
}

impl Graphic for () {
    fn draw(&self) {}
}

impl<T: Graphic> Graphic for [T] {
    fn draw(&self) {
        for graphic in self {
            graphic.draw();
        }
    }
}

impl Graphic for [Box<dyn Graphic>] {
    fn draw(&self) {
        for graphic in self {
            graphic.draw();
        }
    }
}
