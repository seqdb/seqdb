use crate::database::DatabaseBenchmark;
use anyhow::Result;
use fjall3::{Config, Keyspace, PartitionCreateOptions};
use std::path::Path;

pub struct FjallBench {
    keyspace: Keyspace,
    partition: fjall::PartitionHandle,
}

impl DatabaseBenchmark for FjallBench {
    fn name() -> &'static str {
        "fjall3"
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
        for i in 0..count {
            let key = i.to_be_bytes();
            let value = i.to_be_bytes();
            self.partition.insert(key, value)?;
        }
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

    fn flush(&mut self) -> Result<()> {
        self.keyspace.persist(fjall::PersistMode::SyncAll)?;
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
