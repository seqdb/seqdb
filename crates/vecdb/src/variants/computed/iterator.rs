use std::iter::FusedIterator;

use crate::{
    ComputedVec, StoredCompressed, StoredIndex, StoredRaw, VecIterator, VecIteratorExtended,
    variants::{
        LazyVecFrom1Iterator, LazyVecFrom2Iterator, LazyVecFrom3Iterator, StoredVecIterator,
    },
};

pub enum ComputedVecIterator<'a, I, T, S1I, S1T, S2I, S2T, S3I, S3T>
where
    S1T: Clone,
    S2T: Clone,
    S3T: Clone,
{
    Eager(StoredVecIterator<'a, I, T>),
    LazyFrom1(LazyVecFrom1Iterator<'a, I, T, S1I, S1T>),
    LazyFrom2(LazyVecFrom2Iterator<'a, I, T, S1I, S1T, S2I, S2T>),
    LazyFrom3(LazyVecFrom3Iterator<'a, I, T, S1I, S1T, S2I, S2T, S3I, S3T>),
}

impl<'a, I, T, S1I, S1T, S2I, S2T, S3I, S3T>
    ComputedVecIterator<'a, I, T, S1I, S1T, S2I, S2T, S3I, S3T>
where
    I: StoredIndex,
    T: StoredCompressed,
    S1I: StoredIndex,
    S1T: StoredRaw,
    S2I: StoredIndex,
    S2T: StoredRaw,
    S3I: StoredIndex,
    S3T: StoredRaw,
{
    pub fn new(computed: &'a ComputedVec<I, T, S1I, S1T, S2I, S2T, S3I, S3T>) -> Self {
        match computed {
            ComputedVec::Eager { vec, .. } => ComputedVecIterator::Eager(vec.into_iter()),
            ComputedVec::LazyFrom1(v) => ComputedVecIterator::LazyFrom1(v.into_iter()),
            ComputedVec::LazyFrom2(v) => ComputedVecIterator::LazyFrom2(v.into_iter()),
            ComputedVec::LazyFrom3(v) => ComputedVecIterator::LazyFrom3(v.into_iter()),
        }
    }
}

impl<'a, I, T, S1I, S1T, S2I, S2T, S3I, S3T> Iterator
    for ComputedVecIterator<'a, I, T, S1I, S1T, S2I, S2T, S3I, S3T>
where
    I: StoredIndex,
    T: StoredCompressed,
    S1I: StoredIndex,
    S1T: StoredRaw,
    S2I: StoredIndex,
    S2T: StoredRaw,
    S3I: StoredIndex,
    S3T: StoredRaw,
{
    type Item = T;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Eager(i) => i.next(),
            Self::LazyFrom1(i) => i.next(),
            Self::LazyFrom2(i) => i.next(),
            Self::LazyFrom3(i) => i.next(),
        }
    }

    #[inline(always)]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        match self {
            Self::Eager(i) => i.nth(n),
            Self::LazyFrom1(i) => i.nth(n),
            Self::LazyFrom2(i) => i.nth(n),
            Self::LazyFrom3(i) => i.nth(n),
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Eager(i) => i.size_hint(),
            Self::LazyFrom1(i) => i.size_hint(),
            Self::LazyFrom2(i) => i.size_hint(),
            Self::LazyFrom3(i) => i.size_hint(),
        }
    }

    #[inline]
    fn count(self) -> usize {
        match self {
            Self::Eager(i) => i.len(),
            Self::LazyFrom1(i) => i.len(),
            Self::LazyFrom2(i) => i.len(),
            Self::LazyFrom3(i) => i.len(),
        }
    }

    #[inline]
    fn last(self) -> Option<T> {
        match self {
            Self::Eager(i) => i.last(),
            Self::LazyFrom1(i) => i.last(),
            Self::LazyFrom2(i) => i.last(),
            Self::LazyFrom3(i) => i.last(),
        }
    }
}

impl<I, T, S1I, S1T, S2I, S2T, S3I, S3T> VecIterator
    for ComputedVecIterator<'_, I, T, S1I, S1T, S2I, S2T, S3I, S3T>
where
    I: StoredIndex,
    T: StoredCompressed,
    S1I: StoredIndex,
    S1T: StoredRaw,
    S2I: StoredIndex,
    S2T: StoredRaw,
    S3I: StoredIndex,
    S3T: StoredRaw,
{
    fn set_position_(&mut self, i: usize) {
        match self {
            Self::Eager(iter) => iter.set_position_(i),
            Self::LazyFrom1(iter) => iter.set_position_(i),
            Self::LazyFrom2(iter) => iter.set_position_(i),
            Self::LazyFrom3(iter) => iter.set_position_(i),
        }
    }

    fn set_end_(&mut self, i: usize) {
        match self {
            Self::Eager(iter) => iter.set_end_(i),
            Self::LazyFrom1(iter) => iter.set_end_(i),
            Self::LazyFrom2(iter) => iter.set_end_(i),
            Self::LazyFrom3(iter) => iter.set_end_(i),
        };
    }
}

impl<I, T, S1I, S1T, S2I, S2T, S3I, S3T> VecIteratorExtended
    for ComputedVecIterator<'_, I, T, S1I, S1T, S2I, S2T, S3I, S3T>
where
    I: StoredIndex,
    T: StoredCompressed,
    S1I: StoredIndex,
    S1T: StoredRaw,
    S2I: StoredIndex,
    S2T: StoredRaw,
    S3I: StoredIndex,
    S3T: StoredRaw,
{
    type I = I;
    type T = T;
}

impl<I, T, S1I, S1T, S2I, S2T, S3I, S3T> ExactSizeIterator
    for ComputedVecIterator<'_, I, T, S1I, S1T, S2I, S2T, S3I, S3T>
where
    I: StoredIndex,
    T: StoredCompressed,
    S1I: StoredIndex,
    S1T: StoredRaw,
    S2I: StoredIndex,
    S2T: StoredRaw,
    S3I: StoredIndex,
    S3T: StoredRaw,
{
    #[inline(always)]
    fn len(&self) -> usize {
        match self {
            Self::Eager(i) => i.len(),
            Self::LazyFrom1(i) => i.len(),
            Self::LazyFrom2(i) => i.len(),
            Self::LazyFrom3(i) => i.len(),
        }
    }
}

impl<I, T, S1I, S1T, S2I, S2T, S3I, S3T> FusedIterator
    for ComputedVecIterator<'_, I, T, S1I, S1T, S2I, S2T, S3I, S3T>
where
    I: StoredIndex,
    T: StoredCompressed,
    S1I: StoredIndex,
    S1T: StoredRaw,
    S2I: StoredIndex,
    S2T: StoredRaw,
    S3I: StoredIndex,
    S3T: StoredRaw,
{
}
