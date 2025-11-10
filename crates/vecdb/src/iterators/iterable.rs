use crate::{AnyStoredVec, AnyVec, BoxedVecIterator, StoredIndex, StoredRaw};

/// Trait for vectors that can be iterated.
pub trait IterableVec<I, T>: AnyVec {
    #[allow(clippy::wrong_self_convention)]
    fn iter(&self) -> BoxedVecIterator<'_, I, T>
    where
        I: StoredIndex,
        T: StoredRaw;
}

/// Trait combining stored and iterable vector capabilities.
pub trait IterableStoredVec<I, T>: IterableVec<I, T> + AnyStoredVec {}

impl<I, T, U> IterableStoredVec<I, T> for U where U: 'static + IterableVec<I, T> + AnyStoredVec {}

/// Trait for iterable vectors that can be cloned as trait objects.
pub trait IterableCloneableVec<I, T>: IterableVec<I, T> {
    fn boxed_clone(&self) -> Box<dyn IterableCloneableVec<I, T>>;
}

impl<I, T, U> IterableCloneableVec<I, T> for U
where
    U: 'static + IterableVec<I, T> + Clone,
{
    fn boxed_clone(&self) -> Box<dyn IterableCloneableVec<I, T>> {
        Box::new(self.clone())
    }
}

impl<I, T> Clone for Box<dyn IterableCloneableVec<I, T>> {
    fn clone(&self) -> Self {
        self.boxed_clone()
    }
}

/// Type alias for boxed cloneable iterable vectors.
pub type IterableBoxedVec<I, T> = Box<dyn IterableCloneableVec<I, T>>;
