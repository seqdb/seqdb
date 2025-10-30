use anyhow::Result;
use rayon::prelude::*;
use std::{
    path::Path,
    sync::atomic::{AtomicU64, Ordering},
    thread,
};
use vecdb_old::{AnyStoredVec, AnyVec, Database, GenericStoredVec, RawVec, Version};

use crate::database::DatabaseBenchmark;

pub struct VecDbOldBench {
    vec: RawVec<usize, u64>,
}

impl DatabaseBenchmark for VecDbOldBench {
    fn name() -> &'static str {
        "vecdb_old"
    }

    fn create(path: &Path) -> Result<Self> {
        let database = Database::open(path)?;
        let options = (&database, "bench", Version::TWO).into();
        let vec: RawVec<usize, u64> = RawVec::forced_import_with(options)?;
        Ok(Self { vec })
    }

    fn open(path: &Path) -> Result<Self> {
        let database = Database::open(path)?;
        let options = (&database, "bench", Version::TWO).into();
        let vec: RawVec<usize, u64> = RawVec::forced_import_with(options)?;
        Ok(Self { vec })
    }

    fn write_sequential(&mut self, count: u64) -> Result<()> {
        for i in 0..count {
            self.vec.push(i);
        }
        Ok(())
    }

    fn len(&self) -> Result<u64> {
        Ok(self.vec.len() as u64)
    }

    fn read_sequential(&self) -> Result<u64> {
        let mut sum = 0u64;
        let values = self.vec.iter();
        for (_, value) in values {
            sum = sum.wrapping_add(value.into_owned());
        }

        Ok(sum)
    }

    fn read_sequential_threaded(&self, num_threads: usize) -> Result<u64> {
        let total_sum = AtomicU64::new(0);
        let len = self.vec.len();
        let chunk_size = len / num_threads;

        thread::scope(|s| {
            let handles: Vec<_> = (0..num_threads)
                .map(|thread_id| {
                    s.spawn(move || {
                        let start = thread_id * chunk_size;

                        let mut sum = 0u64;

                        let values = self.vec.iter_at(start).take(chunk_size);
                        for (_, value) in values {
                            sum = sum.wrapping_add(value.into_owned());
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
        let reader = self.vec.create_reader();

        for &idx in indices {
            if let Ok(value) = self.vec.read_(idx as usize, &reader) {
                sum = sum.wrapping_add(value);
            }
        }

        Ok(sum)
    }

    fn read_random_rayon(&self, indices: &[u64]) -> Result<u64> {
        let reader = self.vec.create_reader();
        let sum = indices
            .par_iter()
            .map(|&idx| self.vec.read_(idx as usize, &reader).unwrap_or_default())
            .reduce(|| 0, |a, b| a.wrapping_add(b));

        Ok(sum)
    }

    fn flush(&mut self) -> Result<()> {
        self.vec.db().flush()?;
        self.vec.flush()?;
        Ok(())
    }

    fn disk_size(path: &Path) -> Result<u64> {
        let mut total = 0u64;
        if path.exists() {
            for entry in std::fs::read_dir(path)? {
                let entry = entry?;
                if entry.file_type()?.is_file() {
                    total += entry.metadata()?.len();
                }
            }
        }
        Ok(total)
    }
}
