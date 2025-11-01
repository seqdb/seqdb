use std::iter::FusedIterator;

use crate::{
    CompressedVec, Result, StoredCompressed, StoredIndex, VecIterator, VecIteratorExtended,
};

mod clean;
mod dirty;

pub use clean::*;
pub use dirty::*;

pub enum CompressedVecIterator<'a, I, T> {
    Clean(CleanCompressedVecIterator<'a, I, T>),
    Dirty(DirtyCompressedVecIterator<'a, I, T>),
}

impl<'a, I, T> CompressedVecIterator<'a, I, T>
where
    I: StoredIndex,
    T: StoredCompressed,
{
    #[inline]
    pub fn new(vec: &'a CompressedVec<I, T>) -> Result<Self> {
        Ok(if vec.is_dirty() {
            Self::Dirty(DirtyCompressedVecIterator::new(vec)?)
        } else {
            Self::Clean(CleanCompressedVecIterator::new(vec)?)
        })
    }
}

impl<I, T> Iterator for CompressedVecIterator<'_, I, T>
where
    I: StoredIndex,
    T: StoredCompressed,
{
    type Item = T;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Clean(iter) => iter.next(),
            Self::Dirty(iter) => iter.next(),
        }
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<T> {
        match self {
            Self::Clean(iter) => iter.nth(n),
            Self::Dirty(iter) => iter.nth(n),
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Clean(iter) => iter.size_hint(),
            Self::Dirty(iter) => iter.size_hint(),
        }
    }

    #[inline]
    fn count(self) -> usize {
        match self {
            Self::Clean(iter) => iter.count(),
            Self::Dirty(iter) => iter.count(),
        }
    }

    #[inline]
    fn last(self) -> Option<T> {
        match self {
            Self::Clean(iter) => iter.last(),
            Self::Dirty(iter) => iter.last(),
        }
    }
}

impl<I, T> VecIterator for CompressedVecIterator<'_, I, T>
where
    I: StoredIndex,
    T: StoredCompressed,
{
    fn set_position_(&mut self, i: usize) {
        match self {
            Self::Clean(iter) => iter.set_position_(i),
            Self::Dirty(iter) => iter.set_position_(i),
        };
    }

    fn set_end_(&mut self, i: usize) {
        match self {
            Self::Clean(iter) => iter.set_end_(i),
            Self::Dirty(iter) => iter.set_end_(i),
        };
    }
}

impl<I, T> VecIteratorExtended for CompressedVecIterator<'_, I, T>
where
    I: StoredIndex,
    T: StoredCompressed,
{
    type I = I;
    type T = T;
}

impl<I, T> ExactSizeIterator for CompressedVecIterator<'_, I, T>
where
    I: StoredIndex,
    T: StoredCompressed,
{
    #[inline(always)]
    fn len(&self) -> usize {
        match self {
            Self::Clean(iter) => iter.len(),
            Self::Dirty(iter) => iter.len(),
        }
    }
}

impl<I, T> FusedIterator for CompressedVecIterator<'_, I, T>
where
    I: StoredIndex,
    T: StoredCompressed,
{
}
