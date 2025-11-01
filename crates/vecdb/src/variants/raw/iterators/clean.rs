use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    iter::FusedIterator,
};

use parking_lot::RwLockReadGuard;
use seqdb::Region;

use crate::{
    RawVec, Result, StoredIndex, StoredRaw, VecIterator, VecIteratorExtended, likely, unlikely,
    variants::HEADER_OFFSET,
};

/// Clean raw vec iterator, to read on disk data
pub struct CleanRawVecIterator<'a, I, T> {
    pub(crate) file: File,
    buffer: Vec<u8>,
    pub(crate) buffer_pos: usize,
    buffer_len: usize,
    file_offset: u64,
    end_offset: u64,
    start_offset: u64,
    pub(crate) _vec: &'a RawVec<I, T>,
    _lock: RwLockReadGuard<'a, Region>,
}

impl<'a, I, T> CleanRawVecIterator<'a, I, T>
where
    I: StoredIndex,
    T: StoredRaw,
{
    const SIZE_OF_T: usize = size_of::<T>();
    const NORMAL_BUFFER_SIZE: usize = RawVec::<I, T>::aligned_buffer_size();
    const CHECK_T: () = assert!(Self::SIZE_OF_T > 0, "Can't have T with size_of() == 0");

    pub fn new(vec: &'a RawVec<I, T>) -> Result<Self> {
        let region = vec.region.read();
        let file = vec.db.open_read_only_file()?;

        let region_start = region.start();
        let start_offset = region_start + HEADER_OFFSET as u64;

        let mut this = Self {
            file,
            buffer: vec![0; Self::NORMAL_BUFFER_SIZE],
            buffer_pos: 0,
            buffer_len: 0,
            file_offset: start_offset,
            end_offset: region_start + region.len(),
            start_offset,
            _vec: vec,
            _lock: region,
        };

        this.seek(start_offset);

        Ok(this)
    }

    #[inline(always)]
    fn seek(&mut self, pos: u64) -> bool {
        self.file_offset = pos.min(self.end_offset).max(self.start_offset);
        self.buffer_pos = 0;
        self.buffer_len = 0;

        if likely(self.can_read_file()) {
            self.file
                .seek(SeekFrom::Start(self.file_offset))
                .expect("Failed to seek to start position");
            true
        } else {
            false
        }
    }

    #[inline(always)]
    pub(crate) fn can_read_buffer(&self) -> bool {
        self.buffer_pos < self.buffer_len
    }

    #[inline(always)]
    pub(crate) fn cant_read_buffer(&self) -> bool {
        self.buffer_pos >= self.buffer_len
    }

    #[inline(always)]
    pub(crate) fn can_read_file(&self) -> bool {
        self.file_offset < self.end_offset
    }

    #[inline(always)]
    pub(crate) fn cant_read_file(&self) -> bool {
        self.file_offset >= self.end_offset
    }

    #[inline(always)]
    pub(crate) fn remaining_file_bytes(&self) -> usize {
        (self.end_offset - self.file_offset) as usize
    }

    #[inline(always)]
    pub(crate) fn remaining_buffer_bytes(&self) -> usize {
        self.buffer_len - self.buffer_pos
    }

    #[inline(always)]
    pub(crate) fn remaining_bytes(&self) -> usize {
        self.remaining_file_bytes() + self.remaining_buffer_bytes()
    }

    #[inline(always)]
    pub(crate) fn remaining(&self) -> usize {
        self.remaining_bytes() / Self::SIZE_OF_T
    }

    #[inline(always)]
    pub(crate) fn refill_buffer(&mut self) {
        let buffer_len = self.remaining_file_bytes().min(Self::NORMAL_BUFFER_SIZE);

        unsafe {
            self.file
                .read_exact(&mut self.buffer[..buffer_len])
                .unwrap_unchecked()
        };

        self.file_offset += buffer_len as u64;
        self.buffer_len = buffer_len;
        self.buffer_pos = Self::SIZE_OF_T;
    }
}

impl<I, T> Iterator for CleanRawVecIterator<'_, I, T>
where
    I: StoredIndex,
    T: StoredRaw,
{
    type Item = T;

    #[inline(always)]
    fn next(&mut self) -> Option<T> {
        if likely(self.can_read_buffer()) {
            let value = unsafe {
                std::ptr::read_unaligned(self.buffer.as_ptr().add(self.buffer_pos) as *const T)
            };
            self.buffer_pos += Self::SIZE_OF_T;
            return Some(value);
        }

        if unlikely(self.cant_read_file()) {
            return None;
        }

        self.refill_buffer();

        Some(unsafe { std::ptr::read_unaligned(self.buffer.as_ptr() as *const T) })
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<T> {
        if n == 0 {
            return self.next();
        }

        let skip_bytes = n.saturating_mul(Self::SIZE_OF_T);
        let buffer_remaining = self.remaining_buffer_bytes();
        if skip_bytes < buffer_remaining {
            self.buffer_pos += skip_bytes;
            return self.next();
        }

        if !self.seek(
            self.file_offset
                .saturating_add((skip_bytes - buffer_remaining) as u64),
        ) {
            return None;
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

    #[inline]
    fn last(mut self) -> Option<T> {
        if unlikely(self.cant_read_file() || self.start_offset == self.end_offset) {
            return None;
        }

        self.seek(self.end_offset - Self::SIZE_OF_T as u64);

        self.next()
    }
}

impl<I, T> ExactSizeIterator for CleanRawVecIterator<'_, I, T>
where
    I: StoredIndex,
    T: StoredRaw,
{
    #[inline(always)]
    fn len(&self) -> usize {
        self.remaining()
    }
}

impl<I, T> FusedIterator for CleanRawVecIterator<'_, I, T>
where
    I: StoredIndex,
    T: StoredRaw,
{
}

impl<I, T> VecIterator for CleanRawVecIterator<'_, I, T>
where
    I: StoredIndex,
    T: StoredRaw,
{
    fn skip_optimized(mut self, n: usize) -> Self {
        self.seek(
            self.file_offset
                .saturating_add((n.saturating_mul(Self::SIZE_OF_T)) as u64),
        );
        self
    }

    fn take_optimized(mut self, n: usize) -> Self {
        self.end_offset = self.end_offset.min(
            self.file_offset
                .saturating_add((n.saturating_mul(Self::SIZE_OF_T)) as u64),
        );
        self
    }
}

impl<I, T> VecIteratorExtended for CleanRawVecIterator<'_, I, T>
where
    I: StoredIndex,
    T: StoredRaw,
{
    type I = I;
    type T = T;
}
