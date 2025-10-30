use std::{
    collections::HashMap,
    fs::{self, File, OpenOptions},
    io::{Cursor, Read},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use allocative::Allocative;
use parking_lot::{RwLock, RwLockWriteGuard};
use std::os::unix::fs::FileExt;
use zerocopy::{FromBytes, IntoBytes};

use crate::{Error, Result};

use super::{
    Identifier, PAGE_SIZE,
    region::{Region, SIZE_OF_REGION},
};

#[derive(Debug, Allocative)]
pub struct Regions {
    id_to_index: HashMap<String, usize>,
    id_to_index_path: PathBuf,
    index_to_region: Vec<Option<Arc<RwLock<Region>>>>,
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

        let mut index_to_region: Vec<Option<Arc<RwLock<Region>>>> = vec![];

        id_to_index
            .iter()
            .try_for_each(|(_, &index)| -> Result<()> {
                let start = (index * SIZE_OF_REGION) as u64;
                let mut buffer = vec![0; SIZE_OF_REGION];
                index_to_region_file.read_exact_at(&mut buffer, start)?;
                let region = Region::read_from_bytes(&buffer)?;
                if index_to_region.len() < index + 1 {
                    index_to_region.resize_with(index + 1, Default::default);
                }
                index_to_region
                    .get_mut(index)
                    .unwrap()
                    .replace(Arc::new(RwLock::new(region)));
                Ok(())
            })?;

        // TODO: Removes Nones from vec if needed, update map accordingly and save them

        Ok(Self {
            id_to_index,
            id_to_index_path,
            index_to_region,
            index_to_region_file,
            index_to_region_file_len,
            id_to_index_dirty: AtomicBool::new(false),
        })
    }

    pub fn set_min_len(&mut self, len: u64) -> Result<()> {
        if self.index_to_region_file_len < len {
            self.index_to_region_file.set_len(len)?;
            self.index_to_region_file_len = len;
        }
        Ok(())
    }

    pub fn create_region(
        &mut self,
        id: String,
        start: u64,
    ) -> Result<(usize, Arc<RwLock<Region>>)> {
        let index = self
            .index_to_region
            .iter()
            .enumerate()
            .find(|(_, opt)| opt.is_none())
            .map(|(index, _)| index)
            .unwrap_or_else(|| self.index_to_region.len());

        let region = Region::new(start, 0, PAGE_SIZE);

        self.set_min_len(((index + 1) * SIZE_OF_REGION) as u64)?;

        let region_lock = RwLock::new(region);

        self.write_region(&region_lock.write(), index)?;

        let region_arc = Arc::new(region_lock);

        let region_opt = Some(region_arc.clone());
        if index < self.index_to_region.len() {
            self.index_to_region[index] = region_opt
        } else {
            self.index_to_region.push(region_opt);
        }

        if self.id_to_index.insert(id, index).is_some() {
            return Err(Error::Str("Already exists"));
        }

        self.id_to_index_dirty.store(true, Ordering::Release);

        Ok((index, region_arc))
    }

    #[inline]
    pub fn get_region(&self, identifier: Identifier) -> Option<Arc<RwLock<Region>>> {
        match identifier {
            Identifier::Number(index) => self.get_region_from_index(index),
            Identifier::String(id) => self.get_region_from_id(&id),
        }
    }

    #[inline]
    pub fn get_region_from_index(&self, index: usize) -> Option<Arc<RwLock<Region>>> {
        self.index_to_region.get(index).cloned().flatten()
    }

    #[inline]
    pub fn get_region_from_id(&self, id: &str) -> Option<Arc<RwLock<Region>>> {
        self.get_region_index_from_id(id)
            .and_then(|index| self.get_region_from_index(index))
    }

    #[inline]
    pub fn get_region_index_from_id(&self, id: &str) -> Option<usize> {
        self.id_to_index.get(id).copied()
    }

    fn find_id_from_index(&self, index: usize) -> Option<&String> {
        Some(
            self.id_to_index
                .iter()
                .find(|(_, v)| **v == index)
                .unwrap()
                .0,
        )
    }

    #[inline]
    pub fn index_to_region(&self) -> &[Option<Arc<RwLock<Region>>>] {
        &self.index_to_region
    }

    #[inline]
    pub fn id_to_index(&self) -> &HashMap<String, usize> {
        &self.id_to_index
    }

    #[inline]
    pub fn identifier_to_index(&self, identifier: Identifier) -> Option<usize> {
        match identifier {
            Identifier::Number(index) => Some(index),
            Identifier::String(id) => self.get_region_index_from_id(&id),
        }
    }

    pub fn remove_region(&mut self, identifier: Identifier) -> Result<Option<Arc<RwLock<Region>>>> {
        match identifier {
            Identifier::Number(index) => self.remove_region_from_index(index),
            Identifier::String(id) => self.remove_region_from_id(&id),
        }
    }

    pub fn remove_region_from_id(&mut self, id: &str) -> Result<Option<Arc<RwLock<Region>>>> {
        let Some(index) = self.get_region_index_from_id(id) else {
            return Ok(None);
        };
        self.remove_region_from_index(index)
    }

    pub fn remove_region_from_index(
        &mut self,
        index: usize,
    ) -> Result<Option<Arc<RwLock<Region>>>> {
        let Some(region) = self.index_to_region.get_mut(index).and_then(Option::take) else {
            return Ok(None);
        };

        self.id_to_index
            .remove(&self.find_id_from_index(index).unwrap().to_owned());

        self.id_to_index_dirty.store(true, Ordering::Release);

        Ok(Some(region))
    }

    pub fn write_region(&self, region: &RwLockWriteGuard<Region>, index: usize) -> Result<()> {
        let start = (index * SIZE_OF_REGION) as u64;
        self.index_to_region_file
            .write_all_at(region.as_bytes(), start)?;
        Ok(())
    }

    pub fn flush(&self) -> Result<()> {
        self.index_to_region_file.sync_data()?;

        if self.id_to_index_dirty.swap(false, Ordering::Acquire) {
            fs::write(&self.id_to_index_path, Self::serialize(&self.id_to_index))?;
        }

        Ok(())
    }

    fn serialize(map: &HashMap<String, usize>) -> Vec<u8> {
        let mut buffer = Vec::new();

        buffer.extend_from_slice(&map.len().to_ne_bytes());

        for (key, value) in map {
            buffer.extend_from_slice(key.len().as_bytes());
            buffer.extend_from_slice(key.as_bytes());
            buffer.extend_from_slice(value.as_bytes());
        }

        buffer
    }

    fn deserialize(data: &[u8]) -> Result<HashMap<String, usize>> {
        let mut cursor = Cursor::new(data);
        let mut buffer = [0u8; 8];

        cursor
            .read_exact(&mut buffer)
            .map_err(|_| Error::Str("Failed to read entry count"))?;
        let entry_count = usize::read_from_bytes(&buffer)?;

        let mut map = HashMap::with_capacity(entry_count);

        for _ in 0..entry_count {
            cursor
                .read_exact(&mut buffer)
                .map_err(|_| Error::Str("Failed to read key length"))?;
            let key_len = usize::read_from_bytes(&buffer)?;

            let mut key_bytes = vec![0u8; key_len];
            cursor.read_exact(&mut key_bytes)?;
            let key =
                String::from_utf8(key_bytes).map_err(|_| Error::Str("Invalid UTF-8 in key"))?;

            cursor.read_exact(&mut buffer)?;
            let value = usize::read_from_bytes(&buffer)?;

            map.insert(key, value);
        }

        Ok(map)
    }
}
