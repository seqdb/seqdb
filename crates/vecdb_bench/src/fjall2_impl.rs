use std::{
    path::Path,
    sync::atomic::{AtomicU64, Ordering},
    thread,
};

use anyhow::Result;
use fjall2::{Config, Keyspace, PartitionCreateOptions, PartitionHandle, PersistMode};

use crate::database::DatabaseBenchmark;

pub struct Fjall2Bench {
    keyspace: Keyspace,
    partition: PartitionHandle,
}

impl DatabaseBenchmark for Fjall2Bench {
    fn name() -> &'static str {
        "fjall2"
    }

    fn create(path: &Path) -> Result<Self> {
        let keyspace = Config::new(path).open()?;
        let partition = keyspace.open_partition("bench", PartitionCreateOptions::default())?;
        Ok(Self {
            keyspace,
            partition,
        })
    }

    fn open(path: &Path) -> Result<Self> {
        let keyspace = Config::new(path).open()?;
        let partition = keyspace.open_partition("bench", PartitionCreateOptions::default())?;
        Ok(Self {
            keyspace,
            partition,
        })
    }

    fn write_sequential(&mut self, count: u64) -> Result<()> {
        self.partition.ingest((0..count).map(|i| {
            let b = i.to_be_bytes();
            (b, b)
        }))?;
        Ok(())
    }

    fn len(&self) -> Result<u64> {
        Ok(self.partition.len()? as u64)
    }

    fn read_sequential(&self) -> Result<u64> {
        let mut sum = 0u64;

        for item in self.partition.iter() {
            let (_, value) = item?;
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
                    let partition = self.partition.clone();
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
                            if let Some(value) = partition.get(key).ok().flatten()
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
            if let Some(value) = self.partition.get(key)? {
                let val = u64::from_be_bytes(value.as_ref().try_into()?);
                sum = sum.wrapping_add(val);
            }
        }

        Ok(sum)
    }

    fn read_random_threaded(&self, indices_per_thread: &[Vec<u64>]) -> Result<u64> {
        let total_sum = AtomicU64::new(0);

        thread::scope(|s| {
            let handles: Vec<_> = indices_per_thread
                .iter()
                .map(|indices| {
                    let partition = self.partition.clone();
                    s.spawn(move || {
                        let mut sum = 0u64;
                        for &idx in indices {
                            let key = idx.to_be_bytes();
                            if let Some(value) = partition.get(key).ok().flatten()
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

    fn flush(&mut self) -> Result<()> {
        self.keyspace.persist(PersistMode::SyncAll)?;
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
