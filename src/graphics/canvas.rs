
use super::Graphic;

pub enum Canvas<A,B,C,D,U>
where
    A: Graphic,
    B: Graphic,
    C: Graphic,
    D: Graphic,
    U: Graphic,
{
    Zero,
    One(A),
    Two(A,B),
    Three(A,B,C),
    Four(A,B,C,D),
    Uniform(
    graphics: Vec<Box<dyn Graphic>,
}


impl Canvas {
    pub(crate) fn draw(&self) {
        for graphic in self.graphics.iter() {
            graphic.draw();
        }
    }
}
