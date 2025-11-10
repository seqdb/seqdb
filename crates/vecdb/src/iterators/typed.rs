use crate::{PrintableIndex, VecIndex, VecIterator, VecValue};

/// Extended vector iterator with type-safe index operations.
pub trait TypedVecIterator: VecIterator<Item = Self::T> {
    type I: VecIndex;
    type T: VecValue;

    /// Sets the current position using the typed index.
    #[inline]
    fn set_position(&mut self, i: Self::I) {
        self.set_position_to(i.to_usize());
    }

    /// Sets the exclusive end boundary using the typed index.
    #[inline]
    fn set_end(&mut self, i: Self::I) {
        self.set_end_to(i.to_usize());
    }

    /// Gets the item at the given typed index.
    #[inline]
    fn get(&mut self, i: Self::I) -> Option<Self::Item> {
        self.get_at(i.to_usize())
    }

    /// Gets the item at the given typed index, panics if not found.
    #[inline]
    fn get_unwrap(&mut self, i: Self::I) -> Self::Item {
        self.get(i).unwrap()
    }

    /// Gets the item at the given typed index, returns default if not found.
    #[inline]
    fn get_or_default(&mut self, i: Self::I) -> Self::Item
    where
        Self::Item: Default,
    {
        self.get(i).unwrap_or_default()
    }

    fn index_type_to_string(&self) -> &'static str {
        Self::I::to_string()
    }
}
