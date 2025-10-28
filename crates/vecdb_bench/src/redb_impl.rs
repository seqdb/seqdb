use crate::database::DatabaseBenchmark;
use anyhow::Result;
use redb::{Database, ReadableDatabase, ReadableTable, ReadableTableMetadata, TableDefinition};
use std::path::Path;

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
