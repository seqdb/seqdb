use std::iter::FusedIterator;

use crate::{
    Result, StoredCompressed, StoredIndex, StoredVec, VecIterator, VecIteratorExtended,
    variants::{CompressedVecIterator, RawVecIterator},
};

pub enum StoredVecIterator<'a, I, T> {
    Raw(RawVecIterator<'a, I, T>),
    Compressed(CompressedVecIterator<'a, I, T>),
}

impl<'a, I, T> StoredVecIterator<'a, I, T>
where
    I: StoredIndex,
    T: StoredCompressed,
{
    #[inline]
    pub fn new(vec: &'a StoredVec<I, T>) -> Result<Self> {
        Ok(match vec {
            StoredVec::Raw(v) => Self::Raw(v.iter()?),
            StoredVec::Compressed(v) => Self::Compressed(v.iter()?),
        })
    }
}

impl<I, T> Iterator for StoredVecIterator<'_, I, T>
where
    I: StoredIndex,
    T: StoredCompressed,
{
    type Item = T;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Raw(iter) => iter.next(),
            Self::Compressed(iter) => iter.next(),
        }
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<T> {
        match self {
            Self::Raw(iter) => iter.nth(n),
            Self::Compressed(iter) => iter.nth(n),
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Raw(iter) => iter.size_hint(),
            Self::Compressed(iter) => iter.size_hint(),
        }
    }

    #[inline]
    fn count(self) -> usize {
        match self {
            Self::Raw(iter) => iter.count(),
            Self::Compressed(iter) => iter.count(),
        }
    }

    #[inline]
    fn last(self) -> Option<T> {
        match self {
            Self::Raw(iter) => iter.last(),
            Self::Compressed(iter) => iter.last(),
        }
    }
}

impl<I, T> ExactSizeIterator for StoredVecIterator<'_, I, T>
where
    I: StoredIndex,
    T: StoredCompressed,
{
    #[inline(always)]
    fn len(&self) -> usize {
        match self {
            Self::Raw(iter) => iter.len(),
            Self::Compressed(iter) => iter.len(),
        }
    }
}

impl<I, T> FusedIterator for StoredVecIterator<'_, I, T>
where
    I: StoredIndex,
    T: StoredCompressed,
{
}

impl<I, T> VecIterator for StoredVecIterator<'_, I, T>
where
    I: StoredIndex,
    T: StoredCompressed,
{
    fn skip_optimized(self, n: usize) -> Self {
        match self {
            Self::Raw(iter) => Self::Raw(iter.skip_optimized(n)),
            Self::Compressed(iter) => Self::Compressed(iter.skip_optimized(n)),
        }
    }

    fn take_optimized(self, n: usize) -> Self {
        match self {
            Self::Raw(iter) => Self::Raw(iter.take_optimized(n)),
            Self::Compressed(iter) => Self::Compressed(iter.take_optimized(n)),
        }
    }
}

impl<I, T> VecIteratorExtended for StoredVecIterator<'_, I, T>
where
    I: StoredIndex,
    T: StoredCompressed,
{
    type I = I;
    type T = T;
}
