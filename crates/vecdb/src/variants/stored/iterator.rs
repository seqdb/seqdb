use std::iter::FusedIterator;

use crate::{
    Compressable, Result, StoredVec, TypedVecIterator, VecIndex, VecIterator,
    variants::{CompressedVecIterator, RawVecIterator},
};

pub enum StoredVecIterator<'a, I, T> {
    Raw(RawVecIterator<'a, I, T>),
    Compressed(CompressedVecIterator<'a, I, T>),
}

impl<'a, I, T> StoredVecIterator<'a, I, T>
where
    I: VecIndex,
    T: Compressable,
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
    I: VecIndex,
    T: Compressable,
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

impl<I, T> VecIterator for StoredVecIterator<'_, I, T>
where
    I: VecIndex,
    T: Compressable,
{
    fn set_position_to(&mut self, i: usize) {
        match self {
            Self::Raw(iter) => iter.set_position_to(i),
            Self::Compressed(iter) => iter.set_position_to(i),
        };
    }

    fn set_end_to(&mut self, i: usize) {
        match self {
            Self::Raw(iter) => iter.set_end_to(i),
            Self::Compressed(iter) => iter.set_end_to(i),
        };
    }
}

impl<I, T> TypedVecIterator for StoredVecIterator<'_, I, T>
where
    I: VecIndex,
    T: Compressable,
{
    type I = I;
    type T = T;
}

impl<I, T> ExactSizeIterator for StoredVecIterator<'_, I, T>
where
    I: VecIndex,
    T: Compressable,
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
    I: VecIndex,
    T: Compressable,
{
}
