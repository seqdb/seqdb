use crate::database::DatabaseBenchmark;
use anyhow::Result;
use fjall3::{Config, Database, Keyspace, KeyspaceCreateOptions, PersistMode};
use std::path::Path;

pub struct Fjall3Bench {
    database: Database,
    keyspace: Keyspace,
}

impl DatabaseBenchmark for Fjall3Bench {
    fn name() -> &'static str {
        "fjall3"
    }

    fn create(path: &Path) -> Result<Self> {
        let database = Database::open(Config::new(path))?;
        let keyspace = database.keyspace("bench", KeyspaceCreateOptions::default())?;
        Ok(Self { database, keyspace })
    }

    fn open(path: &Path) -> Result<Self> {
        let database = Database::open(Config::new(path))?;
        let keyspace = database.keyspace("bench", KeyspaceCreateOptions::default())?;
        Ok(Self { database, keyspace })
    }

    fn write_sequential(&mut self, count: u64) -> Result<()> {
        self.keyspace.ingest((0..count).map(|i| {
            let b = i.to_be_bytes();
            (b, b)
        }))?;
        Ok(())
    }

    fn len(&self) -> Result<u64> {
        Ok(self.keyspace.len()? as u64)
    }

    fn read_sequential(&self) -> Result<u64> {
        let mut sum = 0u64;

        for item in self.keyspace.iter() {
            let value = item.value()?;
            let val = u64::from_be_bytes(value.as_ref().try_into()?);
            sum = sum.wrapping_add(val);
        }

        Ok(sum)
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
