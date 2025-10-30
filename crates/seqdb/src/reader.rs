use memmap2::MmapMut;
use parking_lot::RwLockReadGuard;

use super::Region;

#[derive(Debug)]
pub struct Reader<'a> {
    mmap: RwLockReadGuard<'a, MmapMut>,
    region: RwLockReadGuard<'static, Region>,
}

impl<'a> Reader<'a> {
    #[inline]
    pub fn new(
        mmap: RwLockReadGuard<'a, MmapMut>,
        region: RwLockReadGuard<'static, Region>,
    ) -> Self {
        Self { mmap, region }
    }

    #[inline(always)]
    pub fn unchecked_read(&self, offset: u64, len: u64) -> &[u8] {
        let start = self.region.start() + offset;
        let end = start + len;
        &self.mmap[start as usize..end as usize]
    }

    #[inline(always)]
    pub fn read(&self, offset: u64, len: u64) -> &[u8] {
        assert!(offset + len <= self.region.len());
        self.unchecked_read(offset, len)
    }

    #[inline(always)]
    pub fn read_all(&self) -> &[u8] {
        self.read(0, self.region().len())
    }

    #[inline(always)]
    pub fn prefixed(&self, offset: u64) -> &[u8] {
        let start = self.region.start() + offset;
        &self.mmap[start as usize..]
    }

    #[inline]
    pub fn region(&self) -> &Region {
        &self.region
    }
}
