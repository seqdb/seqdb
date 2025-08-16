#![doc = include_str!(concat!("../", env!("CARGO_PKG_README")))]
#![doc = "\n## Example\n"]
#![doc = "\n```rust"]
#![doc = include_str!("../examples/db.rs")]
#![doc = "```\n"]

use std::{
    fs::{self, File, OpenOptions},
    ops::Deref,
    os::unix::io::AsRawFd,
    path::{Path, PathBuf},
    sync::Arc,
};

use libc::off_t;
use memmap2::{MmapMut, MmapOptions};
use parking_lot::{RwLock, RwLockReadGuard};

pub mod error;
mod identifier;
mod layout;
mod reader;
mod region;
mod regions;

pub use error::*;
pub use identifier::*;
use layout::*;
use rayon::prelude::*;
pub use reader::*;
pub use region::*;
use regions::*;

pub const PAGE_SIZE: u64 = 4096;
pub const PAGE_SIZE_MINUS_1: u64 = PAGE_SIZE - 1;
const GB: u64 = 1024 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct Database(Arc<DatabaseInner>);

impl Database {
    pub fn open(path: &Path) -> Result<Self> {
        Ok(Self(Arc::new(DatabaseInner::open(path)?)))
    }
}

impl Deref for Database {
    type Target = DatabaseInner;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct DatabaseInner {
    path: PathBuf,
    regions: RwLock<Regions>,
    layout: RwLock<Layout>,
    file: RwLock<File>,
    mmap: RwLock<MmapMut>,
}

impl DatabaseInner {
    fn open(path: &Path) -> Result<Self> {
        fs::create_dir_all(path)?;

        let regions = Regions::open(path)?;

        let layout = Layout::from(&regions);

        let file = OpenOptions::new()
            .read(true)
            .create(true)
            .write(true)
            .truncate(false)
            .open(Self::data_path_(path))?;
        file.try_lock()?;

        let mmap = Self::create_mmap(&file)?;

        Ok(Self {
            path: path.to_owned(),
            file: RwLock::new(file),
            mmap: RwLock::new(mmap),
            regions: RwLock::new(regions),
            layout: RwLock::new(layout),
        })
    }

    pub fn file_len(&self) -> Result<u64> {
        Ok(self.file.read().metadata()?.len())
    }

    pub fn set_min_len(&self, len: u64) -> Result<()> {
        let len = Self::ceil_number_to_page_size_multiple(len);

        let file_len = self.file_len()?;
        if file_len < len {
            let mut mmap = self.mmap.write();
            let file = self.file.write();
            file.set_len(len)?;
            *mmap = Self::create_mmap(&file)?;
            Ok(())
        } else {
            Ok(())
        }
    }

    pub fn set_min_regions(&self, regions: usize) -> Result<()> {
        self.regions
            .write()
            .set_min_len((regions * SIZE_OF_REGION) as u64)?;
        self.set_min_len(regions as u64 * PAGE_SIZE)
    }

    pub fn create_region_if_needed(&self, id: &str) -> Result<(usize, Arc<RwLock<Region>>)> {
        let regions = self.regions.read();
        if let Some(index) = regions.get_region_index_from_id(id) {
            return Ok((index, regions.get_region_from_index(index).unwrap()));
        }
        drop(regions);

        let mut regions = self.regions.write();
        let mut layout = self.layout.write();

        let start = if let Some(start) = layout.find_smallest_adequate_hole(PAGE_SIZE) {
            layout.remove_or_compress_hole(start, PAGE_SIZE);
            start
        } else {
            let start = layout
                .get_last_region_index()
                .map(|index| {
                    let region_opt = regions.get_region_from_index(index);
                    let region = region_opt.as_ref().unwrap().read();
                    region.start() + region.reserved()
                })
                .unwrap_or_default();

            let len = start + PAGE_SIZE;

            self.set_min_len(len)?;

            start
        };

        let (index, region) = regions.create_region(id.to_owned(), start)?;

        layout.insert_region(start, index);

        Ok((index, region))
    }

    pub fn get_region(&self, identifier: Identifier) -> Result<RwLockReadGuard<'static, Region>> {
        let regions = self.regions.read();
        let region_opt = regions.get_region(identifier);
        let region_arc = region_opt.ok_or(Error::Str("Unknown region"))?;
        let region = region_arc.read();
        let region: RwLockReadGuard<'static, Region> = unsafe { std::mem::transmute(region) };
        Ok(region)
    }

    pub fn create_region_reader<'a>(&'a self, identifier: Identifier) -> Result<Reader<'a>> {
        let mmap: RwLockReadGuard<'a, MmapMut> = self.mmap.read();
        let region = self.get_region(identifier)?;
        Ok(Reader::new(mmap, region))
    }

    #[inline]
    pub fn write_all_to_region(&self, identifier: Identifier, data: &[u8]) -> Result<()> {
        self.write_all_to_region_at_(identifier, data, None, false)
    }

    #[inline]
    pub fn write_all_to_region_at(
        &self,
        identifier: Identifier,
        data: &[u8],
        at: u64,
    ) -> Result<()> {
        self.write_all_to_region_at_(identifier, data, Some(at), false)
    }

    #[inline]
    pub fn truncate_write_all_to_region(
        &self,
        identifier: Identifier,
        at: u64,
        data: &[u8],
    ) -> Result<()> {
        self.write_all_to_region_at_(identifier, data, Some(at), true)
    }

    fn write_all_to_region_at_(
        &self,
        identifier: Identifier,
        data: &[u8],
        at: Option<u64>,
        truncate: bool,
    ) -> Result<()> {
        let regions = self.regions.read();
        let Some(region_lock) = regions.get_region(identifier.clone()) else {
            return Err(Error::Str("Unknown region"));
        };

        let region_index = regions.identifier_to_index(identifier).unwrap();

        let region = region_lock.read();
        let start = region.start();
        let reserved = region.reserved();
        let len = region.len();
        drop(region);

        let data_len = data.len() as u64;
        let new_len = at.map_or(len + data_len, |at| {
            assert!(at <= len);
            let new_len = at + data_len;
            if truncate { new_len } else { new_len.max(len) }
        });
        let write_start = start + at.unwrap_or(len);

        if at.is_some_and(|at| at > reserved) {
            return Err(Error::Str("Invalid at parameter"));
        }

        // Write to reserved space if possible
        if new_len <= reserved {
            // info!(
            //     "Write {data_len} bytes to {region_index} reserved space at {write_start} (start = {start}, at = {at:?}, len = {len})"
            // );

            if at.is_none() {
                self.write(write_start, data);
            }

            let mut region = region_lock.write();

            if at.is_some() {
                self.write(write_start, data);
            }

            region.set_len(new_len);
            regions.write_to_mmap(&region, region_index);

            return Ok(());
        }

        assert!(new_len > reserved);
        let mut new_reserved = reserved;
        while new_len > new_reserved {
            new_reserved *= 2;
        }
        assert!(new_len <= new_reserved);
        let added_reserve = new_reserved - reserved;

        let mut layout = self.layout.write();

        // If is last continue writing
        if layout.is_last_anything(region_index) {
            // info!("{region_index} Append to file at {write_start}");

            self.set_min_len(start + new_reserved)?;
            let mut region = region_lock.write();
            region.set_reserved(new_reserved);
            drop(region);
            drop(layout);

            self.write(write_start, data);

            let mut region = region_lock.write();
            region.set_len(new_len);
            regions.write_to_mmap(&region, region_index);

            return Ok(());
        }

        // Expand region to the right if gap is wide enough
        let hole_start = start + reserved;
        if layout
            .get_hole(hole_start)
            .is_some_and(|gap| gap >= added_reserve)
        {
            // info!("Expand {region_index} to hole");

            layout.remove_or_compress_hole(hole_start, added_reserve);
            let mut region = region_lock.write();
            region.set_reserved(new_reserved);
            drop(region);
            drop(layout);

            self.write(write_start, data);

            let mut region = region_lock.write();
            region.set_len(new_len);
            regions.write_to_mmap(&region, region_index);

            return Ok(());
        }

        // Find hole big enough to move the region
        if let Some(hole_start) = layout.find_smallest_adequate_hole(new_reserved) {
            // info!("Move {region_index} to hole at {hole_start}");

            layout.remove_or_compress_hole(hole_start, new_reserved);
            drop(layout);

            self.write(
                hole_start,
                &self.mmap.read()[start as usize..write_start as usize],
            );
            self.write(hole_start + at.unwrap_or(len), data);

            let mut region = region_lock.write();
            let mut layout = self.layout.write();
            layout.move_region(hole_start, region_index, &region)?;
            drop(layout);

            region.set_start(hole_start);
            region.set_reserved(new_reserved);
            region.set_len(new_len);
            regions.write_to_mmap(&region, region_index);

            return Ok(());
        }

        let new_start = layout.len(&regions);
        // Write at the end
        // info!(
        //     "Move {region_index} to the end, from {start}..{} to {new_start}..{}",
        //     start + reserved,
        //     new_start + new_reserved
        // );
        self.set_min_len(new_start + new_reserved)?;
        layout.reserve(new_start, new_reserved);
        drop(layout);

        self.write(
            new_start,
            &self.mmap.read()[start as usize..write_start as usize],
        );
        self.write(new_start + at.unwrap_or(len), data);

        let mut region = region_lock.write();
        let mut layout = self.layout.write();
        layout.move_region(new_start, region_index, &region)?;
        assert!(layout.reserved(new_start) == Some(new_reserved));
        drop(layout);

        region.set_start(new_start);
        region.set_reserved(new_reserved);
        region.set_len(new_len);
        regions.write_to_mmap(&region, region_index);

        Ok(())
    }

    fn write(&self, at: u64, data: &[u8]) {
        let mmap = self.mmap.read();

        let data_len = data.len();
        let start = at as usize;
        let end = start + data_len;

        if end > mmap.len() {
            unreachable!("Trying to write beyond mmap")
        }

        let slice = unsafe { std::slice::from_raw_parts_mut(mmap.as_ptr() as *mut u8, mmap.len()) };

        slice[start..end].copy_from_slice(data);
    }

    ///
    /// From relative to start
    ///
    /// Non destructive
    ///
    pub fn truncate_region(&self, identifier: Identifier, from: u64) -> Result<()> {
        let Some(region) = self.regions.read().get_region(identifier.clone()) else {
            return Err(Error::Str("Unknown region"));
        };
        let mut region_ = region.write();
        let len = region_.len();
        if from == len {
            return Ok(());
        } else if from > len {
            return Err(Error::Str("Truncating further than length"));
        }
        region_.set_len(from);
        Ok(())
    }

    pub fn remove_region(&self, identifier: Identifier) -> Result<Option<Arc<RwLock<Region>>>> {
        let mut regions = self.regions.write();

        let mut layout = self.layout.write();

        let index_opt = regions.identifier_to_index(identifier.clone());

        let Some(region) = regions.remove_region(identifier)? else {
            return Ok(None);
        };

        let index = index_opt.unwrap();

        let region_ = region.write();

        layout.remove_region(index, &region_)?;

        drop(region_);

        Ok(Some(region))
    }

    fn create_mmap(file: &File) -> Result<MmapMut> {
        Ok(unsafe { MmapOptions::new().map_mut(file)? })
    }

    pub fn regions(&self) -> RwLockReadGuard<'_, Regions> {
        self.regions.read()
    }

    pub fn layout(&self) -> RwLockReadGuard<'_, Layout> {
        self.layout.read()
    }

    pub fn mmap(&self) -> RwLockReadGuard<'_, MmapMut> {
        self.mmap.read()
    }

    fn ceil_number_to_page_size_multiple(num: u64) -> u64 {
        (num + PAGE_SIZE_MINUS_1) & !PAGE_SIZE_MINUS_1
    }

    fn data_path(&self) -> PathBuf {
        Self::data_path_(&self.path)
    }
    fn data_path_(path: &Path) -> PathBuf {
        path.join("data")
    }

    pub fn disk_usage(&self) -> String {
        let path = self.data_path();

        let output = std::process::Command::new("du")
            .arg("-h")
            .arg(&path)
            .output()
            .expect("Failed to run du");

        String::from_utf8_lossy(&output.stdout)
            .replace(path.to_str().unwrap(), " ")
            .trim()
            .to_string()
    }

    pub fn flush(&self) -> Result<()> {
        let mmap = self.mmap.read();
        let regions = self.regions.read();
        mmap.flush()?;
        regions.flush()
    }

    pub fn flush_then_punch(&self) -> Result<()> {
        self.flush()?;
        self.punch_holes()
    }

    pub fn punch_holes(&self) -> Result<()> {
        let file = self.file.write();
        let mut mmap = self.mmap.write();
        let regions = self.regions.read();
        let layout = self.layout.read();

        let mut punched = regions
            .index_to_region()
            .par_iter()
            .flatten()
            .map(|region_lock| -> Result<usize> {
                let region = region_lock.read();
                let rstart = region.start();
                let len = region.len();
                let reserved = region.reserved();
                let ceil_len = Self::ceil_number_to_page_size_multiple(len);
                assert!(len <= ceil_len);
                if ceil_len > reserved {
                    panic!()
                } else if ceil_len < reserved {
                    let start = rstart + ceil_len;
                    let hole = reserved - ceil_len;
                    if Self::approx_has_punchable_data(&mmap, start, hole) {
                        // info!(
                        //     "dbg: {:?}",
                        //     (region, rstart, len, ceil_len, reserved, start, hole)
                        // );
                        // info!("Punching a hole of {hole} bytes at {start}...");
                        Self::punch_hole(&file, start, hole)?;
                        return Ok(1);
                    }
                }
                Ok(0)
            })
            .sum::<Result<usize>>()?;

        punched += layout
            .start_to_hole()
            .par_iter()
            .map(|(&start, &hole)| -> Result<usize> {
                if Self::approx_has_punchable_data(&mmap, start, hole) {
                    // info!("dbg: {:?}", (start, hole));
                    // info!("Punching a hole of {hole} bytes at {start}...");
                    Self::punch_hole(&file, start, hole)?;
                    Ok(1)
                } else {
                    Ok(0)
                }
            })
            .sum::<Result<usize>>()?;

        if punched > 0 {
            unsafe {
                libc::fsync(file.as_raw_fd());
            }
            // info!("Remaping post hole punching...");
            *mmap = Self::create_mmap(&file)?;
        }

        Ok(())
    }

    fn approx_has_punchable_data(mmap: &MmapMut, start: u64, len: u64) -> bool {
        assert!(start % PAGE_SIZE == 0);
        assert!(len % PAGE_SIZE == 0);

        let min = start as usize;
        let max = (start + len) as usize;
        let check = |start, end| {
            assert!(start >= min);
            assert!(end < max);
            let start_is_some = mmap[start] != 0;
            // if start_is_some {
            // info!("mmap[start = {}] = {}", start, mmap[start])
            // }
            let end_is_some = mmap[end] != 0;
            // if end_is_some {
            // info!("mmap[end = {}] = {}", end, mmap[end])
            // }
            start_is_some || end_is_some
        };

        // Check first page (first and last byte)
        let first_page_start = start as usize;
        let first_page_end = (start + PAGE_SIZE - 1) as usize;
        if check(first_page_start, first_page_end) {
            return true;
        }

        // Check last page (first and last byte)
        let last_page_start = (start + len - PAGE_SIZE) as usize;
        let last_page_end = (start + len - 1) as usize;
        if check(last_page_start, last_page_end) {
            return true;
        }

        // For large lengths, check at 1GB intervals
        if len > GB {
            let num_gb_checks = (len / GB) as usize;
            for i in 1..num_gb_checks {
                let gb_boundary = start + (i as u64 * GB);
                let page_start = gb_boundary as usize;
                let page_end = (gb_boundary + PAGE_SIZE - 1) as usize;

                if check(page_start, page_end) {
                    return true;
                }
            }
        }

        false
    }

    #[cfg(target_os = "macos")]
    fn punch_hole(file: &File, start: u64, length: u64) -> Result<()> {
        let fpunchhole = FPunchhole {
            fp_flags: 0,
            reserved: 0,
            fp_offset: start as libc::off_t,
            fp_length: length as libc::off_t,
        };

        let result = unsafe {
            libc::fcntl(
                file.as_raw_fd(),
                libc::F_PUNCHHOLE,
                &fpunchhole as *const FPunchhole,
            )
        };

        if result == -1 {
            let err = std::io::Error::last_os_error();
            return Err(Error::String(format!("Failed to punch hole: {err}")));
        }

        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn punch_hole(file: &File, start: u64, length: u64) -> Result<()> {
        let result = unsafe {
            libc::fallocate(
                file.as_raw_fd(),
                libc::FALLOC_FL_PUNCH_HOLE | libc::FALLOC_FL_KEEP_SIZE,
                start as libc::off_t,
                length as libc::off_t,
            )
        };

        if result == -1 {
            let err = std::io::Error::last_os_error();
            return Err(Error::String(format!("Failed to punch hole: {err}")));
        }

        Ok(())
    }

    #[cfg(target_os = "freebsd")]
    fn punch_hole2(file: &File, start: u64, length: u64) -> Result<()> {
        let fd = file.as_raw_fd();

        let mut spacectl = libc::spacectl_range {
            r_offset: offset as libc::off_t,
            r_len: length as libc::off_t,
        };

        let result = unsafe {
            libc::fspacectl(
                fd,
                libc::SPACECTL_DEALLOC,
                &spacectl as *const libc::spacectl_range,
                0,
                &mut spacectl as *mut libc::spacectl_range,
            )
        };

        if result == -1 {
            let err = std::io::Error::last_os_error();
            return Err(Error::String(format!("Failed to punch hole: {err}")));
        }

        Ok(())
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "freebsd")))]
    fn punch_hole(_file: &File, _start: u64, _length: u64) -> Result<()> {
        Err(Error::String(
            "Hole punching not supported on this platform".to_string(),
        ))
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[repr(C)]
struct FPunchhole {
    fp_flags: u32,
    reserved: u32,
    fp_offset: off_t,
    fp_length: off_t,
}
