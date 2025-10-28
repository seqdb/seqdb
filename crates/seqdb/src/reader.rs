use parking_lot::RwLockReadGuard;
use std::{fs::File, os::unix::fs::FileExt};

use crate::uninit_vec;

use super::{Error, Region, Result};

#[derive(Debug)]
pub struct Reader<'a> {
    file: RwLockReadGuard<'a, File>,
    region: RwLockReadGuard<'static, Region>,
}

impl<'a> Reader<'a> {
    #[inline]
    pub fn new(file: RwLockReadGuard<'a, File>, region: RwLockReadGuard<'static, Region>) -> Self {
        Self { file, region }
    }

    #[inline]
    pub fn read_into(&self, offset: u64, buffer: &mut [u8]) -> Result<()> {
        let len = buffer.len() as u64;
        let region_len = self.region.len();
        if offset + len > region_len {
            return Err(Error::String(format!(
                "Read beyond region bounds (buffer_len is {len} and region_len is {region_len})"
            )));
        }
        let start = self.region.start() + offset;

        self.file.read_exact_at(buffer, start)?;

        Ok(())
    }

    #[inline(always)]
    pub fn read(&self, offset: u64, len: u64) -> Result<Vec<u8>> {
        let mut buffer = uninit_vec(len as usize);
        self.read_into(offset, &mut buffer)?;

        Ok(buffer)
    }

    #[inline]
    pub fn read_all(&self) -> Result<Vec<u8>> {
        self.read(0, self.region().len())
    }

    #[inline]
    pub fn region(&self) -> &Region {
        &self.region
    }
}
