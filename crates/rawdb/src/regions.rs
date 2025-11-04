use std::{
    collections::HashMap,
    fs::{self, File, OpenOptions},
    io::{Cursor, Read, Write},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use allocative::Allocative;
use std::os::unix::fs::FileExt;

use crate::{Database, Error, RegionMetadata, Result};

use super::{
    PAGE_SIZE,
    region::{Region, SIZE_OF_REGION_METADATA},
};

#[derive(Debug, Allocative)]
pub struct Regions {
    id_to_index: HashMap<String, usize>,
    id_to_index_path: PathBuf,
    index_to_region: Vec<Option<Region>>,
    #[allocative(skip)]
    index_to_region_file: File,
    index_to_region_file_len: u64,
    #[allocative(skip)]
    id_to_index_dirty: AtomicBool,
}

impl Regions {
    pub fn open(path: &Path) -> Result<Self> {
        let path = path.join("regions");

        fs::create_dir_all(&path)?;

        let id_to_index_path = path.join("id_to_index");

        let id_to_index: HashMap<String, usize> =
            Self::deserialize(&fs::read(&id_to_index_path).unwrap_or_default()).unwrap_or_default();

        let index_to_region_file = OpenOptions::new()
            .read(true)
            .create(true)
            .write(true)
            .truncate(false)
            .open(path.join("index_to_region"))?;
        index_to_region_file.try_lock()?;

        let index_to_region_file_len = index_to_region_file.metadata()?.len();

        // Ensure directory entries are durable
        File::open(path)?.sync_data()?;

        Ok(Self {
            id_to_index,
            id_to_index_path,
            index_to_region: vec![],
            index_to_region_file,
            index_to_region_file_len,
            id_to_index_dirty: AtomicBool::new(false),
        })
    }

    pub fn fill_index_to_region(&mut self, db: &Database) -> Result<()> {
        self.id_to_index
            .iter()
            .try_for_each(|(_, &index)| -> Result<()> {
                let start = (index * SIZE_OF_REGION_METADATA) as u64;
                let mut buffer = vec![0; SIZE_OF_REGION_METADATA];
                self.index_to_region_file
                    .read_exact_at(&mut buffer, start)?;
                let meta = RegionMetadata::from_bytes(&buffer)?;
                if self.index_to_region.len() < index + 1 {
                    self.index_to_region
                        .resize_with(index + 1, Default::default);
                }
                self.index_to_region
                    .get_mut(index)
                    .unwrap()
                    .replace(Region::from(db, index, meta));
                Ok(())
            })
    }

    pub fn set_min_len(&mut self, len: u64) -> Result<()> {
        if self.index_to_region_file_len < len {
            self.index_to_region_file.set_len(len)?;
            self.index_to_region_file_len = len;
        }
        Ok(())
    }

    pub fn create_region(&mut self, db: &Database, id: String, start: u64) -> Result<Region> {
        let index = self
            .index_to_region
            .iter()
            .enumerate()
            .find(|(_, opt)| opt.is_none())
            .map(|(index, _)| index)
            .unwrap_or_else(|| self.index_to_region.len());

        let region = Region::new(db, index, start, 0, PAGE_SIZE);

        self.set_min_len(((index + 1) * SIZE_OF_REGION_METADATA) as u64)?;

        self.write_region(&region)?;

        let region_opt = Some(region.clone());
        if index < self.index_to_region.len() {
            self.index_to_region[index] = region_opt
        } else {
            self.index_to_region.push(region_opt);
        }

        if self.id_to_index.insert(id, index).is_some() {
            return Err(Error::Str("Already exists"));
        }

        self.id_to_index_dirty.store(true, Ordering::Release);

        Ok(region)
    }

    #[inline]
    pub fn get_region_from_index(&self, index: usize) -> Option<&Region> {
        self.index_to_region.get(index).and_then(Option::as_ref)
    }

    #[inline]
    pub fn get_region_from_id(&self, id: &str) -> Option<&Region> {
        self.id_to_index
            .get(id)
            .and_then(|&index| self.get_region_from_index(index))
    }

    fn find_id_from_index(&self, index: usize) -> Option<&String> {
        Some(self.id_to_index.iter().find(|(_, v)| **v == index)?.0)
    }

    #[inline]
    pub fn index_to_region(&self) -> &[Option<Region>] {
        &self.index_to_region
    }

    #[inline]
    pub fn id_to_index(&self) -> &HashMap<String, usize> {
        &self.id_to_index
    }

    pub fn remove_region(&mut self, region: Region) -> Result<Option<Region>> {
        if self
            .index_to_region
            .get_mut(region.index)
            .and_then(Option::take)
            .is_none()
        {
            return Err(Error::Str(
                "Couldn't find region in regions.index_to_region",
            ));
        } else if Arc::strong_count(&region) > 1 {
            return Err(Error::Str("Cannot remove a region held multiple times"));
        }

        let index = region.index;

        self.id_to_index.remove(
            &self
                .find_id_from_index(index)
                .ok_or(Error::Str("Expect regions.id_to_index to know region"))?
                .to_owned(),
        );

        self.id_to_index_dirty.store(true, Ordering::Release);

        Ok(Some(region))
    }

    pub fn write_region(&self, region: &Region) -> Result<()> {
        self.write_region_(region.index, &region.meta.read())
    }

    pub fn write_region_(&self, index: usize, region_meta: &RegionMetadata) -> Result<()> {
        let start = (index * SIZE_OF_REGION_METADATA) as u64;
        let bytes = region_meta.to_bytes();
        self.index_to_region_file.write_all_at(&bytes, start)?;
        Ok(())
    }

    pub fn flush(&self) -> Result<()> {
        self.index_to_region_file.sync_data()?;

        if self.id_to_index_dirty.swap(false, Ordering::Acquire) {
            let tmp = self.id_to_index_path.with_extension("tmp");

            let mut file = File::create(&tmp)?;
            file.write_all(&Self::serialize(&self.id_to_index))?;
            file.sync_data()?;
            drop(file); // Close before rename

            fs::rename(&tmp, &self.id_to_index_path)?;

            File::open(self.id_to_index_path.parent().unwrap())?.sync_data()?; // Sync dir
        }

        Ok(())
    }

    fn serialize(map: &HashMap<String, usize>) -> Vec<u8> {
        let mut buffer = Vec::new();

        buffer.extend_from_slice(&map.len().to_le_bytes());

        for (key, value) in map {
            buffer.extend_from_slice(&key.len().to_le_bytes());
            buffer.extend_from_slice(key.as_bytes());
            buffer.extend_from_slice(&value.to_le_bytes());
        }

        buffer
    }

    fn deserialize(data: &[u8]) -> Result<HashMap<String, usize>> {
        let mut cursor = Cursor::new(data);
        let mut buffer = [0u8; 8];

        cursor
            .read_exact(&mut buffer)
            .map_err(|_| Error::Str("Failed to read entry count"))?;
        let entry_count = usize::from_le_bytes(buffer);

        let mut map = HashMap::with_capacity(entry_count);

        for _ in 0..entry_count {
            cursor
                .read_exact(&mut buffer)
                .map_err(|_| Error::Str("Failed to read key length"))?;
            let key_len = usize::from_le_bytes(buffer);

            let mut key_bytes = vec![0u8; key_len];
            cursor.read_exact(&mut key_bytes)?;
            let key =
                String::from_utf8(key_bytes).map_err(|_| Error::Str("Invalid UTF-8 in key"))?;

            cursor.read_exact(&mut buffer)?;
            let value = usize::from_le_bytes(buffer);

            map.insert(key, value);
        }

        Ok(map)
    }
}
