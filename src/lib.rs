extern crate drying_paint;

pub mod dims;
pub mod graphics;
pub mod interaction;
pub mod platform;
pub mod widget;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
