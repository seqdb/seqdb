use std::{
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    thread,
};

use anyhow::Result;
use rayon::prelude::*;
use redb::{Database, ReadableDatabase, ReadableTable, ReadableTableMetadata, TableDefinition};

use crate::database::DatabaseBenchmark;

const TABLE: TableDefinition<u64, u64> = TableDefinition::new("bench");

pub struct RedbBench {
    db: Database,
}

impl DatabaseBenchmark for RedbBench {
    fn name() -> &'static str {
        "redb"
    }

    fn create(path: &Path) -> Result<Self> {
        let db_path = path.join("bench.redb");
        let db = Database::create(&db_path)?;
        Ok(Self { db })
    }

    fn open(path: &Path) -> Result<Self> {
        let db_path = path.join("bench.redb");
        let db = Database::open(&db_path)?;
        Ok(Self { db })
    }

    fn write_sequential(&mut self, count: u64) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(TABLE)?;
            for i in 0..count {
                table.insert(i, i)?;
            }
        }
        write_txn.commit()?;
        Ok(())
    }

    fn len(&self) -> Result<u64> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(TABLE)?;
        Ok(table.len()?)
    }

    fn read_sequential(&self) -> Result<u64> {
        let mut sum = 0u64;
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(TABLE)?;

        for item in table.iter()? {
            let (_, value) = item?;
            sum = sum.wrapping_add(value.value());
        }

        Ok(sum)
    }

    fn read_sequential_threaded(&self, num_threads: usize) -> Result<u64> {
        let total_sum = AtomicU64::new(0);
        let db = Arc::new(&self.db);
        let len = self.len()?;
        let chunk_size = len / num_threads as u64;

        thread::scope(|s| {
            let handles: Vec<_> = (0..num_threads)
                .map(|thread_id| {
                    let db = db.clone();
                    s.spawn(move || {
                        let start = thread_id as u64 * chunk_size;
                        let end = if thread_id == num_threads - 1 {
                            len
                        } else {
                            (thread_id as u64 + 1) * chunk_size
                        };

                        let mut sum = 0u64;
                        if let Ok(read_txn) = db.begin_read()
                            && let Ok(table) = read_txn.open_table(TABLE)
                        {
                            for idx in start..end {
                                if let Ok(Some(value)) = table.get(idx) {
                                    sum = sum.wrapping_add(value.value());
                                }
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
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(TABLE)?;

        for &idx in indices {
            if let Some(value) = table.get(idx)? {
                sum = sum.wrapping_add(value.value());
            }
        }

        Ok(sum)
    }

    fn read_random_rayon(&self, indices: &[u64]) -> Result<u64> {
        use std::cell::RefCell;

        thread_local! {
            static TXN_CACHE: RefCell<Option<(redb::ReadTransaction, redb::ReadOnlyTable<u64, u64>)>> = const { RefCell::new(None) };
        }

        let db = &self.db;
        let sum = indices
            .par_iter()
            .map(|&idx| {
                TXN_CACHE.with(|cache| {
                    let mut cache_opt = cache.borrow_mut();
                    if cache_opt.is_none()
                        && let Ok(read_txn) = db.begin_read()
                        && let Ok(table) = read_txn.open_table(TABLE)
                    {
                        *cache_opt = Some((read_txn, table));
                    }
                    let table = &cache_opt.as_ref().unwrap().1;
                    if let Ok(Some(value)) = table.get(idx) {
                        return value.value();
                    }
                    0
                })
            })
            .reduce(|| 0, |a, b| a.wrapping_add(b));

        Ok(sum)
    }

    fn flush(&mut self) -> Result<()> {
        // redb automatically syncs on commit, but we can force a checkpoint
        // No explicit flush needed - commits are durable
        Ok(())
    }

    fn disk_size(path: &Path) -> Result<u64> {
        let db_path = path.join("bench.redb");
        if db_path.exists() {
            Ok(std::fs::metadata(&db_path)?.len())
        } else {
            Ok(0)
        }
    }
}
