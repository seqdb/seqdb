mod boxed;
mod extended;
mod iterable;

pub use boxed::*;
pub use extended::*;
pub use iterable::*;

pub trait VecIterator: Iterator {
    fn set_position_(&mut self, i: usize);

    #[inline]
    fn get_(&mut self, i: usize) -> Option<Self::Item> {
        self.set_position_(i);
        self.next()
    }

    #[inline]
    fn unsafe_get_(&mut self, i: usize) -> Self::Item {
        unsafe { self.get_(i).unwrap_unchecked() }
    }

    fn set_end_(&mut self, i: usize);
}
