use std::{
    collections::HashMap,
    fs::{self, File, OpenOptions},
    path::Path,
    sync::Arc,
};

use std::os::unix::fs::FileExt;

use crate::{Database, Error, RegionMetadata, Result};

use super::{
    PAGE_SIZE,
    region::{Region, SIZE_OF_REGION_METADATA},
};

#[derive(Debug)]
pub struct Regions {
    id_to_index: HashMap<String, usize>,
    index_to_region: Vec<Option<Region>>,
    file: File,
    file_len: u64,
}

impl Regions {
    pub fn open(parent: &Path) -> Result<Self> {
        fs::create_dir_all(parent)?;

        let file = OpenOptions::new()
            .read(true)
            .create(true)
            .write(true)
            .truncate(false)
            .open(parent.join("regions"))?;
        file.try_lock()?;

        let file_len = file.metadata()?.len();

        Ok(Self {
            id_to_index: HashMap::new(),
            index_to_region: vec![],
            file,
            file_len,
        })
    }

    pub fn fill_index_to_region(&mut self, db: &Database) -> Result<()> {
        assert_eq!(self.file_len % SIZE_OF_REGION_METADATA as u64, 0);

        let num_slots = (self.file_len / SIZE_OF_REGION_METADATA as u64) as usize;

        self.index_to_region
            .resize_with(num_slots, Default::default);

        for index in 0..num_slots {
            let start = (index * SIZE_OF_REGION_METADATA) as u64;
            let mut buffer = vec![0; SIZE_OF_REGION_METADATA];

            self.file.read_exact_at(&mut buffer, start)?;

            let Ok(meta) = RegionMetadata::from_bytes(&buffer) else {
                continue;
            };

            self.id_to_index.insert(meta.id().to_string(), index);
            self.index_to_region[index] = Some(Region::from(db, index, meta));
        }

        Ok(())
    }

    pub fn set_min_len(&mut self, len: u64) -> Result<()> {
        if self.file_len < len {
            self.file.set_len(len)?;
            self.file_len = len;
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

        let region = Region::new(db, id.clone(), index, start, 0, PAGE_SIZE);

        self.set_min_len(((index + 1) * SIZE_OF_REGION_METADATA) as u64)?;

        let region_opt = Some(region.clone());
        if index < self.index_to_region.len() {
            self.index_to_region[index] = region_opt
        } else {
            self.index_to_region.push(region_opt);
        }

        if self.id_to_index.insert(id, index).is_some() {
            return Err(Error::RegionAlreadyExists);
        }

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

    #[inline]
    pub fn index_to_region(&self) -> &[Option<Region>] {
        &self.index_to_region
    }

    #[inline]
    pub fn id_to_index(&self) -> &HashMap<String, usize> {
        &self.id_to_index
    }

    pub fn rename_region(&mut self, old_id: &str, new_id: &str) -> Result<()> {
        // Check that old_id exists
        let index = self
            .id_to_index
            .get(old_id)
            .copied()
            .ok_or(Error::RegionNotFound)?;

        // Check that new_id doesn't already exist
        if self.id_to_index.contains_key(new_id) {
            return Err(Error::RegionAlreadyExists);
        }

        // Get the region and update its metadata
        let region = self
            .get_region_from_index(index)
            .ok_or(Error::RegionNotFound)?;

        region.meta().write().set_id(new_id.to_string());

        // Update the id_to_index mapping
        self.id_to_index.remove(old_id);
        self.id_to_index.insert(new_id.to_string(), index);

        Ok(())
    }

    pub fn remove_region(&mut self, region: Region) -> Result<Option<Region>> {
        if self
            .index_to_region
            .get_mut(region.index())
            .and_then(Option::take)
            .is_none()
        {
            return Err(Error::RegionNotFound);
        } else if Arc::strong_count(&region) > 1 {
            return Err(Error::RegionStillReferenced {
                ref_count: Arc::strong_count(&region),
            });
        }

        self.id_to_index.remove(region.meta().read().id());

        // Clear metadata from file by writing zeros
        let start = (region.index() * SIZE_OF_REGION_METADATA) as u64;
        let empty = [0u8; SIZE_OF_REGION_METADATA];
        self.file.write_all_at(&empty, start)?;

        Ok(Some(region))
    }

    pub fn flush(&self) -> Result<()> {
        let mut needs_sync = false;

        // Write all dirty metadata to file
        for (index, region) in self
            .index_to_region
            .iter()
            .enumerate()
            .flat_map(|(i, opt)| opt.as_ref().map(|r| (i, r)))
        {
            let mut region_meta = region.meta().write();
            if region_meta.is_clean() {
                continue;
            }
            needs_sync = true;
            let start = (index * SIZE_OF_REGION_METADATA) as u64;
            let bytes = region_meta.to_bytes();
            self.file.write_all_at(&bytes, start)?;
            region_meta.clear_dirty();
        }

        if needs_sync {
            // Sync the metadata file
            self.file.sync_data()?;
        }

        Ok(())
    }
}
