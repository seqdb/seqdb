use std::path::Path;

use anyhow::Result;
use rayon::prelude::*;
use rocksdb::{DB, Options, WriteBatch};

use crate::database::DatabaseBenchmark;

pub struct RocksDbBench {
    db: DB,
}

impl DatabaseBenchmark for RocksDbBench {
    fn name() -> &'static str {
        "rocksdb"
    }

    fn create(path: &Path) -> Result<Self> {
        let db_path = path.join("bench.rocksdb");
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.set_compression_type(rocksdb::DBCompressionType::None);
        let db = DB::open(&opts, &db_path)?;
        Ok(Self { db })
    }

    fn open(path: &Path) -> Result<Self> {
        let db_path = path.join("bench.rocksdb");
        let mut opts = Options::default();
        opts.create_if_missing(false);
        let db = DB::open(&opts, &db_path)?;
        Ok(Self { db })
    }

    fn write_sequential(&mut self, count: u64) -> Result<()> {
        let mut batch = WriteBatch::default();
        for i in 0..count {
            let key = i.to_le_bytes();
            let value = i.to_le_bytes();
            batch.put(key, value);

            // Commit batch every 10000 writes to avoid memory issues
            if i % 10000 == 9999 {
                self.db.write(batch)?;
                batch = WriteBatch::default();
            }
        }
        // Write any remaining items
        if !batch.is_empty() {
            self.db.write(batch)?;
        }
        Ok(())
    }

    fn read_sequential(&self) -> Result<u64> {
        let mut sum = 0u64;
        let iter = self.db.iterator(rocksdb::IteratorMode::Start);

        for item in iter {
            let (_, value) = item?;
            let value_u64 = u64::from_le_bytes(value.as_ref().try_into()?);
            sum = sum.wrapping_add(value_u64);
        }

        Ok(sum)
    }

    fn read_random(&self, indices: &[u64]) -> Result<u64> {
        let mut sum = 0u64;

        for &idx in indices {
            let key = idx.to_le_bytes();
            if let Some(value) = self.db.get(key)? {
                let value_u64 = u64::from_le_bytes(value.as_slice().try_into()?);
                sum = sum.wrapping_add(value_u64);
            }
        }

        Ok(sum)
    }

    fn read_random_rayon(&self, indices: &[u64]) -> Result<u64> {
        let db = &self.db;
        let sum = indices
            .par_iter()
            .map(|&idx| {
                let key = idx.to_le_bytes();
                if let Ok(Some(value)) = db.get(key) {
                    u64::from_le_bytes(value.as_slice().try_into().unwrap_or([0u8; 8]))
                } else {
                    0
                }
            })
            .reduce(|| 0, |a, b| a.wrapping_add(b));

        Ok(sum)
    }

    fn flush(&mut self) -> Result<()> {
        self.db.flush()?;
        Ok(())
    }

    fn disk_size(path: &Path) -> Result<u64> {
        let db_path = path.join("bench.rocksdb");
        if !db_path.exists() {
            return Ok(0);
        }

        let mut total_size = 0u64;
        for entry in walkdir::WalkDir::new(&db_path) {
            let entry = entry?;
            if entry.file_type().is_file() {
                total_size += entry.metadata()?.len();
            }
        }

        Ok(total_size)
    }
}
