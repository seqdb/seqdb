use std::{fs::File, mem, ops::Deref, sync::Arc};

use allocative::Allocative;
use memmap2::MmapMut;
use parking_lot::{RwLock, RwLockReadGuard};

use crate::{Database, Error, Result, WeakDatabase};

use super::{PAGE_SIZE, Reader};

#[derive(Debug, Clone, Allocative)]
pub struct Region(Arc<RegionInner>);

#[derive(Debug, Allocative)]
pub struct RegionInner {
    #[allocative(skip)]
    db: WeakDatabase,
    pub(crate) index: usize,
    pub(crate) meta: RwLock<RegionMetadata>,
}

#[derive(Debug, Clone, Allocative)]
pub struct RegionMetadata {
    /// Must be multiple of 4096
    start: u64,
    len: u64,
    /// Must be multiple of 4096, greater or equal to len
    reserved: u64,
    padding: u64,
}

pub const SIZE_OF_REGION_METADATA: usize = 32; // 4 * u64 = 32 bytes

impl Region {
    pub fn new(db: &Database, index: usize, start: u64, len: u64, reserved: u64) -> Self {
        assert!(start.is_multiple_of(PAGE_SIZE));
        assert!(reserved >= PAGE_SIZE);
        assert!(reserved.is_multiple_of(PAGE_SIZE));
        assert!(len <= reserved);

        Self(Arc::new(RegionInner {
            db: db.weak_clone(),
            index,
            meta: RwLock::new(RegionMetadata {
                start,
                len,
                reserved,
                padding: 0,
            }),
        }))
    }

    pub fn from(db: &Database, index: usize, meta: RegionMetadata) -> Self {
        Self(Arc::new(RegionInner {
            db: db.weak_clone(),
            index,
            meta: RwLock::new(meta),
        }))
    }

    #[inline]
    pub fn index(&self) -> usize {
        self.index
    }

    #[inline]
    pub fn meta(&self) -> &RwLock<RegionMetadata> {
        &self.meta
    }

    #[inline]
    pub fn db(&self) -> Database {
        self.db.upgrade()
    }

    pub fn create_reader(&self) -> Reader<'static> {
        let db = self.db();
        let mmap: RwLockReadGuard<'static, MmapMut> = unsafe { mem::transmute(db.mmap.read()) };
        let region_meta: RwLockReadGuard<'static, RegionMetadata> =
            unsafe { mem::transmute(self.meta.read()) };
        Reader::new(mmap, region_meta)
    }

    pub fn open_db_read_only_file(&self) -> Result<File> {
        self.db().open_read_only_file()
    }

    pub fn write_all_at(&self, data: &[u8], at: u64) -> Result<()> {
        self.db().write_all_to_region_at(self, data, at)
    }

    pub fn truncate(&self, from: u64) -> Result<()> {
        self.db().truncate_region(self, from)
    }

    pub fn truncate_write_all(&self, from: u64, data: &[u8]) -> Result<()> {
        self.db().truncate_write_all_to_region(self, from, data)
    }
}

impl RegionMetadata {
    #[inline]
    pub fn start(&self) -> u64 {
        self.start
    }

    #[inline]
    pub fn set_start(&mut self, start: u64) {
        assert!(start.is_multiple_of(PAGE_SIZE));
        self.start = start
    }

    #[allow(clippy::len_without_is_empty)]
    #[inline]
    pub fn len(&self) -> u64 {
        self.len
    }

    #[inline]
    pub fn set_len(&mut self, len: u64) {
        assert!(len <= self.reserved());
        self.len = len
    }

    #[inline]
    pub fn reserved(&self) -> u64 {
        self.reserved
    }

    pub fn set_reserved(&mut self, reserved: u64) {
        assert!(self.len() <= reserved);
        assert!(reserved >= PAGE_SIZE);
        assert!(reserved.is_multiple_of(PAGE_SIZE));

        self.reserved = reserved;
    }

    #[inline]
    pub fn left(&self) -> u64 {
        self.reserved - self.len
    }

    /// Serialize to bytes using little endian encoding
    pub fn to_bytes(&self) -> [u8; SIZE_OF_REGION_METADATA] {
        let mut bytes = [0u8; SIZE_OF_REGION_METADATA];
        bytes[0..8].copy_from_slice(&self.start.to_le_bytes());
        bytes[8..16].copy_from_slice(&self.len.to_le_bytes());
        bytes[16..24].copy_from_slice(&self.reserved.to_le_bytes());
        bytes[24..32].copy_from_slice(&self.padding.to_le_bytes());
        bytes
    }

    /// Deserialize from bytes using little endian encoding
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < SIZE_OF_REGION_METADATA {
            return Err(Error::Str("Buffer too small for RegionMetadata"));
        }

        let start = u64::from_le_bytes(bytes[0..8].try_into().unwrap());
        let len = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
        let reserved = u64::from_le_bytes(bytes[16..24].try_into().unwrap());
        let padding = u64::from_le_bytes(bytes[24..32].try_into().unwrap());

        Ok(Self {
            start,
            len,
            reserved,
            padding,
        })
    }
}

impl Deref for Region {
    type Target = Arc<RegionInner>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
