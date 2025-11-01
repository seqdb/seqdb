use std::iter::FusedIterator;

use parking_lot::RwLockReadGuard;
use seqdb::Reader;

use crate::{
    AnyStoredVec, CompressedVec, GenericStoredVec, Result, StoredCompressed, StoredIndex,
    VecIterator, VecIteratorExtended, likely, unlikely, variants::MAX_COMPRESSED_PAGE_SIZE,
};

use super::super::pages::Pages;

/// Clean compressed vec iterator, for reading stored compressed data
pub struct CleanCompressedVecIterator<'a, I, T> {
    pub(crate) _vec: &'a CompressedVec<I, T>,
    reader: Reader<'a>,
    decoded: Option<(usize, Vec<T>)>,
    pages: RwLockReadGuard<'a, Pages>,
    stored_len: usize,
    index: usize,
    end_index: usize,
}

impl<'a, I, T> CleanCompressedVecIterator<'a, I, T>
where
    I: StoredIndex,
    T: StoredCompressed,
{
    const SIZE_OF_T: usize = size_of::<T>();
    const PER_PAGE: usize = MAX_COMPRESSED_PAGE_SIZE / Self::SIZE_OF_T;

    pub fn new(vec: &'a CompressedVec<I, T>) -> Result<Self> {
        let pages = vec.pages.read();
        let stored_len = vec.stored_len();

        Ok(Self {
            _vec: vec,
            reader: vec.create_reader(),
            decoded: None,
            pages,
            stored_len,
            index: 0,
            end_index: stored_len,
        })
    }

    #[inline(always)]
    fn remaining(&self) -> usize {
        self.end_index.saturating_sub(self.index)
    }
}

impl<I, T> Iterator for CleanCompressedVecIterator<'_, I, T>
where
    I: StoredIndex,
    T: StoredCompressed,
{
    type Item = T;

    #[inline(always)]
    fn next(&mut self) -> Option<T> {
        let index = self.index;

        if unlikely(index >= self.end_index) {
            return None;
        }

        self.index += 1;

        let page_index = index / Self::PER_PAGE;
        let in_page_index = index % Self::PER_PAGE;

        // Fast path: read from current decoded page
        if likely(
            self.decoded
                .as_ref()
                .is_some_and(|(pi, _)| *pi == page_index),
        ) {
            let (_, values) = self.decoded.as_ref().unwrap();
            return values.get(in_page_index).copied();
        }

        // Slow path: decode new page
        let values = CompressedVec::<I, T>::decode_page_(
            self.stored_len,
            page_index,
            &self.reader,
            &self.pages,
        )
        .ok()?;

        let value = values.get(in_page_index).copied();
        self.decoded = Some((page_index, values));
        value
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<T> {
        if n == 0 {
            return self.next();
        }

        let new_index = self.index.saturating_add(n);
        if new_index >= self.end_index {
            self.index = self.end_index;
            return None;
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

    #[inline]
    fn last(mut self) -> Option<T> {
        if unlikely(self.index >= self.end_index) {
            return None;
        }

        self.index = self.end_index - 1;
        self.next()
    }
}

impl<I, T> ExactSizeIterator for CleanCompressedVecIterator<'_, I, T>
where
    I: StoredIndex,
    T: StoredCompressed,
{
    #[inline(always)]
    fn len(&self) -> usize {
        self.remaining()
    }
}

impl<I, T> FusedIterator for CleanCompressedVecIterator<'_, I, T>
where
    I: StoredIndex,
    T: StoredCompressed,
{
}

impl<I, T> VecIterator for CleanCompressedVecIterator<'_, I, T>
where
    I: StoredIndex,
    T: StoredCompressed,
{
    fn set_position_(&mut self, i: usize) {
        let new_index = i.min(self.stored_len).min(self.end_index);

        // Check if new position is within the currently decoded page
        if let Some((page_index, _)) = &self.decoded {
            let page_start = page_index * Self::PER_PAGE;
            let page_end = page_start + Self::PER_PAGE;

            if new_index >= page_start && new_index < page_end {
                // Keep decoded page, just update index
                self.index = new_index;
                return;
            }
        }

        // New position is outside current page, invalidate cache
        self.decoded = None;
        self.index = new_index;
    }

    fn set_end_(&mut self, i: usize) {
        self.end_index = i.min(self.stored_len);
    }

    fn skip_optimized(mut self, n: usize) -> Self {
        self.index = self.index.saturating_add(n).min(self.end_index);
        self
    }

    fn take_optimized(mut self, n: usize) -> Self {
        self.end_index = self.index.saturating_add(n).min(self.end_index);
        self
    }
}

impl<I, T> VecIteratorExtended for CleanCompressedVecIterator<'_, I, T>
where
    I: StoredIndex,
    T: StoredCompressed,
{
    type I = I;
    type T = T;
}
