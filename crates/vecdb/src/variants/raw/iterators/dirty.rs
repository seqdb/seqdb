use parking_lot::RwLockReadGuard;
use seqdb::Region;

use crate::{
    AnyStoredVec, GenericStoredVec, RawVec, Result, StoredIndex, StoredRaw, likely, unlikely,
    variants::CleanRawVecValues,
};

pub struct DirtyRawVecIterator<'a, I, T> {
    index: usize,
    values: DirtyRawVecValues<'a, I, T>,
}

impl<'a, I, T> DirtyRawVecIterator<'a, I, T>
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
        Ok(Self {
            index,
            values: DirtyRawVecValues::new_at(vec, index)?,
        })
    }
}

impl<I, T> Iterator for DirtyRawVecIterator<'_, I, T>
where
    I: StoredIndex,
    T: StoredRaw,
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

/// Dirty vec iterator - full-featured with holes/updates/pushed support
pub struct DirtyRawVecValues<'a, I, T> {
    inner: CleanRawVecValues<'a, I, T>,
    index: usize,
    stored_len: usize,
    holes: bool,
    updated: bool,
    _lock: RwLockReadGuard<'a, Region>,
}

impl<'a, I, T> DirtyRawVecValues<'a, I, T>
where
    I: StoredIndex,
    T: StoredRaw,
{
    const SIZE_OF_T: usize = size_of::<T>();

    pub fn new(vec: &'a RawVec<I, T>) -> Result<Self> {
        Self::new_at(vec, 0)
    }

    pub fn new_at(vec: &'a RawVec<I, T>, index: usize) -> Result<Self> {
        let holes = !vec.holes.is_empty();
        let updated = !vec.updated.is_empty();

        let stored_len = vec.stored_len();
        let region = vec.region.read();

        Ok(Self {
            inner: CleanRawVecValues::new_at(vec, index)?,
            index,
            stored_len,
            holes,
            updated,
            _lock: region,
        })
    }

    /// Skip one element in the inner iterator's buffer, seeking forward if needed
    #[inline(always)]
    fn skip_element(&mut self) {
        self.inner.buffer_pos += Self::SIZE_OF_T;

        if unlikely(self.inner.cant_read_buffer()) && likely(self.inner.can_read_file()) {
            self.inner.refill_buffer();
        }
    }
}

impl<I, T> Iterator for DirtyRawVecValues<'_, I, T>
where
    I: StoredIndex,
    T: StoredRaw,
{
    type Item = T;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index;
        self.index += 1;

        let stored_len = self.stored_len;

        if unlikely(self.holes) && self.inner._vec.holes().contains(&index) {
            self.skip_element();
            return None;
        }

        if index >= self.stored_len {
            return self.inner._vec.get_pushed(index, stored_len).cloned();
        }

        if unlikely(self.updated)
            && let Some(updated) = self.inner._vec.updated().get(&index)
        {
            self.skip_element();
            return Some(updated.clone());
        }

        self.inner.next()
    }
}
