use parking_lot::RwLockReadGuard;
use std::{fs::File, os::unix::fs::FileExt};

use crate::uninit_vec;

use super::{Error, Region, Result};

#[cfg(target_os = "linux")]
use std::os::unix::io::AsRawFd;

#[cfg(target_os = "macos")]
use std::os::unix::io::AsRawFd;

// Linux constants
#[cfg(target_os = "linux")]
const POSIX_FADV_NORMAL: libc::c_int = 0;
#[cfg(target_os = "linux")]
const POSIX_FADV_SEQUENTIAL: libc::c_int = 2;
#[cfg(target_os = "linux")]
const POSIX_FADV_WILLNEED: libc::c_int = 3;

// macOS constants
#[cfg(target_os = "macos")]
const F_RDADVISE: libc::c_int = 44;

#[cfg(target_os = "macos")]
#[repr(C)]
struct radvisory {
    ra_offset: libc::off_t,
    ra_count: libc::c_int,
}

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

    #[inline]
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

    #[inline]
    pub fn prefixed(&self, offset: u64) -> Result<Vec<u8>> {
        let remaining_len = self.region.len() - offset;
        self.read(offset, remaining_len)
    }

    /// Advise the kernel that this region will be read sequentially for aggressive readahead
    #[inline]
    pub fn advise_sequential(&self) -> Result<()> {
        self.advise_sequential_range(0, self.region.len())
    }

    /// Advise the kernel about sequential reading of a specific range
    #[cfg(target_os = "linux")]
    pub fn advise_sequential_range(&self, offset: u64, len: u64) -> Result<()> {
        let fd = self.file.as_raw_fd();
        let start = self.region.start() + offset;

        unsafe {
            // Mark as sequential access
            libc::posix_fadvise(fd, start as i64, len as i64, POSIX_FADV_SEQUENTIAL);
            // Request aggressive readahead
            libc::posix_fadvise(fd, start as i64, len as i64, POSIX_FADV_WILLNEED);
        }

        Ok(())
    }

    /// Advise the kernel about sequential reading of a specific range
    #[cfg(target_os = "macos")]
    pub fn advise_sequential_range(&self, offset: u64, len: u64) -> Result<()> {
        let fd = self.file.as_raw_fd();
        let start = self.region.start() + offset;

        let advisory = radvisory {
            ra_offset: start as libc::off_t,
            ra_count: len as libc::c_int,
        };

        unsafe {
            libc::fcntl(fd, F_RDADVISE, &advisory);
        }

        Ok(())
    }

    /// Advise the kernel about sequential reading of a specific range
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    pub fn advise_sequential_range(&self, _offset: u64, _len: u64) -> Result<()> {
        // No-op on other platforms
        Ok(())
    }

    /// Revert advisory back to normal random access pattern
    #[inline]
    pub fn advise_normal(&self) -> Result<()> {
        self.advise_normal_range(0, self.region.len())
    }

    /// Revert advisory for a specific range back to normal
    #[cfg(target_os = "linux")]
    pub fn advise_normal_range(&self, offset: u64, len: u64) -> Result<()> {
        let fd = self.file.as_raw_fd();
        let start = self.region.start() + offset;

        unsafe {
            libc::posix_fadvise(fd, start as i64, len as i64, POSIX_FADV_NORMAL);
        }

        Ok(())
    }

    /// Revert advisory for a specific range back to normal
    #[cfg(target_os = "macos")]
    pub fn advise_normal_range(&self, _offset: u64, _len: u64) -> Result<()> {
        // F_RDADVISE on macOS is transient, no need to revert
        Ok(())
    }

    /// Revert advisory for a specific range back to normal
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    pub fn advise_normal_range(&self, _offset: u64, _len: u64) -> Result<()> {
        // No-op on other platforms
        Ok(())
    }
}
