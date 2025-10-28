use crate::{BaseVecIterator, CompressedVec, Result, StoredCompressed, StoredIndex};

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
        Self::new_at(vec, 0)
    }

    #[inline]
    pub fn new_at(vec: &'a CompressedVec<I, T>, index: usize) -> Result<Self> {
        Ok(if vec.is_dirty() {
            Self::Dirty(DirtyCompressedVecIterator::new_at(vec, index)?)
        } else {
            Self::Clean(CleanCompressedVecIterator::new_at(vec, index)?)
        })
    }
}

impl<I, T> Iterator for CompressedVecIterator<'_, I, T>
where
    I: StoredIndex,
    T: StoredCompressed,
{
    type Item = (I, T);

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            CompressedVecIterator::Clean(iter) => iter.next(),
            CompressedVecIterator::Dirty(iter) => iter.next(),
        }
    }
}

impl<I, T> BaseVecIterator for CompressedVecIterator<'_, I, T>
where
    I: StoredIndex,
    T: StoredCompressed,
{
    fn mut_index(&mut self) -> &mut usize {
        todo!();
        // match self {
        //     CompressedVecIterator::Clean(iter) => &mut iter.index,
        //     CompressedVecIterator::Dirty(iter) => &mut iter.index,
        // }
    }

    fn len(&self) -> usize {
        todo!();
        // match self {
        //     CompressedVecIterator::Clean(iter) => iter.stream.vec.len(),
        //     CompressedVecIterator::Dirty(iter) => iter.vec.len(),
        // }
    }

    fn name(&self) -> &str {
        todo!();
        // match self {
        //     CompressedVecIterator::Clean(iter) => iter.stream.vec.name(),
        //     CompressedVecIterator::Dirty(iter) => iter.vec.name(),
        // }
    }
}

// -------

/// Main values enum - uses fast path for clean vecs, full path for dirty vecs
pub enum CompressedVecValues<'a, I, T> {
    Clean(CleanCompressedVecValues<'a, I, T>),
    Dirty(DirtyCompressedVecValues<'a, I, T>),
}

impl<'a, I, T> CompressedVecValues<'a, I, T>
where
    I: StoredIndex,
    T: StoredCompressed,
{
    #[inline]
    pub fn new(vec: &'a CompressedVec<I, T>) -> Result<Self> {
        Self::new_at(vec, 0)
    }

    #[inline]
    pub fn new_at(vec: &'a CompressedVec<I, T>, index: usize) -> Result<Self> {
        Ok(if vec.is_dirty() {
            Self::Dirty(DirtyCompressedVecValues::new_at(vec, index)?)
        } else {
            Self::Clean(CleanCompressedVecValues::new_at(vec, index)?)
        })
    }
}

impl<I, T> Iterator for CompressedVecValues<'_, I, T>
where
    I: StoredIndex,
    T: StoredCompressed,
{
    type Item = T;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            CompressedVecValues::Clean(iter) => iter.next(),
            CompressedVecValues::Dirty(iter) => iter.next(),
        }
    }
}
