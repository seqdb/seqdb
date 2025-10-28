use parking_lot::RwLockReadGuard;
use seqdb::Reader;

use crate::{
    AnyStoredVec, CompressedVec, GenericStoredVec, Result, StoredCompressed, StoredIndex, likely,
};

use super::super::pages::Pages;

pub struct CleanCompressedVecIterator<'a, I, T> {
    pub(crate) index: usize,
    pub(crate) values: CleanCompressedVecValues<'a, I, T>,
}

impl<'a, I, T> CleanCompressedVecIterator<'a, I, T>
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
            values: CleanCompressedVecValues::new_at(vec, index)?,
        })
    }
}

impl<I, T> Iterator for CleanCompressedVecIterator<'_, I, T>
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

pub struct CleanCompressedVecValues<'a, I, T> {
    pub(crate) _vec: &'a CompressedVec<I, T>,
    reader: Reader<'a>,
    pub(crate) decoded: Option<(usize, Vec<T>)>,
    pub(crate) decoded_pos: usize,
    pages: RwLockReadGuard<'a, Pages>,
    pub(crate) stored_len: usize,
    pub(crate) index: usize,
}

impl<'a, I, T> CleanCompressedVecValues<'a, I, T>
where
    I: StoredIndex,
    T: StoredCompressed,
{
    const SIZE_OF_T: usize = size_of::<T>();
    const PER_PAGE: usize = crate::variants::compressed::MAX_COMPRESSED_PAGE_SIZE / Self::SIZE_OF_T;

    pub fn new(vec: &'a CompressedVec<I, T>) -> Result<Self> {
        Self::new_at(vec, 0)
    }

    pub fn new_at(vec: &'a CompressedVec<I, T>, index: usize) -> Result<Self> {
        let pages = vec.pages.read();
        let stored_len = vec.stored_len();

        Ok(Self {
            _vec: vec,
            reader: vec.create_reader(),
            decoded: None,
            decoded_pos: 0,
            pages,
            stored_len,
            index,
        })
    }
}

impl<I, T> Iterator for CleanCompressedVecValues<'_, I, T>
where
    I: StoredIndex,
    T: StoredCompressed,
{
    type Item = T;

    #[inline(always)]
    fn next(&mut self) -> Option<T> {
        let index = self.index;
        self.index += 1;

        let stored_len = self.stored_len;

        // Check if we're reading pushed values
        if index >= stored_len {
            return self._vec.get_pushed(index, stored_len).copied();
        }

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
        let values =
            CompressedVec::<I, T>::decode_page_(stored_len, page_index, &self.reader, &self.pages)
                .ok()?;

        let value = values.get(in_page_index).copied();
        self.decoded.replace((page_index, values));

        value
    }
}
