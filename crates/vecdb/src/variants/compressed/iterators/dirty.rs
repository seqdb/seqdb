use std::iter::FusedIterator;

use crate::{
    AnyStoredVec, CompressedVec, GenericStoredVec, Result, StoredCompressed, StoredIndex,
    VecIterator, VecIteratorExtended, likely,
};

use super::CleanCompressedVecIterator;

/// Dirty compressed vec iterator, handles pushed values on top of stored data
pub struct DirtyCompressedVecIterator<'a, I, T> {
    inner: CleanCompressedVecIterator<'a, I, T>,
    index: usize,
    stored_len: usize,
    pushed_len: usize,
}

impl<'a, I, T> DirtyCompressedVecIterator<'a, I, T>
where
    I: StoredIndex,
    T: StoredCompressed,
{
    pub fn new(vec: &'a CompressedVec<I, T>) -> Result<Self> {
        let stored_len = vec.stored_len();
        let pushed_len = vec.pushed_len();

        Ok(Self {
            inner: CleanCompressedVecIterator::new(vec)?,
            index: 0,
            stored_len,
            pushed_len,
        })
    }

    #[inline(always)]
    fn remaining(&self) -> usize {
        self.vec_len() - self.index
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

impl<I, T> Iterator for DirtyCompressedVecIterator<'_, I, T>
where
    I: StoredIndex,
    T: StoredCompressed,
{
    type Item = T;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index;
        self.index += 1;

        if likely(index < self.stored_len) {
            return self.inner.next();
        }

        self.inner._vec.get_pushed(index, self.stored_len).copied()
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

        // Optimize: skip within inner iterator if crossing stored range
        if self.index < self.stored_len && new_index > self.index {
            let skip_in_stored = new_index.min(self.stored_len) - self.index;
            self.inner.nth(skip_in_stored - 1)?;
        }

        self.index = new_index;
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

    fn last(self) -> Option<T> {
        let last_index = self.vec_len().checked_sub(1)?;

        if last_index < self.stored_len {
            // Last element is in stored data
            self.inner.last()
        } else {
            // Last element is in pushed data
            self.inner
                ._vec
                .get_pushed(last_index, self.stored_len)
                .copied()
        }
    }
}

impl<I, T> ExactSizeIterator for DirtyCompressedVecIterator<'_, I, T>
where
    I: StoredIndex,
    T: StoredCompressed,
{
    #[inline(always)]
    fn len(&self) -> usize {
        self.remaining()
    }
}

impl<I, T> FusedIterator for DirtyCompressedVecIterator<'_, I, T>
where
    I: StoredIndex,
    T: StoredCompressed,
{
}

impl<I, T> VecIterator for DirtyCompressedVecIterator<'_, I, T>
where
    I: StoredIndex,
    T: StoredCompressed,
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

impl<I, T> VecIteratorExtended for DirtyCompressedVecIterator<'_, I, T>
where
    I: StoredIndex,
    T: StoredCompressed,
{
    type I = I;
    type T = T;
}
