use crate::{AnyStoredVec, AnyVec, BoxedVecIterator, StoredIndex, StoredRaw};

/// Trait for vectors that can be iterated.
pub trait AnyIterableVec<I, T>: AnyVec {
    #[allow(clippy::wrong_self_convention)]
    fn boxed_iter(&self) -> BoxedVecIterator<'_, I, T>
    where
        I: StoredIndex,
        T: StoredRaw;

    fn iter(&self) -> BoxedVecIterator<'_, I, T>
    where
        I: StoredIndex,
        T: StoredRaw,
    {
        self.boxed_iter()
    }
}

/// Trait combining stored and iterable vector capabilities.
pub trait AnyStoredIterableVec<I, T>: AnyIterableVec<I, T> + AnyStoredVec {}

impl<I, T, U> AnyStoredIterableVec<I, T> for U where U: 'static + AnyIterableVec<I, T> + AnyStoredVec
{}

/// Trait for iterable vectors that can be cloned as trait objects.
pub trait AnyCloneableIterableVec<I, T>: AnyIterableVec<I, T> {
    fn boxed_clone(&self) -> Box<dyn AnyCloneableIterableVec<I, T>>;
}

impl<I, T, U> AnyCloneableIterableVec<I, T> for U
where
    U: 'static + AnyIterableVec<I, T> + Clone,
{
    fn boxed_clone(&self) -> Box<dyn AnyCloneableIterableVec<I, T>> {
        Box::new(self.clone())
    }
}

impl<I, T> Clone for Box<dyn AnyCloneableIterableVec<I, T>> {
    fn clone(&self) -> Self {
        self.boxed_clone()
    }
}

/// Type alias for boxed cloneable iterable vectors.
pub type AnyBoxedIterableVec<I, T> = Box<dyn AnyCloneableIterableVec<I, T>>;
