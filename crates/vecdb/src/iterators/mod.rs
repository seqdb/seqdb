mod boxed;
mod extended;
mod iterable;

use std::iter::FusedIterator;

pub use boxed::*;
pub use extended::*;
pub use iterable::*;

/// Base trait for vector iterators with positioning capabilities.
pub trait VecIterator: ExactSizeIterator + FusedIterator {
    /// Sets the current position to the given usize index.
    fn set_position_to(&mut self, i: usize);

    /// Gets the item at the given usize index.
    #[inline]
    fn get_at(&mut self, i: usize) -> Option<Self::Item> {
        self.set_position_to(i);
        self.next()
    }

    /// Gets the item at the given usize index, panics if not found.
    #[inline]
    fn get_at_unwrap(&mut self, i: usize) -> Self::Item {
        self.get_at(i).unwrap()
    }

    /// Sets the exclusive end boundary to the given usize index.
    fn set_end_to(&mut self, i: usize);
}
