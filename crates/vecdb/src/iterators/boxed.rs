use crate::TypedVecIterator;

/// Type alias for boxed vector iterators.
pub type BoxedVecIterator<'a, I, T> = Box<dyn TypedVecIterator<I = I, T = T, Item = T> + 'a>;
