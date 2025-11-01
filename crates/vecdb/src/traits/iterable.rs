use crate::{AnyStoredVec, AnyVec, BoxedVecIterator, StoredIndex, StoredRaw};

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

pub trait AnyStoredIterableVec<I, T>: AnyIterableVec<I, T> + AnyStoredVec {}

impl<I, T, U> AnyStoredIterableVec<I, T> for U where U: 'static + AnyIterableVec<I, T> + AnyStoredVec
{}

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

pub type AnyBoxedIterableVec<I, T> = Box<dyn AnyCloneableIterableVec<I, T>>;
