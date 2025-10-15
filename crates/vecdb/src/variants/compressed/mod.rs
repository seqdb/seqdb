use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
    mem,
    sync::Arc,
};

use allocative::Allocative;
use log::info;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use seqdb::{Database, Reader, Region};

use crate::{
    AnyCollectableVec, AnyIterableVec, AnyStoredVec, AnyVec, AsInnerSlice, BaseVecIterator,
    BoxedVecIterator, CollectableVec, Error, Format, FromInnerSlice, GenericStoredVec,
    HEADER_OFFSET, Header, RawVec, Result, StoredCompressed, StoredIndex, Version,
    variants::ImportOptions,
};

mod page;
mod pages;

use page::*;
use pages::*;

const ONE_KIB: usize = 1024;
const MAX_PAGE_SIZE: usize = 16 * ONE_KIB;
const PCO_COMPRESSION_LEVEL: usize = 4;

const VERSION: Version = Version::TWO;

#[derive(Debug, Allocative)]
pub struct CompressedVec<I, T> {
    inner: RawVec<I, T>,
    pages: Arc<RwLock<Pages>>,
}

impl<I, T> CompressedVec<I, T>
where
    I: StoredIndex,
    T: StoredCompressed,
{
    const PER_PAGE: usize = MAX_PAGE_SIZE / Self::SIZE_OF_T;

    /// Same as import but will reset the vec under certain errors, so be careful !
    pub fn forced_import(db: &Database, name: &str, version: Version) -> Result<Self> {
        Self::forced_import_with((db, name, version).into())
    }

    /// Same as import but will reset the vec under certain errors, so be careful !
    pub fn forced_import_with(mut options: ImportOptions) -> Result<Self> {
        options.version = options.version + VERSION;
        let res = Self::import_with(options);
        match res {
            Err(Error::DifferentCompressionMode)
            | Err(Error::WrongEndian)
            | Err(Error::WrongLength)
            | Err(Error::DifferentVersion { .. }) => {
                info!("Resetting {}...", options.name);
                let _ = options
                    .db
                    .remove_region(Self::vec_region_name_(options.name).into());
                let _ = options
                    .db
                    .remove_region(Self::holes_region_name_(options.name).into());
                let _ = options
                    .db
                    .remove_region(Self::pages_region_name_(options.name).into());
                Self::import_with(options)
            }
            _ => res,
        }
    }

    pub fn import(db: &Database, name: &str, version: Version) -> Result<Self> {
        Self::import_with((db, name, version).into())
    }

    pub fn import_with(options: ImportOptions) -> Result<Self> {
        let inner = RawVec::import_(options, Format::Compressed)?;

        let pages = Pages::import(options.db, &Self::pages_region_name_(options.name))?;

        let this = Self {
            inner,
            pages: Arc::new(RwLock::new(pages)),
        };

        *this.mut_stored_len() = this.real_stored_len();

        Ok(this)
    }

    fn decode_page(&self, page_index: usize, reader: &Reader) -> Result<Vec<T>> {
        Self::decode_page_(self.stored_len(), page_index, reader, &self.pages.read())
    }

    fn decode_page_(
        stored_len: usize,
        page_index: usize,
        reader: &Reader,
        pages: &Pages,
    ) -> Result<Vec<T>> {
        if Self::page_index_to_index(page_index) >= stored_len {
            return Err(Error::IndexTooHigh);
        } else if page_index >= pages.len() {
            return Err(Error::ExpectVecToHaveIndex);
        }

        let page = pages.get(page_index).unwrap();
        let len = page.bytes as u64;
        let offset = page.start;

        let slice = reader.read(offset, len);

        let vec: Vec<T::NumberType> = pco::standalone::simple_decompress(slice)?;
        let vec = T::from_inner_slice(vec);

        if vec.len() != page.values as usize {
            dbg!((offset, len, vec.len(), page.values));
            dbg!(vec);
            unreachable!()
        }

        Ok(vec)
    }

    fn compress_page(chunk: &[T]) -> Vec<u8> {
        if chunk.len() > Self::PER_PAGE {
            panic!();
        }

        pco::standalone::simpler_compress(chunk.as_inner_slice(), PCO_COMPRESSION_LEVEL).unwrap()
    }

    #[inline]
    fn index_to_page_index(index: usize) -> usize {
        index / Self::PER_PAGE
    }

    #[inline]
    fn page_index_to_index(page_index: usize) -> usize {
        page_index * Self::PER_PAGE
    }

    #[inline]
    pub fn iter(&self) -> CompressedVecIterator<'_, I, T> {
        self.into_iter()
    }

    #[inline]
    pub fn iter_at(&self, i: I) -> CompressedVecIterator<'_, I, T> {
        self.iter_at_(i.unwrap_to_usize())
    }

    #[inline]
    pub fn iter_at_(&self, i: usize) -> CompressedVecIterator<'_, I, T> {
        let mut iter = self.into_iter();
        iter.set_(i);
        iter
    }

    fn pages_region_name(&self) -> String {
        Self::pages_region_name_(self.name())
    }
    fn pages_region_name_(name: &str) -> String {
        format!("{}_pages", Self::vec_region_name_(name))
    }
}

impl<I, T> Clone for CompressedVec<I, T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            pages: self.pages.clone(),
        }
    }
}

impl<I, T> AnyVec for CompressedVec<I, T>
where
    I: StoredIndex,
    T: StoredCompressed,
{
    #[inline]
    fn version(&self) -> Version {
        self.inner.version()
    }

    #[inline]
    fn name(&self) -> &str {
        self.inner.name()
    }

    #[inline]
    fn len(&self) -> usize {
        self.len_()
    }

    #[inline]
    fn index_type_to_string(&self) -> &'static str {
        I::to_string()
    }

    #[inline]
    fn value_type_to_size_of(&self) -> usize {
        size_of::<T>()
    }

    #[inline]
    fn region_names(&self) -> Vec<String> {
        let mut v = self.inner.region_names();
        v.push(self.pages_region_name());
        v
    }
}

impl<I, T> AnyStoredVec for CompressedVec<I, T>
where
    I: StoredIndex,
    T: StoredCompressed,
{
    fn db(&self) -> &Database {
        self.inner.db()
    }

    #[inline]
    fn region(&self) -> &RwLock<Region> {
        self.inner.region()
    }

    #[inline]
    fn region_index(&self) -> usize {
        self.inner.region_index()
    }

    #[inline]
    fn header(&self) -> &Header {
        self.inner.header()
    }

    #[inline]
    fn mut_header(&mut self) -> &mut Header {
        self.inner.mut_header()
    }

    #[inline]
    fn saved_stamped_changes(&self) -> u16 {
        self.inner.saved_stamped_changes()
    }

    #[inline]
    fn stored_len(&self) -> usize {
        self.inner.stored_len()
    }

    #[inline]
    fn real_stored_len(&self) -> usize {
        self.pages.read().stored_len(Self::PER_PAGE)
    }

    fn flush(&mut self) -> Result<()> {
        self.inner.write_header_if_needed()?;

        let stored_len = self.stored_len();
        let pushed_len = self.pushed_len();
        let real_stored_len = self.real_stored_len();
        assert!(stored_len <= real_stored_len);
        let truncated = stored_len != real_stored_len;
        let has_new_data = pushed_len != 0;

        if !has_new_data && !truncated {
            // info!("Nothing to push {}", self.region_index());
            return Ok(());
        }

        let mut pages = self.pages.write();
        let pages_len = pages.len();
        let starting_page_index = Self::index_to_page_index(stored_len);
        assert!(starting_page_index <= pages_len);

        let mut values = vec![];

        let offset = HEADER_OFFSET as u64;

        let truncate_at = if starting_page_index < pages_len {
            let len = stored_len % Self::PER_PAGE;

            if len != 0 {
                let mut page_values = Self::decode_page_(
                    stored_len,
                    starting_page_index,
                    &self.create_static_reader(),
                    &pages,
                )?;
                page_values.truncate(len);
                values = page_values;
            }

            pages.truncate(starting_page_index).unwrap().start
        } else {
            pages
                .last()
                .map_or(offset, |page| page.start + page.bytes as u64)
        };

        values.append(&mut mem::take(self.inner.mut_pushed()));

        let compressed = values
            .chunks(Self::PER_PAGE)
            .map(|chunk| (Self::compress_page(chunk), chunk.len()))
            .collect::<Vec<_>>();

        compressed.iter().enumerate().for_each(|(i, (bytes, len))| {
            let page_index = starting_page_index + i;

            let start = if page_index != 0 {
                let prev = pages.get(page_index - 1).unwrap();
                prev.start + prev.bytes as u64
            } else {
                offset
            };

            let page = Page::new(start, bytes.len() as u32, *len as u32);

            pages.checked_push(page_index, page);
        });

        let buf = compressed
            .into_iter()
            .flat_map(|(v, _)| v)
            .collect::<Vec<_>>();

        let db = self.db();

        let mut mut_stored_len = self.mut_stored_len();

        db.truncate_write_all_to_region(self.region_index().into(), truncate_at, &buf)?;

        *mut_stored_len += pushed_len;

        pages.flush(db)?;

        Ok(())
    }

    fn serialize_changes(&self) -> Result<Vec<u8>> {
        self.inner.serialize_changes()
    }
}

impl<I, T> GenericStoredVec<I, T> for CompressedVec<I, T>
where
    I: StoredIndex,
    T: StoredCompressed,
{
    #[inline]
    fn read_(&self, index: usize, reader: &Reader) -> Result<T> {
        let page_index = Self::index_to_page_index(index);
        let decoded_index = index % Self::PER_PAGE;
        Ok(unsafe {
            *self
                .decode_page(page_index, reader)?
                .get_unchecked(decoded_index)
        })
    }

    #[inline]
    fn pushed(&self) -> &[T] {
        self.inner.pushed()
    }
    #[inline]
    fn mut_pushed(&mut self) -> &mut Vec<T> {
        self.inner.mut_pushed()
    }
    #[inline]
    fn prev_pushed(&self) -> &[T] {
        self.inner.prev_pushed()
    }
    #[inline]
    fn mut_prev_pushed(&mut self) -> &mut Vec<T> {
        self.inner.mut_prev_pushed()
    }
    #[inline]
    fn holes(&self) -> &BTreeSet<usize> {
        self.inner.holes()
    }
    #[inline]
    fn mut_holes(&mut self) -> &mut BTreeSet<usize> {
        panic!("unsupported for now")
    }
    #[inline]
    fn prev_holes(&self) -> &BTreeSet<usize> {
        self.inner.prev_holes()
    }
    #[inline]
    fn mut_prev_holes(&mut self) -> &mut BTreeSet<usize> {
        panic!("unsupported for now")
    }
    #[inline]
    fn updated(&self) -> &BTreeMap<usize, T> {
        self.inner.updated()
    }
    #[inline]
    fn mut_updated(&mut self) -> &mut BTreeMap<usize, T> {
        panic!("unsupported for now")
    }
    #[inline]
    fn prev_updated(&self) -> &BTreeMap<usize, T> {
        self.inner.prev_updated()
    }
    #[inline]
    fn mut_prev_updated(&mut self) -> &mut BTreeMap<usize, T> {
        panic!("unsupported for now")
    }

    #[inline]
    fn mut_stored_len(&'_ self) -> RwLockWriteGuard<'_, usize> {
        self.inner.mut_stored_len()
    }
    #[inline]
    fn prev_stored_len(&self) -> usize {
        self.inner.prev_stored_len()
    }
    #[inline]
    fn mut_prev_stored_len(&mut self) -> &mut usize {
        self.inner.mut_prev_stored_len()
    }

    fn reset(&mut self) -> Result<()> {
        self.pages.write().reset();
        self.reset_()
    }
}

#[derive(Debug)]
pub struct CompressedVecIterator<'a, I, T> {
    vec: &'a CompressedVec<I, T>,
    reader: Reader<'a>,
    decoded: Option<(usize, Vec<T>)>,
    pages: RwLockReadGuard<'a, Pages>,
    stored_len: usize,
    index: usize,
}

impl<I, T> CompressedVecIterator<'_, I, T>
where
    I: StoredIndex,
    T: StoredCompressed,
{
    const SIZE_OF_T: usize = size_of::<T>();
    const PER_PAGE: usize = MAX_PAGE_SIZE / Self::SIZE_OF_T;
}

impl<I, T> BaseVecIterator for CompressedVecIterator<'_, I, T>
where
    I: StoredIndex,
    T: StoredCompressed,
{
    #[inline]
    fn mut_index(&mut self) -> &mut usize {
        &mut self.index
    }

    #[inline]
    fn len(&self) -> usize {
        self.vec.len()
    }

    #[inline]
    fn name(&self) -> &str {
        self.vec.name()
    }
}

impl<'a, I, T> Iterator for CompressedVecIterator<'a, I, T>
where
    I: StoredIndex,
    T: StoredCompressed,
{
    type Item = (I, Cow<'a, T>);

    fn next(&mut self) -> Option<Self::Item> {
        let i = self.index;
        let stored_len = self.stored_len;

        let result = if i >= stored_len {
            let j = i - stored_len;
            if j >= self.vec.pushed_len() {
                return None;
            }
            self.vec
                .pushed()
                .get(j)
                .map(|v| (I::from(i), Cow::Borrowed(v)))
        } else {
            let page_index = i / Self::PER_PAGE;

            if self.decoded.as_ref().is_none_or(|b| b.0 != page_index) {
                let values = CompressedVec::<I, T>::decode_page_(
                    stored_len,
                    page_index,
                    &self.reader,
                    &self.pages,
                )
                .unwrap();
                self.decoded.replace((page_index, values));
            }

            self.decoded
                .as_ref()
                .unwrap()
                .1
                .get(i % Self::PER_PAGE)
                .map(|v| (I::from(i), Cow::Owned(*v)))
        };

        self.index += 1;

        result
    }
}

impl<'a, I, T> IntoIterator for &'a CompressedVec<I, T>
where
    I: StoredIndex,
    T: StoredCompressed,
{
    type Item = (I, Cow<'a, T>);
    type IntoIter = CompressedVecIterator<'a, I, T>;

    fn into_iter(self) -> Self::IntoIter {
        let pages = self.pages.read();
        let stored_len = self.stored_len();

        CompressedVecIterator {
            vec: self,
            reader: self.create_static_reader(),
            decoded: None,
            pages,
            index: 0,
            stored_len,
        }
    }
}

impl<I, T> AnyIterableVec<I, T> for CompressedVec<I, T>
where
    I: StoredIndex,
    T: StoredCompressed,
{
    fn boxed_iter<'a>(&'a self) -> BoxedVecIterator<'a, I, T>
    where
        T: 'a,
    {
        Box::new(self.into_iter())
    }
}

impl<I, T> AnyCollectableVec for CompressedVec<I, T>
where
    I: StoredIndex,
    T: StoredCompressed,
{
    fn collect_range_json_bytes(&self, from: Option<usize>, to: Option<usize>) -> Vec<u8> {
        CollectableVec::collect_range_json_bytes(self, from, to)
    }

    fn collect_range_string(&self, from: Option<usize>, to: Option<usize>) -> Vec<String> {
        CollectableVec::collect_range_string(self, from, to)
    }
}
