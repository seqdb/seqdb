use memmap2::{Advice, MmapMut};
use parking_lot::RwLockReadGuard;

use super::Region;

#[derive(Debug)]
pub struct Reader<'a> {
    mmap: RwLockReadGuard<'a, MmapMut>,
    region: RwLockReadGuard<'static, Region>,
}

impl<'a> Reader<'a> {
    pub fn new(
        mmap: RwLockReadGuard<'a, MmapMut>,
        region: RwLockReadGuard<'static, Region>,
    ) -> Self {
        Self { mmap, region }
    }

    pub fn with_advice(self, advice: Advice) -> Self {
        self.advice(advice);
        self
    }

    pub fn with_random_advice(self) -> Self {
        self.advice(Advice::Random);
        self
    }

    pub fn with_seq_advice(self) -> Self {
        self.advice(Advice::Sequential);
        self
    }

    pub fn advice(&self, advice: Advice) {
        let offset = self.region().start() as usize;
        let len = self.region().len() as usize;
        let _ = self.mmap.advise_range(advice, offset, len);
    }

    pub fn read(&self, offset: u64, len: u64) -> &[u8] {
        assert!(offset + len <= self.region.len());
        let start = self.region.start() + offset;
        let end = start + len;
        &self.mmap[start as usize..end as usize]
    }

    pub fn read_all(&self) -> &[u8] {
        self.read(0, self.region().len())
    }

    pub fn region(&self) -> &Region {
        &self.region
    }

    pub fn prefixed(&self, offset: u64) -> &[u8] {
        let start = self.region.start() + offset;
        &self.mmap[start as usize..]
    }
}
