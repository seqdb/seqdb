use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
};

use parking_lot::RwLockReadGuard;
use seqdb::Region;

use crate::{RawVec, Result, StoredIndex, StoredRaw, likely, unlikely, variants::HEADER_OFFSET};

pub struct CleanRawVecIterator<'a, I, T> {
    index: usize,
    values: CleanRawVecValues<'a, I, T>,
}

impl<'a, I, T> CleanRawVecIterator<'a, I, T>
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
            values: CleanRawVecValues::new_at(vec, index)?,
        })
    }
}

impl<I, T> Iterator for CleanRawVecIterator<'_, I, T>
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

pub struct CleanRawVecValues<'a, I, T> {
    file: File,
    buffer: Vec<u8>,
    buffer_pos: usize,
    buffer_len: usize,
    file_offset: u64,
    end_offset: u64,
    _vec: &'a RawVec<I, T>,
    _lock: RwLockReadGuard<'a, Region>,
}

impl<'a, I, T> CleanRawVecValues<'a, I, T>
where
    I: StoredIndex,
    T: StoredRaw,
{
    const SIZE_OF_T: usize = size_of::<T>();

    pub fn new(vec: &'a RawVec<I, T>) -> Result<Self> {
        Self::new_at(vec, 0)
    }

    pub fn new_at(vec: &'a RawVec<I, T>, index: usize) -> Result<Self> {
        let region = vec.region.read();

        let region_start = region.start();
        let region_len = region.len();

        let mut file = vec.db.open_read_only_file()?;

        let start_offset = region_start + HEADER_OFFSET as u64 + (index * size_of::<T>()) as u64;

        file.seek(SeekFrom::Start(start_offset))
            .expect("Failed to seek to start position");

        Ok(Self {
            file,
            buffer: vec![0; RawVec::<I, T>::aligned_buffer_size()],
            buffer_pos: 0,
            buffer_len: 0,
            file_offset: start_offset,
            end_offset: region_start + region_len,
            _vec: vec,
            _lock: region,
        })
    }
}

impl<I, T> Iterator for CleanRawVecValues<'_, I, T>
where
    I: StoredIndex,
    T: StoredRaw,
{
    type Item = T;

    #[inline(always)]
    fn next(&mut self) -> Option<T> {
        // Fast path: read from current buffer
        if likely(self.buffer_pos < self.buffer_len) {
            let value = unsafe {
                std::ptr::read_unaligned(self.buffer.as_ptr().add(self.buffer_pos) as *const T)
            };
            self.buffer_pos += Self::SIZE_OF_T;
            return Some(value);
        }

        // Slowest path: Stop
        if unlikely(self.file_offset >= self.end_offset) {
            return None;
        }

        // Slow path: refill buffer and read
        let remaining = (self.end_offset - self.file_offset) as usize;
        let to_read = remaining.min(self.buffer.len());

        // Safety: we're within file bounds, read should succeed
        unsafe {
            self.file
                .read_exact(&mut self.buffer[..to_read])
                .unwrap_unchecked()
        };

        self.file_offset += to_read as u64;
        self.buffer_len = to_read;
        self.buffer_pos = Self::SIZE_OF_T;

        Some(unsafe { std::ptr::read_unaligned(self.buffer.as_ptr() as *const T) })
    }
}
