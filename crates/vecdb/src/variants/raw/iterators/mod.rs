use crate::{BaseVecIterator, RawVec, Result, StoredIndex, StoredRaw};

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
        Self::new_at(vec, 0)
    }

    #[inline]
    pub fn new_at(vec: &'a RawVec<I, T>, index: usize) -> Result<Self> {
        Ok(if vec.is_dirty() {
            Self::Dirty(DirtyRawVecIterator::new_at(vec, index)?)
        } else {
            Self::Clean(CleanRawVecIterator::new_at(vec, index)?)
        })
    }
}

impl<I, T> Iterator for RawVecIterator<'_, I, T>
where
    I: StoredIndex,
    T: StoredRaw,
{
    type Item = (I, T);

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            RawVecIterator::Clean(iter) => iter.next(),
            RawVecIterator::Dirty(iter) => iter.next(),
        }
    }
}

impl<I, T> BaseVecIterator for RawVecIterator<'_, I, T>
where
    I: StoredIndex,
    T: StoredRaw,
{
    fn mut_index(&mut self) -> &mut usize {
        todo!();
        // match self {
        //     RawVecIterator::Clean(iter) => &mut iter.index,
        //     RawVecIterator::Dirty(iter) => &mut iter.index,
        // }
    }

    fn len(&self) -> usize {
        todo!();
        // match self {
        //     RawVecIterator::Clean(iter) => iter.stream.vec.len(),
        //     RawVecIterator::Dirty(iter) => iter.vec.len(),
        // }
    }

    fn name(&self) -> &str {
        todo!();
        // match self {
        //     RawVecIterator::Clean(iter) => iter.stream.vec.name(),
        //     RawVecIterator::Dirty(iter) => iter.vec.name(),
        // }
    }
}

// -------

/// Main values enum - uses fast path for clean vecs, full path for dirty vecs
pub enum RawVecValues<'a, I, T> {
    Clean(CleanRawVecValues<'a, I, T>),
    Dirty(DirtyRawVecValues<'a, I, T>),
}

impl<'a, I, T> RawVecValues<'a, I, T>
where
    I: StoredIndex,
    T: StoredRaw,
{
    #[inline]
    pub fn new(vec: &'a RawVec<I, T>) -> Result<Self> {
        Self::new_at(vec, 0)
    }

    #[inline]
    pub fn new_at(vec: &'a RawVec<I, T>, index: usize) -> Result<Self> {
        Ok(if vec.is_dirty() {
            Self::Dirty(DirtyRawVecValues::new_at(vec, index)?)
        } else {
            Self::Clean(CleanRawVecValues::new_at(vec, index)?)
        })
    }
}

impl<I, T> Iterator for RawVecValues<'_, I, T>
where
    I: StoredIndex,
    T: StoredRaw,
{
    type Item = T;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            RawVecValues::Clean(iter) => iter.next(),
            RawVecValues::Dirty(iter) => iter.next(),
        }
    }
}
