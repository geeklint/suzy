
#[derive(Debug, Default)]
pub struct Canvas {
}

pub struct CanvasRenderer {
}

impl CanvasRenderer {
    pub(crate) fn draw(&mut self, canvas: &Canvas) {
        println!("draw {:?}", canvas);
    }
}
