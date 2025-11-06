use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
    fs,
    path::PathBuf,
};

use log::info;
use rawdb::Reader;
use zerocopy::FromBytes;

use crate::{AnyStoredVec, Error, Exit, Result, SEPARATOR, Stamp, Version};

const ONE_KIB: usize = 1024;
const ONE_MIB: usize = ONE_KIB * ONE_KIB;
const MAX_CACHE_SIZE: usize = 256 * ONE_MIB;

use super::{StoredIndex, StoredRaw};

pub trait GenericStoredVec<I, T>: Send + Sync
where
    Self: AnyStoredVec,
    I: StoredIndex,
    T: StoredRaw,
{
    const SIZE_OF_T: usize = size_of::<T>();

    ///
    /// Be careful with deadlocks
    ///
    /// You'll want to drop the reader before mutable ops
    ///
    fn create_reader(&'_ self) -> Reader<'_> {
        self.create_static_reader()
    }

    ///
    /// Be careful with deadlocks
    ///
    /// You'll want to drop the reader before mutable ops
    ///
    fn create_static_reader(&self) -> Reader<'static> {
        unsafe { std::mem::transmute(self.region().create_reader()) }
    }

    /// Reads value at index using provided reader. Panics if read fails.
    #[inline]
    fn read_unchecked(&self, index: I, reader: &Reader) -> T {
        self.read_with(index, reader).unwrap()
    }

    // With usize index (internal)
    #[inline]
    fn read_unchecked_at(&self, index: usize, reader: &Reader) -> T {
        self.read_at(index, reader).unwrap()
    }

    /// Reads value at index using provided reader.
    #[inline]
    fn read_with(&self, index: I, reader: &Reader) -> Result<T> {
        self.read_at(index.to_usize(), reader)
    }

    /// Reads value at index, creating a temporary reader.
    /// For multiple reads, prefer `read_with` with a reused reader.
    #[inline]
    fn read(&self, index: I) -> Result<T> {
        self.read_with(index, &self.create_reader())
    }

    /// Internal implementation for reading at usize index.
    fn read_at(&self, index: usize, reader: &Reader) -> Result<T>;

    /// Gets value from any layer (updated, pushed, or storage). Panics on error.
    #[inline]
    fn get_or_read_unchecked(&self, index: I) -> T {
        self.get_or_read(index).unwrap().unwrap()
    }

    /// Gets value from any layer, creating a temporary reader if needed.
    /// Returns None if index is in holes or beyond available data.
    #[inline]
    fn get_or_read(&self, index: I) -> Result<Option<T>> {
        self.get_or_read_with(index, &self.create_reader())
    }

    /// Gets value from any layer using provided reader.
    #[inline]
    fn get_or_read_with(&self, index: I, reader: &Reader) -> Result<Option<T>> {
        self.get_or_read_at(index.to_usize(), reader)
    }

    /// Internal implementation checking all layers.
    #[inline]
    fn get_or_read_at(&self, index: usize, reader: &Reader) -> Result<Option<T>> {
        // Check holes first
        let holes = self.holes();
        if !holes.is_empty() && holes.contains(&index) {
            return Ok(None);
        }

        let stored_len = self.stored_len();

        // Check pushed (beyond stored length)
        if index >= stored_len {
            return Ok(self.get_pushed_at(index, stored_len).cloned());
        }

        // Check updated layer
        let updated = self.updated();
        if !updated.is_empty()
            && let Some(updated_value) = updated.get(&index)
        {
            return Ok(Some(updated_value.clone()));
        }

        // Fall back to reading from storage
        Ok(Some(self.read_at(index, reader)?))
    }

    // ============================================================================
    // Pushed-only operations (skips updated layer)
    // ============================================================================

    /// Gets value from pushed layer or storage, creating a temporary reader.
    /// Does not check the updated layer.
    #[inline]
    fn get_pushed_or_read(&self, index: I) -> Result<Option<T>> {
        self.get_pushed_or_read_with(index, &self.create_reader())
    }

    /// Gets value from pushed layer or storage using provided reader.
    #[inline]
    fn get_pushed_or_read_with(&self, index: I, reader: &Reader) -> Result<Option<T>> {
        self.get_pushed_or_read_at(index.to_usize(), reader)
    }

    /// Internal implementation checking only pushed and storage.
    #[inline]
    fn get_pushed_or_read_at(&self, index: usize, reader: &Reader) -> Result<Option<T>> {
        let stored_len = self.stored_len();

        if index >= stored_len {
            return Ok(self.get_pushed_at(index, stored_len).cloned());
        }

        Ok(Some(self.read_at(index, reader)?))
    }

    /// Gets value from pushed layer only.
    #[inline(always)]
    fn get_pushed_at(&self, index: usize, stored_len: usize) -> Option<&T> {
        let pushed = self.pushed();
        let offset = index.checked_sub(stored_len)?;
        pushed.get(offset)
    }

    #[inline]
    fn len_(&self) -> usize {
        self.stored_len() + self.pushed_len()
    }

    fn prev_pushed(&self) -> &[T];
    fn mut_prev_pushed(&mut self) -> &mut Vec<T>;
    fn pushed(&self) -> &[T];
    fn mut_pushed(&mut self) -> &mut Vec<T>;
    #[inline]
    fn pushed_len(&self) -> usize {
        self.pushed().len()
    }
    #[inline]
    fn push(&mut self, value: T) {
        self.mut_pushed().push(value)
    }

    #[inline]
    fn push_if_needed(&mut self, index: I, value: T) -> Result<()> {
        let index_usize = index.to_usize();
        let len = self.len();

        if index_usize == len {
            self.push(value);
            return Ok(());
        }

        // Already pushed
        if index_usize < len {
            return Ok(());
        }

        // This should never happen in correct code
        debug_assert!(
            false,
            "Index too high: idx={}, len={}, header={:?}, region={}",
            index_usize,
            len,
            self.header(),
            self.region().index()
        );

        Err(Error::IndexTooHigh)
    }

    #[inline]
    fn forced_push_at(&mut self, index: I, value: T, exit: &Exit) -> Result<()> {
        self.forced_push_at_(index.to_usize(), value, exit)
    }

    #[inline]
    fn forced_push_at_(&mut self, index: usize, value: T, exit: &Exit) -> Result<()> {
        match self.len().cmp(&index.to_usize()) {
            Ordering::Less => {
                return Err(Error::IndexTooHigh);
            }
            ord => {
                if ord == Ordering::Greater {
                    self.truncate_if_needed_(index)?;
                }
                self.push(value);
            }
        }

        let pushed_bytes = self.pushed_len() * Self::SIZE_OF_T;
        if pushed_bytes >= MAX_CACHE_SIZE {
            // info!("pushed_bytes ({pushed_bytes}) >= MAX_CACHE_SIZE ({MAX_CACHE_SIZE})");
            self.safe_flush(exit)?;
        }

        Ok(())
    }

    #[inline]
    fn update_or_push(&mut self, index: I, value: T) -> Result<()> {
        let len = self.len();
        match len.cmp(&index.to_usize()) {
            Ordering::Less => {
                dbg!(index, value, len, self.header());
                Err(Error::IndexTooHigh)
            }
            Ordering::Equal => {
                self.push(value);
                Ok(())
            }
            Ordering::Greater => self.update(index, value),
        }
    }

    #[inline]
    fn get_first_empty_index(&self) -> I {
        self.holes()
            .first()
            .cloned()
            .unwrap_or_else(|| self.len_())
            .into()
    }

    #[inline]
    fn fill_first_hole_or_push(&mut self, value: T) -> Result<I> {
        Ok(
            if let Some(hole) = self.mut_holes().pop_first().map(I::from) {
                self.update(hole, value)?;
                hole
            } else {
                self.push(value);
                I::from(self.len() - 1)
            },
        )
    }

    fn holes(&self) -> &BTreeSet<usize>;
    fn mut_holes(&mut self) -> &mut BTreeSet<usize>;

    fn prev_holes(&self) -> &BTreeSet<usize>;
    fn mut_prev_holes(&mut self) -> &mut BTreeSet<usize>;

    fn take(&mut self, index: I, reader: &Reader) -> Result<Option<T>> {
        let opt = self.get_or_read_with(index, reader)?;
        if opt.is_some() {
            self.unchecked_delete(index);
        }
        Ok(opt)
    }

    #[inline]
    fn delete(&mut self, index: I) {
        if index.to_usize() < self.len() {
            self.unchecked_delete(index);
        }
    }
    #[inline]
    #[doc(hidden)]
    fn unchecked_delete(&mut self, index: I) {
        let uindex = index.to_usize();
        let updated = self.mut_updated();
        if !updated.is_empty() {
            updated.remove(&uindex);
        }
        self.mut_holes().insert(uindex);
    }

    fn updated(&self) -> &BTreeMap<usize, T>;
    fn mut_updated(&mut self) -> &mut BTreeMap<usize, T>;

    fn prev_updated(&self) -> &BTreeMap<usize, T>;
    fn mut_prev_updated(&mut self) -> &mut BTreeMap<usize, T>;

    #[inline]
    fn update(&mut self, index: I, value: T) -> Result<()> {
        self.update_(index.to_usize(), value)
    }

    #[inline]
    fn update_(&mut self, index: usize, value: T) -> Result<()> {
        let stored_len = self.stored_len();

        if index >= stored_len {
            if let Some(prev) = self.mut_pushed().get_mut(index - stored_len) {
                *prev = value;
                return Ok(());
            } else {
                return Err(Error::IndexTooHigh);
            }
        }

        let holes = self.mut_holes();
        if !holes.is_empty() {
            holes.remove(&index);
        }

        self.mut_updated().insert(index, value);

        Ok(())
    }

    fn reset(&mut self) -> Result<()>;

    #[inline]
    fn reset_(&mut self) -> Result<()> {
        self.truncate_if_needed_(0)
    }

    fn validate_computed_version_or_reset(&mut self, version: Version) -> Result<()> {
        if version != self.header().computed_version() {
            self.mut_header().update_computed_version(version);
            if !self.is_empty() {
                self.reset()?;
            }
        }

        if self.is_empty() {
            info!(
                "Computing {}_to_{}...",
                self.index_type_to_string(),
                self.name()
            )
        }

        Ok(())
    }

    #[inline]
    fn is_pushed_empty(&self) -> bool {
        self.pushed_len() == 0
    }

    #[inline]
    fn has(&self, index: I) -> bool {
        self.has_(index.to_usize())
    }
    #[inline]
    fn has_(&self, index: usize) -> bool {
        index < self.len_()
    }

    fn prev_stored_len(&self) -> usize;
    fn mut_prev_stored_len(&mut self) -> &mut usize;
    #[doc(hidden)]
    fn update_stored_len(&self, val: usize);

    fn truncate_if_needed(&mut self, index: I) -> Result<()> {
        self.truncate_if_needed_(index.to_usize())
    }
    fn truncate_if_needed_(&mut self, index: usize) -> Result<()> {
        let stored_len = self.stored_len();
        let pushed_len = self.pushed_len();
        let len = stored_len + pushed_len;

        if index >= len {
            return Ok(());
        }

        if self.holes().last().is_some_and(|&h| h >= index) {
            self.mut_holes().retain(|&i| i < index);
        }

        if self
            .updated()
            .last_key_value()
            .is_some_and(|(&k, _)| k >= index)
        {
            self.mut_updated().retain(|&i, _| i < index);
        }

        if index <= stored_len {
            self.mut_pushed().clear();
        } else {
            self.mut_pushed().truncate(index - stored_len);
        }

        if index >= stored_len {
            return Ok(());
        }

        self.update_stored_len(index);

        Ok(())
    }

    #[inline]
    fn truncate_if_needed_with_stamp(&mut self, index: I, stamp: Stamp) -> Result<()> {
        self.update_stamp(stamp);
        self.truncate_if_needed(index)
    }

    fn deserialize_then_undo_changes(&mut self, bytes: &[u8]) -> Result<()> {
        let mut pos = 0;
        let mut len = 8;

        let prev_stamp = u64::read_from_bytes(&bytes[..pos + len])?;
        self.mut_header().update_stamp(Stamp::new(prev_stamp));
        pos += len;

        let prev_stored_len = usize::read_from_bytes(&bytes[pos..pos + len])?;
        pos += len;

        let _stored_len = usize::read_from_bytes(&bytes[pos..pos + len])?;
        pos += len;

        let current_stored_len = self.stored_len();

        // Restore to the length BEFORE the changes that we're undoing
        if prev_stored_len < current_stored_len {
            // Shrinking - truncate will handle this
            self.truncate_if_needed_(prev_stored_len)?;
        } else if prev_stored_len > current_stored_len {
            // Expanding - truncate won't handle this, manually set stored_len
            self.update_stored_len(prev_stored_len);
        }
        // If equal, no change needed

        let truncated_count = usize::read_from_bytes(&bytes[pos..pos + len])?;
        pos += len;

        // Clear pushed (will be replaced with prev_pushed from change file)
        self.mut_pushed().clear();

        // DON'T clear updated! The change file only contains indices modified in this stamp.
        // When doing multiple rollbacks, we need to preserve updates from previous rollbacks.
        // The update_() calls below will overwrite specific indices as needed.

        // Restore truncated items into the updated map since they're now at indices < stored_len
        // The disk still has stale data for these indices, so we need to override with correct values
        if truncated_count > 0 {
            len = Self::SIZE_OF_T * truncated_count;
            let truncated_values = bytes[pos..pos + len]
                .chunks(Self::SIZE_OF_T)
                .map(|b| T::read_from_bytes(b).map_err(|_| Error::ZeroCopyError))
                .collect::<Result<Vec<_>>>()?;
            pos += len;

            // Add truncated values to updated map at their correct indices
            let start_index = prev_stored_len - truncated_count;
            for (i, val) in truncated_values.into_iter().enumerate() {
                self.mut_updated().insert(start_index + i, val);
            }
        }

        len = 8;
        let prev_pushed_len = usize::read_from_bytes(&bytes[pos..pos + len])?;
        pos += len;
        len = Self::SIZE_OF_T * prev_pushed_len;
        let mut prev_pushed = bytes[pos..pos + len]
            .chunks(Self::SIZE_OF_T)
            .map(|s| T::read_from_bytes(s).map_err(|_| Error::ZeroCopyError))
            .collect::<Result<Vec<_>>>()?;
        pos += len;
        self.mut_pushed().append(&mut prev_pushed);

        len = 8;
        let pushed_len = usize::read_from_bytes(&bytes[pos..pos + len])?;
        pos += len;
        len = Self::SIZE_OF_T * pushed_len;
        let _pushed = bytes[pos..pos + len]
            .chunks(Self::SIZE_OF_T)
            .map(|s| T::read_from_bytes(s).map_err(|_| Error::ZeroCopyError))
            .collect::<Result<Vec<_>>>()?;
        pos += len;

        len = 8;
        let prev_modified_len = usize::read_from_bytes(&bytes[pos..pos + len])?;
        pos += len;
        len = size_of::<usize>() * prev_modified_len;
        let prev_indexes = bytes[pos..pos + len].chunks(8);
        pos += len;
        len = Self::SIZE_OF_T * prev_modified_len;
        let prev_values = bytes[pos..pos + len].chunks(Self::SIZE_OF_T);
        let _prev_updated: BTreeMap<usize, T> = prev_indexes
            .zip(prev_values)
            .map(|(i, v)| {
                let idx = usize::read_from_bytes(i).map_err(|_| Error::ZeroCopyError)?;
                let val = T::read_from_bytes(v).map_err(|_| Error::ZeroCopyError)?;
                Ok((idx, val))
            })
            .collect::<Result<_>>()?;
        pos += len;

        len = 8;
        let modified_len = usize::read_from_bytes(&bytes[pos..pos + len])?;
        pos += len;
        len = size_of::<usize>() * modified_len;
        let indexes = bytes[pos..pos + len].chunks(8);
        pos += len;
        len = Self::SIZE_OF_T * modified_len;
        let values = bytes[pos..pos + len].chunks(Self::SIZE_OF_T);
        let old_values_to_restore: BTreeMap<usize, T> = indexes
            .zip(values)
            .map(|(i, v)| {
                let idx = usize::read_from_bytes(i).map_err(|_| Error::ZeroCopyError)?;
                let val = T::read_from_bytes(v).map_err(|_| Error::ZeroCopyError)?;
                Ok((idx, val))
            })
            .collect::<Result<_>>()?;
        pos += len;

        len = 8;
        let prev_holes_len = usize::read_from_bytes(&bytes[pos..pos + len])?;
        pos += len;
        len = size_of::<usize>() * prev_holes_len;
        let prev_holes = bytes[pos..pos + len]
            .chunks(8)
            .map(|b| usize::read_from_bytes(b).map_err(|_| Error::ZeroCopyError))
            .collect::<Result<BTreeSet<_>>>()?;
        pos += len;

        len = 8;
        let holes_len = usize::read_from_bytes(&bytes[pos..pos + len])?;
        pos += len;
        len = size_of::<usize>() * holes_len;
        let _holes = bytes[pos..pos + len]
            .chunks(8)
            .map(|b| usize::read_from_bytes(b).map_err(|_| Error::ZeroCopyError))
            .collect::<Result<BTreeSet<_>>>()?;

        if !self.holes().is_empty() || !self.prev_holes().is_empty() || !prev_holes.is_empty() {
            *self.mut_holes() = prev_holes.clone();
            *self.mut_prev_holes() = prev_holes;
        }

        // Restore old values to updated map (the "modified" section contains the values we want to restore)
        old_values_to_restore
            .into_iter()
            .try_for_each(|(i, v)| self.update_(i, v))?;

        // After rollback, prev_* should reflect the rolled-back state (for the next flush)
        *self.mut_prev_updated() = self.updated().clone();
        *self.mut_prev_pushed() = self.pushed().to_vec();
        // prev_holes and prev_stored_len are already set above

        Ok(())
    }

    #[inline]
    fn stamped_flush_maybe_with_changes(&mut self, stamp: Stamp, with_changes: bool) -> Result<()> {
        if with_changes {
            self.stamped_flush_with_changes(stamp)
        } else {
            self.stamped_flush(stamp)
        }
    }

    fn changes_path(&self) -> PathBuf {
        self.db_path().join(self.index_to_name()).join("changes")
    }

    #[inline]
    fn stamped_flush_with_changes(&mut self, stamp: Stamp) -> Result<()> {
        let saved_stamped_changes = self.saved_stamped_changes();

        if saved_stamped_changes == 0 {
            return self.stamped_flush(stamp);
        }

        let path = self.changes_path();

        fs::create_dir_all(&path)?;

        let files: BTreeMap<Stamp, PathBuf> = fs::read_dir(&path)?
            .filter_map(|entry| {
                let path = entry.ok()?.path();
                let name = path.file_name()?.to_str()?;
                if let Ok(s) = name.parse::<u64>().map(Stamp::from) {
                    if s < stamp {
                        Some((s, path))
                    } else {
                        let _ = fs::remove_file(path);
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        for (_, path) in files.iter().take(
            files
                .len()
                .saturating_sub((saved_stamped_changes - 1) as usize),
        ) {
            fs::remove_file(path)?;
        }

        // Save current state BEFORE flush clears it (flush uses mem::take on pushed/updated)
        let holes_before_flush = self.holes().clone();

        fs::write(
            path.join(u64::from(stamp).to_string()),
            self.serialize_changes()?,
        )?;

        self.stamped_flush(stamp)?;

        // Update prev_ fields to reflect the PERSISTED state after flush
        // After flush: pushed data → stored (on disk), updated → stored (on disk)
        // So prev_pushed is always empty, prev_updated is ALSO empty (updates are now on disk)
        *self.mut_prev_stored_len() = self.stored_len(); // Use NEW stored_len after flush
        *self.mut_prev_pushed() = vec![]; // Always empty after flush - pushed data is now stored
        *self.mut_prev_updated() = BTreeMap::new(); // Always empty after flush - updated data is now stored
        *self.mut_prev_holes() = holes_before_flush;

        Ok(())
    }

    fn rollback_before(&mut self, stamp: Stamp) -> Result<Stamp> {
        if self.stamp() < stamp {
            return Ok(self.stamp());
        }

        let dir = fs::read_dir(self.changes_path())?
            .filter_map(|entry| {
                let path = entry.ok()?.path();
                let name = path.file_name()?.to_str()?;
                if let Ok(stamp) = name.parse::<u64>().map(Stamp::from) {
                    Some((stamp, path))
                } else {
                    None
                }
            })
            .collect::<BTreeMap<Stamp, PathBuf>>();

        let mut iter = dir.range(..=self.stamp());

        while let Some((&s, _)) = iter.next_back()
            && self.stamp() >= stamp
        {
            if s != self.stamp() {
                dbg!((s, self.stamp(), stamp));
                return Err(Error::Str("File stamp should be the same as vec stamp"));
            }
            self.rollback()?;
        }

        // Save the restored state to prev_ fields so they're available for the next flush
        *self.mut_prev_stored_len() = self.stored_len();
        *self.mut_prev_pushed() = self.pushed().to_vec();
        *self.mut_prev_updated() = self.updated().clone();
        *self.mut_prev_holes() = self.holes().clone();

        Ok(self.stamp())
    }

    fn is_dirty(&mut self) -> bool {
        !self.is_pushed_empty() || !self.updated().is_empty()
    }

    fn rollback(&mut self) -> Result<()> {
        let path = self
            .changes_path()
            .join(u64::from(self.stamp()).to_string());
        let bytes = fs::read(&path)?;
        self.deserialize_then_undo_changes(&bytes)
    }

    fn reset_unsaved(&mut self) {
        self.mut_pushed().clear();
        if !self.holes().is_empty() {
            self.mut_holes().clear();
        }
        if !self.updated().is_empty() {
            self.mut_updated().clear();
        }
    }

    fn collect_holed(&self) -> Result<Vec<Option<T>>> {
        self.collect_holed_range(None, None)
    }

    fn collect_holed_range(
        &self,
        from: Option<usize>,
        to: Option<usize>,
    ) -> Result<Vec<Option<T>>> {
        let len = self.len();
        let from = from.unwrap_or_default();
        let to = to.map_or(len, |to| to.min(len));

        if from >= len || from >= to {
            return Ok(vec![]);
        }

        let reader = self.create_reader();

        (from..to)
            .map(|i| self.get_or_read_at(i, &reader))
            .collect::<Result<Vec<_>>>()
    }

    fn vec_region_name(&self) -> String {
        Self::vec_region_name_(self.name())
    }
    // MUST BE in sync with AnyVec::index_to_name
    fn vec_region_name_(name: &str) -> String {
        format!("{}{SEPARATOR}{}", I::to_string(), name)
    }

    fn holes_region_name(&self) -> String {
        Self::holes_region_name_(self.name())
    }
    fn holes_region_name_(name: &str) -> String {
        format!("{}_holes", Self::vec_region_name_(name))
    }
}
