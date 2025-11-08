use crate::VecIteratorExtended;

/// Type alias for boxed vector iterators.
pub type BoxedVecIterator<'a, I, T> = Box<dyn VecIteratorExtended<I = I, T = T, Item = T> + 'a>;
