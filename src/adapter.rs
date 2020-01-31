
pub trait Adaptable<T: ?Sized>: for<'a> From<&'a T> {
    fn adapt(&mut self, data: &T);
}
