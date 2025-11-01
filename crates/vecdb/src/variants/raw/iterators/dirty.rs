use std::iter::FusedIterator;

use crate::{
    AnyStoredVec, GenericStoredVec, RawVec, Result, StoredIndex, StoredRaw, VecIterator,
    VecIteratorExtended, likely, unlikely,
};

use super::CleanRawVecIterator;

/// Dirty raw vec iterator, full-featured with holes/updates/pushed support
pub struct DirtyRawVecIterator<'a, I, T> {
    inner: CleanRawVecIterator<'a, I, T>,
    index: usize,
    stored_len: usize,
    pushed_len: usize,
    holes: bool,
    updated: bool,
}

impl<'a, I, T> DirtyRawVecIterator<'a, I, T>
where
    I: StoredIndex,
    T: StoredRaw,
{
    const SIZE_OF_T: usize = size_of::<T>();

    pub fn new(vec: &'a RawVec<I, T>) -> Result<Self> {
        let holes = !vec.holes.is_empty();
        let updated = !vec.updated.is_empty();

        let stored_len = vec.stored_len();
        let pushed_len = vec.pushed_len();

        Ok(Self {
            inner: CleanRawVecIterator::new(vec)?,
            index: 0,
            stored_len,
            pushed_len,
            holes,
            updated,
        })
    }

    #[inline(always)]
    fn skip_element_if_needed(&mut self) {
        if self.index >= self.stored_len {
            return;
        }

        self.inner.buffer_pos += Self::SIZE_OF_T;

        if unlikely(self.inner.cant_read_buffer()) && likely(self.inner.can_read_file()) {
            self.inner.refill_buffer();
        }
    }

    #[inline(always)]
    fn remaining(&self) -> usize {
        (self.vec_len()) - self.index
    }

    #[inline(always)]
    fn vec_len(&self) -> usize {
        self.stored_len + self.pushed_len
    }
}

impl<I, T> Iterator for DirtyRawVecIterator<'_, I, T>
where
    I: StoredIndex,
    T: StoredRaw,
{
    type Item = T;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index;
        self.index += 1;

        if unlikely(self.holes) && self.inner._vec.holes().contains(&index) {
            self.skip_element_if_needed();
            return self.next();
        }

        if index >= self.stored_len {
            return self.inner._vec.get_pushed(index, self.stored_len).cloned();
        }

        if unlikely(self.updated)
            && let Some(updated) = self.inner._vec.updated().get(&index)
        {
            self.skip_element_if_needed();
            return Some(updated.clone());
        }

        self.inner.next()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<T> {
        for _ in 0..n {
            if self.index >= self.vec_len() {
                self.index = self.vec_len();
                return None;
            }
            self.skip_element_if_needed();
            self.index += 1;
        }
        self.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.remaining();
        (remaining, Some(remaining))
    }

    #[inline]
    fn count(self) -> usize {
        self.len()
    }

    fn last(mut self) -> Option<T> {
        let last_index = self.vec_len().checked_sub(1)?;
        self.nth(last_index - self.index)
    }
}

impl<I, T> ExactSizeIterator for DirtyRawVecIterator<'_, I, T>
where
    I: StoredIndex,
    T: StoredRaw,
{
    #[inline(always)]
    fn len(&self) -> usize {
        self.remaining()
    }
}

impl<I, T> FusedIterator for DirtyRawVecIterator<'_, I, T>
where
    I: StoredIndex,
    T: StoredRaw,
{
}

impl<I, T> VecIterator for DirtyRawVecIterator<'_, I, T>
where
    I: StoredIndex,
    T: StoredRaw,
{
    fn set_position_(&mut self, i: usize) {
        self.index = i.min(self.vec_len());

        // Update inner iterator position if within stored range
        if i < self.stored_len {
            self.inner.set_position_(i);
        }
    }

    fn set_end_(&mut self, i: usize) {
        let new_total_len = i.min(self.vec_len());
        let new_pushed_len = new_total_len.saturating_sub(self.stored_len);
        self.pushed_len = new_pushed_len;

        // Cap inner iterator if new end is within stored range
        if i <= self.stored_len {
            self.inner.set_end_(i);
        }
    }

    fn skip_optimized(mut self, n: usize) -> Self {
        let stored_skip = n.min(self.stored_len.saturating_sub(self.index));
        if stored_skip > 0 {
            self.inner = self.inner.skip_optimized(stored_skip);
        }
        self.index = self.index.saturating_add(n).min(self.vec_len());
        self
    }

    fn take_optimized(mut self, n: usize) -> Self {
        let new_total_len = self.index.saturating_add(n);
        let new_pushed_len = new_total_len.saturating_sub(self.stored_len);
        self.pushed_len = self.pushed_len.min(new_pushed_len);

        let stored_remaining = self.stored_len.saturating_sub(self.index);
        let inner_take = n.min(stored_remaining);
        self.inner = self.inner.take_optimized(inner_take);

        self
    }
}

impl<I, T> VecIteratorExtended for DirtyRawVecIterator<'_, I, T>
where
    I: StoredIndex,
    T: StoredRaw,
{
    type I = I;
    type T = T;
}
