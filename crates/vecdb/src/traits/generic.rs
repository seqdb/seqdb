use std::{
    borrow::Cow,
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
    fs,
    path::PathBuf,
};

use parking_lot::RwLockWriteGuard;
use seqdb::Reader;
use zerocopy::FromBytes;

use crate::{AnyStoredVec, Error, Exit, Result, Stamp};

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
        unsafe {
            std::mem::transmute(
                self.db()
                    .create_region_reader(self.region_index().into())
                    .unwrap(),
            )
        }
    }

    #[inline]
    fn unwrap_read(&self, index: I, reader: &Reader) -> T {
        self.read(index, reader).unwrap()
    }
    #[inline]
    fn unwrap_read_(&self, index: usize, reader: &Reader) -> T {
        self.read_(index, reader).unwrap()
    }
    #[inline]
    fn read(&self, index: I, reader: &Reader) -> Result<T> {
        self.read_(index.to_usize()?, reader)
    }
    fn read_(&self, index: usize, reader: &Reader) -> Result<T>;

    #[inline]
    fn get_or_read(&'_ self, index: I, reader: &Reader) -> Result<Option<Cow<'_, T>>> {
        self.get_or_read_(index.to_usize()?, reader)
    }
    #[inline]
    fn get_or_read_(&'_ self, index: usize, reader: &Reader) -> Result<Option<Cow<'_, T>>> {
        let stored_len = self.stored_len();

        let holes = self.holes();
        if !holes.is_empty() && holes.contains(&index) {
            return Ok(None);
        }

        if index >= stored_len {
            let pushed = self.pushed();
            let j = index - stored_len;
            if j >= pushed.len() {
                return Ok(None);
            }
            return Ok(pushed.get(j).map(Cow::Borrowed));
        }

        let updated = self.updated();
        if !updated.is_empty()
            && let Some(updated) = updated.get(&index)
        {
            return Ok(Some(Cow::Borrowed(updated)));
        }

        Ok(Some(Cow::Owned(self.read_(index, reader)?)))
    }

    #[inline]
    fn len_(&self) -> usize {
        self.stored_len() + self.pushed_len()
    }

    fn pushed(&self) -> &[T];
    #[inline]
    fn pushed_len(&self) -> usize {
        self.pushed().len()
    }
    fn mut_pushed(&mut self) -> &mut Vec<T>;
    #[inline]
    fn push(&mut self, value: T) {
        self.mut_pushed().push(value)
    }

    #[inline]
    fn push_if_needed(&mut self, index: I, value: T) -> Result<()> {
        let len = self.len();
        match len.cmp(&index.to_usize()?) {
            Ordering::Greater => {
                // dbg!(len, index, &self.pathbuf);
                // panic!();
                Ok(())
            }
            Ordering::Equal => {
                self.push(value);
                Ok(())
            }
            Ordering::Less => {
                dbg!(index, value, len, self.header(), self.region_index());
                Err(Error::IndexTooHigh)
            }
        }
    }

    #[inline]
    fn forced_push_at(&mut self, index: I, value: T, exit: &Exit) -> Result<()> {
        match self.len().cmp(&index.to_usize()?) {
            Ordering::Less => {
                return Err(Error::IndexTooHigh);
            }
            ord => {
                if ord == Ordering::Greater {
                    self.truncate_if_needed(index)?;
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
        match len.cmp(&index.to_usize()?) {
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
    fn take(&mut self, index: I, reader: &Reader) -> Result<Option<T>> {
        let opt = self.get_or_read(index, reader)?.map(|v| v.into_owned());
        if opt.is_some() {
            self.unchecked_delete(index);
        }
        Ok(opt)
    }
    #[inline]
    fn delete(&mut self, index: I) {
        if index.unwrap_to_usize() < self.len() {
            self.unchecked_delete(index);
        }
    }
    #[inline]
    #[doc(hidden)]
    fn unchecked_delete(&mut self, index: I) {
        let uindex = index.unwrap_to_usize();
        let updated = self.mut_updated();
        if !updated.is_empty() {
            updated.remove(&uindex);
        }
        self.mut_holes().insert(uindex);
    }

    fn updated(&self) -> &BTreeMap<usize, T>;
    fn mut_updated(&mut self) -> &mut BTreeMap<usize, T>;
    #[inline]
    fn update(&mut self, index: I, value: T) -> Result<()> {
        self.update_(index.unwrap_to_usize(), value)
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

    #[inline]
    fn is_pushed_empty(&self) -> bool {
        self.pushed_len() == 0
    }

    #[inline]
    fn has(&self, index: I) -> Result<bool> {
        Ok(self.has_(index.to_usize()?))
    }
    #[inline]
    fn has_(&self, index: usize) -> bool {
        index < self.len_()
    }

    #[doc(hidden)]
    fn mut_stored_len(&'_ self) -> RwLockWriteGuard<'_, usize>;

    fn truncate_if_needed(&mut self, index: I) -> Result<()> {
        self.truncate_if_needed_(index.to_usize()?)
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

        *self.mut_stored_len() = index;

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
        self.truncate_if_needed_(prev_stored_len)?;
        pos += len;

        let truncated = usize::read_from_bytes(&bytes[pos..pos + len])?;
        pos += len;

        if truncated > 0 {
            len = Self::SIZE_OF_T * truncated;
            bytes[pos..pos + len]
                .chunks(Self::SIZE_OF_T)
                .try_for_each(|b| -> Result<()> {
                    self.push(T::read_from_bytes(b)?);
                    Ok(())
                })?;
            pos += len;
        }

        len = 8;
        let prev_holes_len = usize::read_from_bytes(&bytes[pos..pos + len])?;
        pos += len;

        len = size_of::<usize>() * prev_holes_len;
        let holes = bytes[pos..pos + len]
            .chunks(8)
            .map(|b| usize::read_from_bytes(b).map_err(|_| Error::ZeroCopyError))
            .collect::<Result<BTreeSet<_>>>()?;
        pos += len;

        len = 8;
        let modified_len = usize::read_from_bytes(&bytes[pos..pos + len])?;
        pos += len;

        len = size_of::<usize>() * modified_len;
        let indexes = bytes[pos..pos + len].chunks(8);
        pos += len;

        len = Self::SIZE_OF_T * modified_len;
        let values = bytes[pos..pos + len].chunks(Self::SIZE_OF_T);

        let updated: BTreeMap<usize, T> = indexes
            .zip(values)
            .map(|(i, v)| {
                let idx = usize::read_from_bytes(i).map_err(|_| Error::ZeroCopyError)?;
                let val = T::read_from_bytes(v).map_err(|_| Error::ZeroCopyError)?;
                Ok((idx, val))
            })
            .collect::<Result<_>>()?;

        *self.mut_holes() = holes;
        updated
            .into_iter()
            .try_for_each(|(i, v)| self.update_(i, v))?;

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
        self.db().path().join(self.index_to_name()).join("changes")
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

        fs::write(
            path.join(u64::from(stamp).to_string()),
            self.serialize_changes()?,
        )?;

        self.stamped_flush(stamp)
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
                // dbg!((s, self.stamp()));
                return Err(Error::Str("File stamp should be the same as vec stamp"));
            }
            self.rollback()?;
        }

        Ok(self.stamp())
    }

    fn rollback(&mut self) -> Result<()> {
        let bytes = fs::read(
            self.changes_path()
                .join(u64::from(self.stamp()).to_string()),
        )?;
        self.deserialize_then_undo_changes(&bytes)
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
            .map(|i| {
                self.get_or_read_(i, &reader)
                    .map(|o| o.map(|c| c.into_owned()))
            })
            .collect::<Result<Vec<_>>>()
    }

    fn index_to_name(&self) -> String {
        format!("{}_to_{}", I::to_string(), self.name())
    }

    fn vec_region_name(&self) -> String {
        Self::vec_region_name_(self.name())
    }
    fn vec_region_name_(name: &str) -> String {
        format!("{}_to_{name}", I::to_string())
    }

    fn holes_region_name(&self) -> String {
        Self::holes_region_name_(self.name())
    }
    fn holes_region_name_(name: &str) -> String {
        format!("{}_holes", Self::vec_region_name_(name))
    }
}
