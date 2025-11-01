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

    /// Skip one stored element without reading it (for holes/updates optimization)
    #[inline(always)]
    fn skip_stored_element(&mut self) {
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

    /// Set the absolute end position for the iterator
    #[inline(always)]
    fn set_absolute_end(&mut self, absolute_end: usize) {
        let new_total_len = absolute_end.min(self.vec_len());
        let new_pushed_len = new_total_len.saturating_sub(self.stored_len);
        self.pushed_len = new_pushed_len;

        // Cap inner iterator if new end is within stored range
        if absolute_end <= self.stored_len {
            self.inner.set_end_(absolute_end);
        }
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
            self.skip_stored_element();
            return self.next();
        }

        if index >= self.stored_len {
            return self.inner._vec.get_pushed(index, self.stored_len).cloned();
        }

        if unlikely(self.updated)
            && let Some(updated) = self.inner._vec.updated().get(&index)
        {
            self.skip_stored_element();
            return Some(updated.clone());
        }

        self.inner.next()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<T> {
        if n == 0 {
            return self.next();
        }

        let new_index = self.index.saturating_add(n);
        if new_index >= self.vec_len() {
            self.index = self.vec_len();
            return None;
        }

        // Fast path: no holes or updates, can use optimized inner nth
        if !self.holes && !self.updated {
            if new_index < self.stored_len {
                // All skips are in stored data
                self.inner.nth(n - 1)?;
                self.index = new_index;
                return self.next();
            } else if self.index < self.stored_len {
                // Skip to end of stored, then into pushed
                let stored_skip = self.stored_len - self.index;
                if stored_skip > 0 {
                    self.inner.nth(stored_skip - 1);
                }
                self.index = new_index;
                return self.next();
            } else {
                // Already in pushed, just update index
                self.index = new_index;
                return self.next();
            }
        }

        // Slow path: need to check each element for holes/updates
        for _ in 0..n {
            if self.index >= self.vec_len() {
                self.index = self.vec_len();
                return None;
            }
            self.skip_stored_element();
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
        self.set_absolute_end(i);
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
        let absolute_end = self.index.saturating_add(n);
        self.set_absolute_end(absolute_end);
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
