use std::iter::FusedIterator;

use crate::{RawVec, Result, StoredIndex, StoredRaw, VecIterator, VecIteratorExtended};

mod clean;
mod dirty;

pub use clean::*;
pub use dirty::*;

pub enum RawVecIterator<'a, I, T> {
    Clean(CleanRawVecIterator<'a, I, T>),
    Dirty(DirtyRawVecIterator<'a, I, T>),
}

impl<'a, I, T> RawVecIterator<'a, I, T>
where
    I: StoredIndex,
    T: StoredRaw,
{
    #[inline]
    pub fn new(vec: &'a RawVec<I, T>) -> Result<Self> {
        Ok(if vec.is_dirty() {
            Self::Dirty(DirtyRawVecIterator::new(vec)?)
        } else {
            Self::Clean(CleanRawVecIterator::new(vec)?)
        })
    }

    pub fn is_clean(&self) -> bool {
        matches!(self, Self::Clean(_))
    }

    pub fn is_dirty(&self) -> bool {
        matches!(self, Self::Dirty(_))
    }
}

impl<I, T> Iterator for RawVecIterator<'_, I, T>
where
    I: StoredIndex,
    T: StoredRaw,
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

impl<I, T> ExactSizeIterator for RawVecIterator<'_, I, T>
where
    I: StoredIndex,
    T: StoredRaw,
{
    #[inline(always)]
    fn len(&self) -> usize {
        match self {
            Self::Clean(iter) => iter.len(),
            Self::Dirty(iter) => iter.len(),
        }
    }
}

impl<I, T> FusedIterator for RawVecIterator<'_, I, T>
where
    I: StoredIndex,
    T: StoredRaw,
{
}

impl<I, T> VecIterator for RawVecIterator<'_, I, T>
where
    I: StoredIndex,
    T: StoredRaw,
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

    fn skip_optimized(self, n: usize) -> Self {
        match self {
            Self::Clean(iter) => Self::Clean(iter.skip_optimized(n)),
            Self::Dirty(iter) => Self::Dirty(iter.skip_optimized(n)),
        }
    }

    fn take_optimized(self, n: usize) -> Self {
        match self {
            Self::Clean(iter) => Self::Clean(iter.take_optimized(n)),
            Self::Dirty(iter) => Self::Dirty(iter.take_optimized(n)),
        }
    }
}

impl<I, T> VecIteratorExtended for RawVecIterator<'_, I, T>
where
    I: StoredIndex,
    T: StoredRaw,
{
    type I = I;
    type T = T;
}
