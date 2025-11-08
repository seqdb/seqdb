use memmap2::MmapMut;
use parking_lot::RwLockReadGuard;

use crate::RegionMetadata;

/// Zero-copy reader for accessing region data from memory-mapped storage.
///
/// Holds locks on the memory map and region metadata during its lifetime,
/// preventing concurrent modifications. Should be dropped as soon as reading
/// is complete to avoid blocking writes.
#[derive(Debug)]
pub struct Reader<'a> {
    mmap: RwLockReadGuard<'a, MmapMut>,
    region_meta: RwLockReadGuard<'a, RegionMetadata>,
}

impl<'a> Reader<'a> {
    #[inline]
    pub fn new(
        mmap: RwLockReadGuard<'a, MmapMut>,
        region_meta: RwLockReadGuard<'a, RegionMetadata>,
    ) -> Self {
        Self { mmap, region_meta }
    }

    #[inline(always)]
    pub fn unchecked_read(&self, offset: u64, len: u64) -> &[u8] {
        let start = self.region_meta.start() + offset;
        let end = start + len;
        &self.mmap[start as usize..end as usize]
    }

    #[inline(always)]
    pub fn read(&self, offset: u64, len: u64) -> &[u8] {
        assert!(offset + len <= self.region_meta.len());
        self.unchecked_read(offset, len)
    }

    #[inline(always)]
    pub fn read_all(&self) -> &[u8] {
        self.read(0, self.region_meta.len())
    }

    #[inline(always)]
    pub fn prefixed(&self, offset: u64) -> &[u8] {
        let start = self.region_meta.start() + offset;
        &self.mmap[start as usize..]
    }

    #[inline]
    pub fn region_meta(&self) -> &RegionMetadata {
        &self.region_meta
    }
}
