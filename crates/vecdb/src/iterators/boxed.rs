use crate::VecIteratorExtended;

pub type BoxedVecIterator<'a, I, T> = Box<dyn VecIteratorExtended<I = I, T = T, Item = T> + 'a>;
