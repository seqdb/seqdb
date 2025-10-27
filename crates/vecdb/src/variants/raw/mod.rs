use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::{Read, Seek, SeekFrom},
    marker::PhantomData,
    mem,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use allocative::Allocative;
use log::info;
use parking_lot::RwLock;
use seqdb::{Database, Reader, Region, RegionReader};
use zerocopy::{FromBytes, IntoBytes};

use crate::{
    AnyCollectableVec, AnyIterableVec, AnyStoredVec, AnyVec, BaseVecIterator, BoxedVecIterator,
    CollectableVec, Error, GenericStoredVec, Result, StoredIndex, StoredRaw, VEC_PAGE_SIZE,
    Version, likely, unlikely,
};

use super::Format;

mod header;
mod options;

pub use header::*;
pub use options::*;

const VERSION: Version = Version::ONE;

#[derive(Debug, Allocative)]
pub struct RawVec<I, T> {
    db: Database,
    region: Arc<RwLock<Region>>,
    region_index: usize,

    header: Header,
    name: &'static str,
    prev_pushed: Vec<T>,
    pushed: Vec<T>,
    has_stored_holes: bool,
    holes: BTreeSet<usize>,
    prev_holes: BTreeSet<usize>,
    updated: BTreeMap<usize, T>,
    prev_updated: BTreeMap<usize, T>,
    prev_stored_len: usize,
    stored_len: Arc<AtomicUsize>,
    /// Default is 0
    saved_stamped_changes: u16,

    phantom: PhantomData<I>,
}

impl<I, T> RawVec<I, T>
where
    I: StoredIndex,
    T: StoredRaw,
{
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
                Self::import_with(options)
            }
            _ => res,
        }
    }

    pub fn import(db: &Database, name: &str, version: Version) -> Result<Self> {
        Self::import_with((db, name, version).into())
    }

    pub fn import_with(options: ImportOptions) -> Result<Self> {
        Self::import_(options, Format::Raw)
    }

    #[doc(hidden)]
    pub fn import_(
        ImportOptions {
            db,
            name,
            version,
            saved_stamped_changes,
        }: ImportOptions,
        format: Format,
    ) -> Result<Self> {
        let (region_index, region) = db.create_region_if_needed(&Self::vec_region_name_(name))?;

        let region_len = region.read().len() as usize;
        if region_len > 0
            && (region_len < HEADER_OFFSET
                || (format.is_raw()
                    && !(region_len - HEADER_OFFSET).is_multiple_of(Self::SIZE_OF_T)))
        {
            dbg!(region_len, region_len, HEADER_OFFSET);
            return Err(Error::Str("Region was saved incorrectly"));
        }

        let header = if region_len == 0 {
            Header::create_and_write(db, region_index, version, format)?
        } else {
            Header::import_and_verify(db, region_index, region.read().len(), version, format)?
        };

        let holes = if let Ok(holes) = db.get_region(Self::holes_region_name_(name).into()) {
            Some(
                holes
                    .create_reader(db)
                    .read_all()?
                    .chunks(size_of::<usize>())
                    .map(|b| -> Result<usize> { usize::read_from_bytes(b).map_err(|e| e.into()) })
                    .collect::<Result<BTreeSet<usize>>>()?,
            )
        } else {
            None
        };

        let mut this = Self {
            db: db.clone(),
            region: region.clone(),
            region_index,
            header,
            name: Box::leak(Box::new(name.to_string())),
            prev_pushed: vec![],
            pushed: vec![],
            has_stored_holes: holes.is_some(),
            holes: holes.clone().unwrap_or_default(),
            prev_holes: holes.unwrap_or_default(),
            updated: BTreeMap::new(),
            prev_updated: BTreeMap::new(),
            phantom: PhantomData,
            prev_stored_len: 0,
            stored_len: Arc::new(AtomicUsize::new(0)),
            saved_stamped_changes,
        };

        let len = this.real_stored_len();
        *this.mut_prev_stored_len() = len;
        this.update_stored_len(len);

        Ok(this)
    }

    #[inline]
    pub fn iter(&self) -> RawVecIterator<'_, I, T> {
        self.into_iter()
    }

    #[inline]
    pub fn iter_at(&self, i: I) -> RawVecIterator<'_, I, T> {
        self.iter_at_(i.to_usize())
    }

    #[inline]
    pub fn iter_at_(&self, i: usize) -> RawVecIterator<'_, I, T> {
        let mut iter = self.into_iter();
        iter.set_(i);
        iter
    }

    /// Iterator over values only (no indices) for maximum performance
    #[inline]
    pub fn values(&self) -> RawVecValues<'_, I, T> {
        RawVecValues(self.into_iter())
    }

    /// Streaming iterator for maximum throughput (no indices, no holes/updates support)
    #[inline]
    pub fn stream(&self) -> RawStreamIterator<'_, I, T> {
        self.stream_at_(0)
    }

    /// Streaming iterator starting at index
    #[inline]
    pub fn stream_at(&self, i: I) -> RawStreamIterator<'_, I, T> {
        self.stream_at_(i.to_usize())
    }

    /// Streaming iterator starting at index
    #[inline]
    pub fn stream_at_(&self, i: usize) -> RawStreamIterator<'_, I, T> {
        let _reader = self.create_static_reader();
        let region_start = self.region.read().start();

        let mut seq_file = self
            .db
            .open_sequential_reader()
            .expect("Failed to open sequential reader");

        // Use stored_len which is the actual length including what's on disk
        // The regular iterator handles pushed/holes/updates separately
        let stored_len = self.stored_len();
        let start_index = i.min(stored_len);
        let start_offset =
            region_start + HEADER_OFFSET as u64 + (start_index * size_of::<T>()) as u64;
        let total_bytes = (stored_len * size_of::<T>()) as u64;

        // Seek to starting position
        seq_file
            .seek(SeekFrom::Start(start_offset))
            .expect("Failed to seek to start position");

        // Round buffer size down to multiple of SIZE_OF_T to ensure no partial values
        let size_of_t = size_of::<T>();
        let buffer_size = (VEC_PAGE_SIZE / size_of_t) * size_of_t;

        RawStreamIterator {
            seq_file,
            buffer: vec![0; buffer_size],
            buffer_pos: 0,
            buffer_len: 0,
            file_offset: start_offset,
            end_offset: region_start + HEADER_OFFSET as u64 + total_bytes,
            _reader,
            _marker: PhantomData,
        }
    }

    pub fn write_header_if_needed(&mut self) -> Result<()> {
        if self.header.modified() {
            self.header.write(&self.db, self.region_index)?;
        }
        Ok(())
    }

    pub fn prev_holes(&self) -> &BTreeSet<usize> {
        &self.prev_holes
    }
}

impl<I, T> Clone for RawVec<I, T> {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            region: self.region.clone(),
            region_index: self.region_index,
            header: self.header.clone(),
            name: self.name,
            prev_pushed: vec![],
            pushed: vec![],
            updated: BTreeMap::new(),
            prev_updated: BTreeMap::new(),
            has_stored_holes: false,
            holes: BTreeSet::new(),
            prev_holes: BTreeSet::new(),
            prev_stored_len: 0,
            stored_len: self.stored_len.clone(),
            saved_stamped_changes: self.saved_stamped_changes,
            phantom: PhantomData,
        }
    }
}

impl<I, T> AnyVec for RawVec<I, T>
where
    I: StoredIndex,
    T: StoredRaw,
{
    #[inline]
    fn version(&self) -> Version {
        self.header.vec_version()
    }

    #[inline]
    fn name(&self) -> &str {
        self.name
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
        vec![self.index_to_name()]
    }
}

impl<I, T> AnyStoredVec for RawVec<I, T>
where
    I: StoredIndex,
    T: StoredRaw,
{
    #[inline]
    fn header(&self) -> &Header {
        &self.header
    }

    #[inline]
    fn mut_header(&mut self) -> &mut Header {
        &mut self.header
    }

    #[inline]
    fn saved_stamped_changes(&self) -> u16 {
        self.saved_stamped_changes
    }

    #[inline]
    fn real_stored_len(&self) -> usize {
        (self.region.read().len() as usize - HEADER_OFFSET) / Self::SIZE_OF_T
    }

    #[inline]
    fn stored_len(&self) -> usize {
        self.stored_len.load(Ordering::Acquire)
    }

    fn flush(&mut self) -> Result<()> {
        self.write_header_if_needed()?;

        let stored_len = self.stored_len();
        let pushed_len = self.pushed_len();
        let real_stored_len = self.real_stored_len();
        assert!(stored_len <= real_stored_len);
        let truncated = stored_len != real_stored_len;
        let has_new_data = pushed_len != 0;
        let has_prev_updated_data = !self.prev_updated.is_empty();
        let has_updated_data = !self.updated.is_empty();
        let has_holes = !self.holes.is_empty();
        let had_holes = !self.prev_holes.is_empty() || self.has_stored_holes;

        if !truncated
            && !has_new_data
            && !has_prev_updated_data
            && !has_updated_data
            && !has_holes
            && !had_holes
        {
            return Ok(());
        }

        let from = (stored_len * Self::SIZE_OF_T + HEADER_OFFSET) as u64;

        // self.prev_stored_len = stored_len;

        // BE CAREFUL WITH `.clone()` !
        // Take into account the first pass which hold a ton of data
        // self.prev_pushed = self.pushed.clone();

        if has_new_data {
            self.db.truncate_write_all_to_region(
                self.region_index.into(),
                from,
                mem::take(&mut self.pushed).as_bytes(),
            )?;
            self.update_stored_len(stored_len + pushed_len);
        } else if truncated {
            self.db.truncate_region(self.region_index.into(), from)?;
        }

        if has_updated_data || has_prev_updated_data {
            let mut updated = mem::take(&mut self.updated);
            updated.append(&mut mem::take(&mut self.prev_updated));
            updated.into_iter().try_for_each(|(i, v)| -> Result<()> {
                let bytes = v.as_bytes();
                let at = ((i * Self::SIZE_OF_T) + HEADER_OFFSET) as u64;
                self.db
                    .write_all_to_region_at(self.region_index.into(), bytes, at)?;
                Ok(())
            })?;
        }

        if has_holes {
            self.has_stored_holes = true;
            let (holes_index, _) = self.db.create_region_if_needed(&self.holes_region_name())?;
            let bytes = self
                .holes
                .iter()
                .flat_map(|i| i.to_ne_bytes())
                .collect::<Vec<_>>();
            self.db
                .truncate_write_all_to_region(holes_index.into(), 0, &bytes)?;
        } else if had_holes {
            self.has_stored_holes = false;
            let _ = self.db.remove_region(self.holes_region_name().into());
        }

        // self.prev_holes = self.holes.clone();

        Ok(())
    }

    fn db(&self) -> &Database {
        &self.db
    }

    fn region_index(&self) -> usize {
        self.region_index
    }

    fn region(&self) -> &RwLock<Region> {
        &self.region
    }

    fn serialize_changes(&self) -> Result<Vec<u8>> {
        let mut bytes = vec![];
        let reader = self.create_reader();

        bytes.extend(self.stamp().as_bytes());

        // let real_stored_len = self.real_stored_len();
        let prev_stored_len = self.prev_stored_len();
        let stored_len = self.stored_len();

        bytes.extend(prev_stored_len.as_bytes());
        bytes.extend(stored_len.as_bytes());

        let truncated = prev_stored_len.checked_sub(stored_len).unwrap_or_default();
        bytes.extend(truncated.as_bytes());
        if truncated > 0 {
            // dbg!((
            //     "trunc",
            //     stored_len,
            //     prev_stored_len,
            //     (stored_len..prev_stored_len)
            //         .map(|i| self.unwrap_read_(i, &reader))
            //         .collect::<Vec<_>>()
            // ));
            bytes.extend(
                (stored_len..prev_stored_len)
                    .map(|i| self.unwrap_read_(i, &reader))
                    .collect::<Vec<_>>()
                    .as_bytes(),
            );
        }

        bytes.extend(self.prev_pushed.len().as_bytes());
        bytes.extend(self.prev_pushed.iter().flat_map(|v| v.as_bytes()));

        bytes.extend(self.pushed.len().as_bytes());
        bytes.extend(self.pushed.iter().flat_map(|v| v.as_bytes()));

        let (prev_modified_indexes, prev_modified_values) = self
            .prev_updated
            .iter()
            .map(|(&i, v)| (i, v.clone()))
            .collect::<(Vec<_>, Vec<_>)>();
        bytes.extend(prev_modified_indexes.len().as_bytes());
        bytes.extend(prev_modified_indexes.as_bytes());
        bytes.extend(prev_modified_values.as_bytes());

        let (modified_indexes, modified_values) = self
            .updated
            .keys()
            .map(|&i| (i, self.unwrap_read_(i, &reader)))
            .collect::<(Vec<_>, Vec<_>)>();
        bytes.extend(modified_indexes.len().as_bytes());
        bytes.extend(modified_indexes.as_bytes());
        bytes.extend(modified_values.as_bytes());

        let prev_holes = self.prev_holes.iter().copied().collect::<Vec<_>>();
        bytes.extend(prev_holes.len().as_bytes());
        bytes.extend(prev_holes.as_bytes());

        let holes = self.holes.iter().copied().collect::<Vec<_>>();
        bytes.extend(holes.len().as_bytes());
        bytes.extend(holes.as_bytes());

        Ok(bytes)
    }
}

impl<I, T> GenericStoredVec<I, T> for RawVec<I, T>
where
    I: StoredIndex,
    T: StoredRaw,
{
    #[inline]
    fn read_(&self, index: usize, reader: &Reader) -> Result<T> {
        T::read_from_prefix(&reader.prefixed((index * Self::SIZE_OF_T + HEADER_OFFSET) as u64)?)
            .map(|(v, _)| v)
            .map_err(Error::from)
    }

    #[inline]
    fn pushed(&self) -> &[T] {
        self.pushed.as_slice()
    }
    #[inline]
    fn mut_pushed(&mut self) -> &mut Vec<T> {
        &mut self.pushed
    }
    #[inline]
    fn prev_pushed(&self) -> &[T] {
        self.prev_pushed.as_slice()
    }
    #[inline]
    fn mut_prev_pushed(&mut self) -> &mut Vec<T> {
        &mut self.prev_pushed
    }

    #[inline]
    fn holes(&self) -> &BTreeSet<usize> {
        &self.holes
    }
    #[inline]
    fn mut_holes(&mut self) -> &mut BTreeSet<usize> {
        &mut self.holes
    }
    #[inline]
    fn prev_holes(&self) -> &BTreeSet<usize> {
        &self.prev_holes
    }
    #[inline]
    fn mut_prev_holes(&mut self) -> &mut BTreeSet<usize> {
        &mut self.prev_holes
    }

    fn prev_stored_len(&self) -> usize {
        self.prev_stored_len
    }
    fn mut_prev_stored_len(&mut self) -> &mut usize {
        &mut self.prev_stored_len
    }
    fn update_stored_len(&self, val: usize) {
        self.stored_len.store(val, Ordering::Release);
    }

    #[inline]
    fn updated(&self) -> &BTreeMap<usize, T> {
        &self.updated
    }
    #[inline]
    fn mut_updated(&mut self) -> &mut BTreeMap<usize, T> {
        &mut self.updated
    }
    #[inline]
    fn prev_updated(&self) -> &BTreeMap<usize, T> {
        &self.prev_updated
    }
    #[inline]
    fn mut_prev_updated(&mut self) -> &mut BTreeMap<usize, T> {
        &mut self.prev_updated
    }

    fn reset(&mut self) -> Result<()> {
        self.reset_()
    }
}

pub struct RawVecIterator<'a, I, T> {
    // HOT: accessed every iteration (first cache line)
    index: usize,
    stored_len: usize,
    current_page: usize,
    cursor_page: usize,
    dirty: bool,
    holes: bool,
    updated: bool,

    // WARM: accessed per page
    seq_file: File,
    region_start: u64,
    buffer: Vec<u8>,

    // COLD: only accessed in slow path or for lifetimes
    vec: &'a RawVec<I, T>,
    _reader: Reader<'a>, // Holds locks, must stay alive
}

impl<I, T> RawVecIterator<'_, I, T>
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

impl<I, T> BaseVecIterator for RawVecIterator<'_, I, T>
where
    I: StoredIndex,
    T: StoredRaw,
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

impl<'a, I, T> RawVecIterator<'a, I, T>
where
    I: StoredIndex,
    T: StoredRaw,
{
    /// Load page if needed and read value at index from stored data
    #[inline(always)]
    fn read_stored(&mut self, index: usize) -> T {
        let page_index = Self::index_to_page(index);
        let buffer_index = Self::mul_size(Self::index_in_page(index));

        if unlikely(page_index != self.current_page) {
            let remaining = self.stored_len - index;
            let new_len = Self::mul_size(remaining.min(Self::PER_PAGE));

            if unlikely(self.cursor_page != page_index) {
                let offset = self.region_start + Self::mul_size(page_index * Self::PER_PAGE) as u64;
                // Safety: seek position is always valid (within file bounds)
                unsafe {
                    self.seq_file
                        .seek(SeekFrom::Start(offset))
                        .unwrap_unchecked()
                };
            }

            // Read into buffer
            // Safety: buffer is correctly sized and file has enough data
            unsafe {
                self.seq_file
                    .read_exact(&mut self.buffer[..new_len])
                    .unwrap_unchecked()
            };
            self.current_page = page_index;
            self.cursor_page = page_index + 1;
        }

        // Safety: buffer_index is guaranteed to be in bounds by page logic
        // and properly aligned since buffer_index is always a multiple of SIZE_OF_T
        // and Vec allocator provides proper alignment
        unsafe { std::ptr::read_unaligned(self.buffer.as_ptr().add(buffer_index) as *const T) }
    }

    /// Core iteration logic - returns just the value
    #[inline(always)]
    pub fn next_value(&mut self) -> Option<T> {
        let index = self.index;
        self.index += 1;

        // Fast path: clean vec (no dirty data)
        if likely(!self.dirty) {
            if unlikely(index >= self.stored_len) {
                return None;
            }

            return Some(self.read_stored(index));
        }

        // Slow path: handle dirty data (holes/updates/pushed items)
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

        Some(self.read_stored(index))
    }
}

/// Iterator adapter that yields only values (no indices)
pub struct RawVecValues<'a, I, T>(RawVecIterator<'a, I, T>);

impl<I, T> Iterator for RawVecValues<'_, I, T>
where
    I: StoredIndex,
    T: StoredRaw,
{
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next_value()
    }
}

/// Streaming iterator for maximum throughput - no state tracking, just raw sequential reads
pub struct RawStreamIterator<'a, I, T> {
    seq_file: File,
    buffer: Vec<u8>,
    buffer_pos: usize,
    buffer_len: usize,
    file_offset: u64,
    end_offset: u64,
    _reader: Reader<'a>,
    _marker: PhantomData<(I, T)>,
}

impl<I, T> RawStreamIterator<'_, I, T>
where
    I: StoredIndex,
    T: StoredRaw,
{
    const SIZE_OF_T: usize = size_of::<T>();
}

impl<I, T> Iterator for RawStreamIterator<'_, I, T>
where
    I: StoredIndex,
    T: StoredRaw,
{
    type Item = T;

    #[inline(always)]
    fn next(&mut self) -> Option<T> {
        // Fast path: read from current buffer
        if likely(self.buffer_pos + Self::SIZE_OF_T <= self.buffer_len) {
            self.buffer_pos += Self::SIZE_OF_T;
            return Some(unsafe {
                std::ptr::read_unaligned(self.buffer.as_ptr().add(self.buffer_pos) as *const T)
            });
        }

        // Slow path: refill buffer and read
        if unlikely(self.file_offset >= self.end_offset) {
            return None;
        }

        // Read fresh buffer
        let remaining = (self.end_offset - self.file_offset) as usize;
        let to_read = remaining.min(self.buffer.len());

        // Safety: we're within file bounds, read should succeed
        unsafe {
            self.seq_file
                .read_exact(&mut self.buffer[..to_read])
                .unwrap_unchecked()
        };

        self.file_offset += to_read as u64;
        self.buffer_len = to_read;
        self.buffer_pos = Self::SIZE_OF_T;

        Some(unsafe { std::ptr::read_unaligned(self.buffer.as_ptr() as *const T) })
    }
}

impl<'a, I, T> Iterator for RawVecIterator<'a, I, T>
where
    I: StoredIndex,
    T: StoredRaw,
{
    type Item = (I, T);

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        // Get current index before increment happens
        let index = self.index;
        let value = self.next_value()?;
        Some((I::from(index), value))
    }
}

impl<'a, I, T> IntoIterator for &'a RawVec<I, T>
where
    I: StoredIndex,
    T: StoredRaw,
{
    type Item = (I, T);
    type IntoIter = RawVecIterator<'a, I, T>;

    fn into_iter(self) -> Self::IntoIter {
        let pushed = !self.pushed.is_empty();
        let holes = !self.holes.is_empty();
        let updated = !self.updated.is_empty();
        let stored_len = self.stored_len();

        let _reader = self.create_static_reader();
        let region_start = self.region.read().start() + HEADER_OFFSET as u64;

        // Open dedicated file handle for optimal sequential readahead
        let seq_file = self
            .db
            .open_sequential_reader()
            .expect("Failed to open sequential reader");

        // Round buffer size down to multiple of SIZE_OF_T to ensure no partial values
        let size_of_t = size_of::<T>();
        let buffer_size = (VEC_PAGE_SIZE / size_of_t) * size_of_t;

        RawVecIterator {
            // HOT fields first
            index: 0,
            stored_len,
            current_page: usize::MAX,
            cursor_page: usize::MAX,
            dirty: pushed || holes || updated,
            holes,
            updated,

            // WARM fields
            seq_file,
            region_start,
            buffer: vec![0; buffer_size],

            // COLD fields
            vec: self,
            _reader,
        }
    }
}

impl<I, T> AnyIterableVec<I, T> for RawVec<I, T>
where
    I: StoredIndex,
    T: StoredRaw,
{
    fn boxed_iter(&self) -> BoxedVecIterator<'_, I, T> {
        Box::new(self.into_iter())
    }
}

impl<I, T> AnyCollectableVec for RawVec<I, T>
where
    I: StoredIndex,
    T: StoredRaw,
{
    fn collect_range_json_bytes(&self, from: Option<usize>, to: Option<usize>) -> Vec<u8> {
        CollectableVec::collect_range_json_bytes(self, from, to)
    }

    fn collect_range_string(&self, from: Option<usize>, to: Option<usize>) -> Vec<String> {
        CollectableVec::collect_range_string(self, from, to)
    }
}
