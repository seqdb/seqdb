use std::fs::File;

use allocative::Allocative;
use parking_lot::RwLockReadGuard;
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

use crate::Result;

use super::{DatabaseInner, PAGE_SIZE, Reader};

#[derive(Debug, Clone, FromBytes, IntoBytes, Immutable, KnownLayout, Allocative)]
#[repr(C)]
pub struct Region {
    /// Must be multiple of 4096
    start: u64,
    len: u64,
    /// Must be multiple of 4096, greater or equal to len
    reserved: u64,
    padding: u64,
}

pub const SIZE_OF_REGION: usize = size_of::<Region>();

impl Region {
    pub fn new(start: u64, len: u64, reserved: u64) -> Self {
        assert!(start.is_multiple_of(PAGE_SIZE));
        assert!(reserved >= PAGE_SIZE);
        assert!(reserved.is_multiple_of(PAGE_SIZE));
        assert!(len <= reserved);

        Self {
            start,
            len,
            reserved,
            padding: 0,
        }
    }

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
}

pub trait RegionReader {
    fn create_reader(self, seqdb: &'_ DatabaseInner) -> Result<Reader<'_>>;
}

impl<'a> RegionReader for RwLockReadGuard<'a, Region> {
    fn create_reader(self, db: &DatabaseInner) -> Result<Reader<'static>> {
        let region: RwLockReadGuard<'static, Region> = unsafe { std::mem::transmute(self) };
        let _lock: RwLockReadGuard<'static, File> = unsafe { std::mem::transmute(db.file.read()) };
        Ok(Reader::new(db.open_read_only_file()?, region, _lock))
    }
}
