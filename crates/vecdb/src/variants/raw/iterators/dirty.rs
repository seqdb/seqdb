use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
};

use parking_lot::RwLockReadGuard;
use seqdb::Region;

use crate::{
    AnyStoredVec, GenericStoredVec, RawVec, Result, StoredIndex, StoredRaw, VEC_PAGE_SIZE,
    unlikely, variants::HEADER_OFFSET,
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
    // HOT: accessed every iteration (first cache line)
    index: usize,
    stored_len: usize,
    current_page: usize,
    cursor_page: usize,
    holes: bool,
    updated: bool,

    // WARM: accessed per page
    file: File,
    region_start: u64,
    buffer: Vec<u8>,

    // COLD: only accessed in slow path or for lifetimes
    pub vec: &'a RawVec<I, T>,
    _lock: RwLockReadGuard<'a, Region>,
}

impl<'a, I, T> DirtyRawVecValues<'a, I, T>
where
    I: StoredIndex,
    T: StoredRaw,
{
    const SIZE_OF_T: usize = size_of::<T>();
    const PER_PAGE: usize = VEC_PAGE_SIZE / Self::SIZE_OF_T;

    // Check if we can use bit shift optimizations
    const IS_SIZE_POW2: bool =
        Self::SIZE_OF_T > 0 && (Self::SIZE_OF_T & (Self::SIZE_OF_T - 1)) == 0;
    const IS_PER_PAGE_POW2: bool =
        Self::PER_PAGE > 0 && (Self::PER_PAGE & (Self::PER_PAGE - 1)) == 0;

    // Bit shift amounts (only valid when power-of-2)
    const PAGE_SHIFT: u32 = Self::PER_PAGE.trailing_zeros();
    const SIZE_SHIFT: u32 = Self::SIZE_OF_T.trailing_zeros();
    const PAGE_MASK: usize = Self::PER_PAGE - 1;

    pub fn new(vec: &'a RawVec<I, T>) -> Result<Self> {
        Self::new_at(vec, 0)
    }

    pub fn new_at(vec: &'a RawVec<I, T>, index: usize) -> Result<Self> {
        let holes = !vec.holes.is_empty();
        let updated = !vec.updated.is_empty();

        // Use full-featured iterator for dirty vecs
        let stored_len = vec.stored_len();
        let region = vec.region.read();
        let region_start = region.start() + HEADER_OFFSET as u64;

        let file = vec
            .db
            .open_read_only_file()
            .expect("Failed to open read only file");

        Ok(Self {
            index,
            stored_len,
            current_page: usize::MAX,
            cursor_page: usize::MAX,
            holes,
            updated,
            file,
            region_start,
            buffer: vec![0; RawVec::<I, T>::aligned_buffer_size()],
            vec,
            _lock: region,
        })
    }

    #[inline(always)]
    fn index_to_page(index: usize) -> usize {
        if Self::IS_PER_PAGE_POW2 {
            index >> Self::PAGE_SHIFT
        } else {
            index / Self::PER_PAGE
        }
    }

    #[inline(always)]
    fn index_in_page(index: usize) -> usize {
        if Self::IS_PER_PAGE_POW2 {
            index & Self::PAGE_MASK
        } else {
            index % Self::PER_PAGE
        }
    }

    #[inline(always)]
    fn mul_size(n: usize) -> usize {
        if Self::IS_SIZE_POW2 {
            n << Self::SIZE_SHIFT
        } else {
            n * Self::SIZE_OF_T
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

        if self.holes && self.vec.holes().contains(&index) {
            return None;
        }

        if index >= stored_len {
            return self.vec.get_pushed(index, stored_len).cloned();
        }

        if self.updated
            && let Some(updated) = self.vec.updated().get(&index)
        {
            return Some(updated.clone());
        }

        let page_index = Self::index_to_page(index);
        let buffer_index = Self::mul_size(Self::index_in_page(index));

        if unlikely(page_index != self.current_page) {
            let remaining = self.stored_len - index;
            let new_len = Self::mul_size(remaining.min(Self::PER_PAGE));

            if unlikely(self.cursor_page != page_index) {
                let offset = self.region_start + Self::mul_size(page_index * Self::PER_PAGE) as u64;
                // Safety: seek position is always valid (within file bounds)
                unsafe { self.file.seek(SeekFrom::Start(offset)).unwrap_unchecked() };
            }

            // Read into buffer
            // Safety: buffer is correctly sized and file has enough data
            unsafe {
                self.file
                    .read_exact(&mut self.buffer[..new_len])
                    .unwrap_unchecked()
            };
            self.current_page = page_index;
            self.cursor_page = page_index + 1;
        }

        // Safety: buffer_index is guaranteed to be in bounds by page logic
        // and properly aligned since buffer_index is always a multiple of SIZE_OF_T
        // and Vec allocator provides proper alignment
        Some(unsafe {
            std::ptr::read_unaligned(self.buffer.as_ptr().add(buffer_index) as *const T)
        })
    }
}

// impl<I, T> BaseVecIterator for DirtyRawVecValues<'_, I, T>
// where
//     I: StoredIndex,
//     T: StoredRaw,
// {
//     #[inline]
//     fn mut_index(&mut self) -> &mut usize {
//         &mut self.index
//     }

//     #[inline]
//     fn len(&self) -> usize {
//         self.vec.len()
//     }

//     #[inline]
//     fn name(&self) -> &str {
//         self.vec.name()
//     }
// }
