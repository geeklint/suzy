pub enum PackerNode<T> {
    Empty { x: f64, y: f64, width: f64, height: f64 },
    Occupied { item: T, x: f64, y: f64 },
    Split { smaller: Box<Self>, bigger: Box<Self> },
}

impl<T> PackerNode<T> {
    pub fn new(width: f64, height: f64) -> Self {
        Self::Empty { x: 0.0, y: 0.0, width, height }
    }

    pub fn for_each<F>(&self, mut func: F) -> F
        where F: FnMut(f64, f64, &T)
    {
        use PackerNode::*;

        match self {
            Empty { .. } => func,
            Occupied { item, x, y } => {
                (func)(*x, *y, item);
                func
            },
            Split { smaller, bigger } => {
                smaller.for_each(bigger.for_each(func))
            },
        }
    }

    pub fn empty_area(&self) -> f64 {
        use PackerNode::*;

        match self {
            Empty { width, height, .. } => width * height,
            Occupied { .. } => 0.0,
            Split { smaller, bigger } => {
                bigger.empty_area() + smaller.empty_area()
            },
        }
    }

    pub fn add(&mut self, item: T, width: f64, height: f64) -> Result<(), T> {
        use PackerNode::*;

        match self {
            Occupied { .. } => Err(item),
            Split { smaller, bigger } => {
                smaller.add(item, width, height)
                    .or_else(|item| bigger.add(item, width, height))
            },
            Empty { x, y, width: self_width, height: self_height } => {
                let x = *x;
                let y = *y;
                let self_width = *self_width;
                let self_height = *self_height;
                if self_width < width || self_height < height {
                    Err(item)
                } else if self_width == width && self_height == height {
                    *self = Occupied { item, x, y };
                    Ok(())
                } else if self_width == width {
                    let other_height = self_height - height;
                    let entry = Box::new(Occupied { item, x, y });
                    let other = Box::new(Empty {
                        x,
                        y: y + height,
                        width,
                        height: other_height,
                    });
                    *self = if height < other_height {
                        Split { smaller: entry, bigger: other }
                    } else {
                        Split { smaller: other, bigger: entry }
                    };
                    Ok(())
                } else {
                    let other_width = self_width - width;
                    let mut entry = Box::new(Empty {
                        x,
                        y,
                        width,
                        height: self_height,
                    });
                    let other = Box::new(Empty {
                        x: x + width,
                        y,
                        width: other_width,
                        height: self_height,
                    });
                    entry.add(item, width, height)
                        .map_err(|_| ())
                        .unwrap();
                    *self = if width < other_width {
                        Split { smaller: entry, bigger: other }
                    } else {
                        Split { smaller: other, bigger: entry }
                    };
                    Ok(())
                }
            }
        }
    }
}
