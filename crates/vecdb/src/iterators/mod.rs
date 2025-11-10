use std::iter::FusedIterator;

mod boxed;
mod iterable;
mod typed;
mod writer;

pub use boxed::*;
pub use iterable::*;
pub use typed::*;
pub use writer::*;

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

    /// Gets the item at the given usize index, returns default if not found.
    #[inline]
    fn get_at_or_default(&mut self, i: usize) -> Self::Item
    where
        Self::Item: Default,
    {
        self.get_at(i).unwrap_or_default()
    }

    /// Sets the exclusive end boundary to the given usize index.
    fn set_end_to(&mut self, i: usize);
}
