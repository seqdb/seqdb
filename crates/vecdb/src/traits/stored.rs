use std::{collections::BTreeMap, fs, path::PathBuf};

use parking_lot::RwLock;
use seqdb::{Database, Region};

use crate::{AnyVec, Exit, Result, Stamp, variants::Header};

pub trait AnyStoredVec: AnyVec {
    fn db(&self) -> &Database;

    fn region_index(&self) -> usize;

    fn region(&self) -> &RwLock<Region>;

    fn header(&self) -> &Header;

    fn mut_header(&mut self) -> &mut Header;

    fn saved_stamped_changes(&self) -> u16;

    fn flush(&mut self) -> Result<()>;

    #[inline]
    fn safe_flush(&mut self, exit: &Exit) -> Result<()> {
        // info!("safe flush {}", self.name());
        let _lock = exit.lock();
        self.flush()
    }

    fn real_stored_len(&self) -> usize;
    fn stored_len(&self) -> usize;

    fn update_stamp(&mut self, stamp: Stamp) {
        self.mut_header().update_stamp(stamp);
    }

    fn stamp(&self) -> Stamp {
        self.header().stamp()
    }

    fn changes_path(&self) -> PathBuf {
        self.db().path().join("changes")
    }

    fn serialize_changes(&self) -> Result<Vec<u8>>;

    #[inline]
    fn stamped_flush(&mut self, stamp: Stamp) -> Result<()> {
        self.update_stamp(stamp);

        let saved_stamped_changes = self.saved_stamped_changes();

        if saved_stamped_changes == 0 {
            return self.flush();
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

        self.flush()
    }
}
