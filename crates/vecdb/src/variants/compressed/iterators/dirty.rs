use parking_lot::RwLockReadGuard;
use seqdb::Region;

use crate::{AnyStoredVec, CompressedVec, GenericStoredVec, Result, StoredCompressed, StoredIndex};

use super::CleanCompressedVecValues;

pub struct DirtyCompressedVecIterator<'a, I, T> {
    index: usize,
    values: DirtyCompressedVecValues<'a, I, T>,
}

impl<'a, I, T> DirtyCompressedVecIterator<'a, I, T>
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
        Ok(Self {
            index,
            values: DirtyCompressedVecValues::new_at(vec, index)?,
        })
    }
}

impl<I, T> Iterator for DirtyCompressedVecIterator<'_, I, T>
where
    I: StoredIndex,
    T: StoredCompressed,
{
    type Item = (I, T);

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        let index = I::from(self.index);
        let value = self.values.next()?;
        self.index += 1;
        Some((index, value))
    }
}

pub struct DirtyCompressedVecValues<'a, I, T> {
    inner: CleanCompressedVecValues<'a, I, T>,
    // index: usize,
    // stored_len: usize,
    _lock: RwLockReadGuard<'a, Region>,
}

impl<'a, I, T> DirtyCompressedVecValues<'a, I, T>
where
    I: StoredIndex,
    T: StoredCompressed,
{
    pub fn new(vec: &'a CompressedVec<I, T>) -> Result<Self> {
        Self::new_at(vec, 0)
    }

    pub fn new_at(vec: &'a CompressedVec<I, T>, index: usize) -> Result<Self> {
        // let stored_len = vec.stored_len();
        let region = vec.region().read();

        Ok(Self {
            inner: CleanCompressedVecValues::new_at(vec, index)?,
            // index,
            // stored_len,
            _lock: region,
        })
    }
}

impl<I, T> Iterator for DirtyCompressedVecValues<'_, I, T>
where
    I: StoredIndex,
    T: StoredCompressed,
{
    type Item = T;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        let index = self.inner.index;
        self.inner.index += 1;

        let stored_len = self.inner.stored_len;

        if index >= stored_len {
            return self.inner._vec.get_pushed(index, stored_len).copied();
        }

        self.inner.next()
    }
}
