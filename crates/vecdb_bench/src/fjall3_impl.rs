use std::{
    path::Path,
    sync::atomic::{AtomicU64, Ordering},
    thread,
};

use anyhow::Result;
use fjall3::{KeyspaceCreateOptions, PersistMode, TxDatabase, TxKeyspace};
use rayon::prelude::*;

use crate::database::DatabaseBenchmark;

pub struct Fjall3Bench {
    database: TxDatabase,
    keyspace: TxKeyspace,
}

impl DatabaseBenchmark for Fjall3Bench {
    fn name() -> &'static str {
        "fjall3"
    }

    fn create(path: &Path) -> Result<Self> {
        Self::open(path)
    }

    fn open(path: &Path) -> Result<Self> {
        let database = TxDatabase::builder(path)
            .cache_size(1024 * 1024 * 1024)
            .open()?;
        let options = KeyspaceCreateOptions::default();
        let keyspace = database.keyspace("bench", options)?;
        Ok(Self { database, keyspace })
    }

    fn write_sequential(&mut self, count: u64) -> Result<()> {
        // Should be another test
        // self.keyspace.ingest((0..count).map(|i| {
        //     let b = i.to_be_bytes();
        //     (b, b)
        // }))?;

        (0..count).try_for_each(|i| {
            let b = i.to_be_bytes();
            self.keyspace.insert(b, b)
        })?;

        Ok(())
    }

    fn len(&self) -> Result<u64> {
        Ok(self.database.read_tx().len(&self.keyspace)? as u64)
    }

    fn read_sequential(&self) -> Result<u64> {
        let mut sum = 0u64;

        for item in self.database.read_tx().iter(&self.keyspace) {
            let value = item.value()?;
            let val = u64::from_be_bytes(value.as_ref().try_into()?);
            sum = sum.wrapping_add(val);
        }

        Ok(sum)
    }

    fn read_sequential_threaded(&self, num_threads: usize) -> Result<u64> {
        let total_sum = AtomicU64::new(0);
        let len = self.len()?;
        let chunk_size = len / num_threads as u64;

        thread::scope(|s| {
            let handles: Vec<_> = (0..num_threads)
                .map(|thread_id| {
                    let keyspace = self.keyspace.clone();
                    s.spawn(move || {
                        let start = thread_id as u64 * chunk_size;
                        let end = if thread_id == num_threads - 1 {
                            len
                        } else {
                            (thread_id as u64 + 1) * chunk_size
                        };

                        let mut sum = 0u64;
                        for idx in start..end {
                            let key = idx.to_be_bytes();
                            if let Some(value) = keyspace.get(key).ok().flatten()
                                && let Ok(val_bytes) = TryInto::<[u8; 8]>::try_into(value.as_ref())
                            {
                                let val = u64::from_be_bytes(val_bytes);
                                sum = sum.wrapping_add(val);
                            }
                        }
                        sum
                    })
                })
                .collect();

            for handle in handles {
                if let Ok(sum) = handle.join() {
                    total_sum.fetch_add(sum, Ordering::Relaxed);
                }
            }
        });

        Ok(total_sum.load(Ordering::Relaxed))
    }

    fn read_random(&self, indices: &[u64]) -> Result<u64> {
        let mut sum = 0u64;

        for &idx in indices {
            let key = idx.to_be_bytes();
            if let Some(value) = self.keyspace.get(key)? {
                let val = u64::from_be_bytes(value.as_ref().try_into()?);
                sum = sum.wrapping_add(val);
            }
        }

        Ok(sum)
    }

    fn read_random_rayon(&self, indices: &[u64]) -> Result<u64> {
        let keyspace = &self.keyspace;
        let sum = indices
            .par_iter()
            .map(|&idx| {
                let key = idx.to_be_bytes();
                if let Some(value) = keyspace.get(key).ok().flatten()
                    && let Ok(val_bytes) = TryInto::<[u8; 8]>::try_into(value.as_ref())
                {
                    u64::from_be_bytes(val_bytes)
                } else {
                    0
                }
            })
            .reduce(|| 0, |a, b| a.wrapping_add(b));

        Ok(sum)
    }

    fn flush(&mut self) -> Result<()> {
        self.database.persist(PersistMode::SyncAll)?;
        Ok(())
    }

    fn disk_size(path: &Path) -> Result<u64> {
        let mut total = 0u64;
        if path.exists() {
            for entry in walkdir::WalkDir::new(path) {
                let entry = entry?;
                if entry.file_type().is_file() {
                    total += entry.metadata()?.len();
                }
            }
        }
        Ok(total)
    }
}
