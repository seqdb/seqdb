use std::path::Path;

use anyhow::Result;
use heed::types::*;
use heed::{Database as HeedDb, EnvOpenOptions};
use rayon::prelude::*;

use crate::database::DatabaseBenchmark;

pub struct LmdbBench {
    env: heed::Env,
    db: HeedDb<U64<byteorder::NativeEndian>, U64<byteorder::NativeEndian>>,
}

impl DatabaseBenchmark for LmdbBench {
    fn name() -> &'static str {
        "lmdb"
    }

    fn create(path: &Path) -> Result<Self> {
        std::fs::create_dir_all(path)?;

        let env = unsafe {
            EnvOpenOptions::new()
                .map_size(100 * 1024 * 1024 * 1024) // 100 GB
                .max_dbs(1)
                .open(path)?
        };

        let mut wtxn = env.write_txn()?;
        let db = env.create_database(&mut wtxn, Some("bench"))?;
        wtxn.commit()?;

        Ok(Self { env, db })
    }

    fn open(path: &Path) -> Result<Self> {
        let env = unsafe {
            EnvOpenOptions::new()
                .map_size(100 * 1024 * 1024 * 1024) // 100 GB
                .max_dbs(1)
                .open(path)?
        };

        let rtxn = env.read_txn()?;
        let db = env.open_database(&rtxn, Some("bench"))?.unwrap();
        rtxn.commit()?;

        Ok(Self { env, db })
    }

    fn write_sequential(&mut self, count: u64) -> Result<()> {
        let mut wtxn = self.env.write_txn()?;
        for i in 0..count {
            self.db.put(&mut wtxn, &i, &i)?;
        }
        wtxn.commit()?;
        Ok(())
    }

    fn read_sequential(&self) -> Result<u64> {
        let mut sum = 0u64;
        let rtxn = self.env.read_txn()?;

        for item in self.db.iter(&rtxn)? {
            let (_, value) = item?;
            sum = sum.wrapping_add(value);
        }

        Ok(sum)
    }

    fn read_random(&self, indices: &[u64]) -> Result<u64> {
        let mut sum = 0u64;
        let rtxn = self.env.read_txn()?;

        for &idx in indices {
            if let Some(value) = self.db.get(&rtxn, &idx)? {
                sum = sum.wrapping_add(value);
            }
        }

        Ok(sum)
    }

    fn read_random_rayon(&self, indices: &[u64]) -> Result<u64> {
        // Split work into chunks to avoid creating too many transactions
        // Use rayon's default thread pool size as chunk count
        let num_chunks = rayon::current_num_threads();
        let chunk_size = indices.len().div_ceil(num_chunks);

        let env = &self.env;
        let db = self.db;

        let sum = indices
            .par_chunks(chunk_size)
            .map(|chunk| {
                // Create a transaction for this chunk of work
                if let Ok(rtxn) = env.read_txn() {
                    let mut local_sum = 0u64;
                    for &idx in chunk {
                        if let Ok(Some(value)) = db.get(&rtxn, &idx) {
                            local_sum = local_sum.wrapping_add(value);
                        }
                    }
                    local_sum
                } else {
                    0
                }
            })
            .reduce(|| 0, |a, b| a.wrapping_add(b));

        Ok(sum)
    }

    fn flush(&mut self) -> Result<()> {
        self.env.force_sync()?;
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
