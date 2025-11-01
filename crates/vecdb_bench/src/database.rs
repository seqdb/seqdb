use anyhow::Result;
use std::path::Path;

/// Common interface for all database benchmarks
pub trait DatabaseBenchmark: Sized + Send + Sync {
    /// Name of the database for reporting
    fn name() -> &'static str;

    /// Create a new database at the given path
    fn create(path: &Path) -> Result<Self>;

    /// Open an existing populated database
    fn open(path: &Path) -> Result<Self>;

    /// Write sequential u64 values (0, 1, 2, ..., count-1)
    fn write_sequential(&mut self, count: u64) -> Result<()>;

    /// Read all items sequentially, returning the sum for verification
    fn read_sequential(&self) -> Result<u64>;

    /// Read items at the given indices, returning the sum for verification
    fn read_random(&self, indices: &[u64]) -> Result<u64>;

    /// Read items at the given indices using rayon parallel iteration, returning the sum for verification
    fn read_random_rayon(&self, indices: &[u64]) -> Result<u64>;

    /// Ensure all data is flushed to disk
    fn flush(&mut self) -> Result<()>;

    /// Get the approximate size of the database on disk
    fn disk_size(path: &Path) -> Result<u64>;
}
