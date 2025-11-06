mod boxed;
mod extended;
mod iterable;

use std::iter::FusedIterator;

pub use boxed::*;
pub use extended::*;
pub use iterable::*;

pub trait VecIterator: ExactSizeIterator + FusedIterator {
    fn set_position_to(&mut self, i: usize);

    #[inline]
    fn get_at(&mut self, i: usize) -> Option<Self::Item> {
        self.set_position_to(i);
        self.next()
    }

    #[inline]
    fn get_unwrap_at(&mut self, i: usize) -> Self::Item {
        self.get_at(i).unwrap()
    }

    fn set_end_to(&mut self, i: usize);
}
