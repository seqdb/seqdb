use crate::database::DatabaseBenchmark;
use anyhow::Result;
use rand::{Rng, SeedableRng};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

pub const WRITE_COUNT: u64 = 10_000_000;
pub const RANDOM_READ_COUNT: usize = 1_000_000;
pub const RANDOM_SEED: u64 = 42;
pub const NUM_ITERATIONS: usize = 5;

pub struct BenchmarkResult {
    pub name: String,
    pub open_time: Duration,
    pub write_time: Duration,
    pub len_time: Duration,
    pub linear_read_time: Duration,
    pub random_read_time: Duration,
    pub disk_size: u64,
}

impl BenchmarkResult {
    fn format_duration(d: Duration) -> String {
        let secs = d.as_secs_f64();
        if secs < 1.0 {
            format!("{:.2} ms", secs * 1000.0)
        } else if secs < 60.0 {
            format!("{:.2} s", secs)
        } else {
            let mins = (secs / 60.0).floor();
            let secs = secs % 60.0;
            format!("{:.0}m {:.2}s", mins, secs)
        }
    }

    fn format_size(bytes: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        if bytes >= GB {
            format!("{:.2} GB", bytes as f64 / GB as f64)
        } else if bytes >= MB {
            format!("{:.2} MB", bytes as f64 / MB as f64)
        } else if bytes >= KB {
            format!("{:.2} KB", bytes as f64 / KB as f64)
        } else {
            format!("{} B", bytes)
        }
    }

    fn format_throughput(ops: u64, duration: Duration) -> String {
        let secs = duration.as_secs_f64();
        if secs < 0.000001 {
            return String::from("N/A");
        }
        let ops_per_sec = ops as f64 / secs;
        if ops_per_sec >= 1_000_000.0 {
            format!("{:.2} M ops/s", ops_per_sec / 1_000_000.0)
        } else if ops_per_sec >= 1_000.0 {
            format!("{:.2} K ops/s", ops_per_sec / 1_000.0)
        } else {
            format!("{:.2} ops/s", ops_per_sec)
        }
    }

    fn format_bandwidth(bytes: u64, duration: Duration) -> String {
        let secs = duration.as_secs_f64();
        if secs < 0.000001 {
            return String::from("N/A");
        }
        let bytes_per_sec = bytes as f64 / secs;
        let mb_per_sec = bytes_per_sec / (1024.0 * 1024.0);
        if mb_per_sec >= 1024.0 {
            format!("{:.2} GB/s", mb_per_sec / 1024.0)
        } else {
            format!("{:.2} MB/s", mb_per_sec)
        }
    }
}

pub struct BenchmarkRunner {
    base_path: PathBuf,
}

impl BenchmarkRunner {
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self {
        Self {
            base_path: base_path.as_ref().to_path_buf(),
        }
    }

    fn db_path(&self, db_name: &str) -> PathBuf {
        self.base_path.join(db_name)
    }

    fn clean_path(&self, db_name: &str) -> Result<()> {
        let path = self.db_path(db_name);
        if path.exists() {
            std::fs::remove_dir_all(&path)?;
        }
        std::fs::create_dir_all(&path)?;
        Ok(())
    }

    pub fn generate_random_indices(count: u64) -> Vec<u64> {
        let mut rng = rand::rngs::StdRng::seed_from_u64(RANDOM_SEED);
        (0..RANDOM_READ_COUNT)
            .map(|_| rng.random_range(0..count))
            .collect()
    }

    pub fn prepare_database<DB: DatabaseBenchmark>(&self) -> Result<Duration> {
        let name = DB::name();
        print!("  {} ... ", name);

        // Clean and prepare
        self.clean_path(name)?;
        let path = self.db_path(name);

        // Create and write
        let mut db = DB::create(&path)?;

        let start = Instant::now();
        db.write_sequential(WRITE_COUNT)?;
        let write_time = start.elapsed();

        // Flush
        db.flush()?;
        drop(db);

        println!("done");

        Ok(write_time)
    }

    pub fn run_iteration<DB: DatabaseBenchmark>(
        &self,
        indices: &[u64],
    ) -> Result<(Duration, Duration, Duration, Duration)> {
        let name = DB::name();
        let path = self.db_path(name);

        // Open
        let start = Instant::now();
        let db = DB::open(&path)?;
        let open_time = start.elapsed();

        // len()
        let start = Instant::now();
        let _len = db.len()?;
        let len_time = start.elapsed();

        // Linear reads
        let start = Instant::now();
        let _sum = db.read_sequential()?;
        let linear_read_time = start.elapsed();

        // Random reads
        let start = Instant::now();
        let _sum = db.read_random(indices)?;
        let random_read_time = start.elapsed();

        drop(db);

        Ok((open_time, len_time, linear_read_time, random_read_time))
    }

    pub fn measure_disk_size<DB: DatabaseBenchmark>(&self) -> Result<u64> {
        let name = DB::name();
        let path = self.db_path(name);
        DB::disk_size(&path)
    }

    pub fn cleanup<DB: DatabaseBenchmark>(&self) -> Result<()> {
        let name = DB::name();
        self.clean_path(name)
    }

    pub fn print_summary(results: &[BenchmarkResult]) {
        println!("\nRESULTS\n");

        let write_bytes = WRITE_COUNT * 8;
        let random_bytes = RANDOM_READ_COUNT as u64 * 8;

        for result in results {
            println!("{}", result.name);
            println!(
                "  Open:        {}",
                BenchmarkResult::format_duration(result.open_time)
            );
            println!(
                "  Write:       {} | {}",
                BenchmarkResult::format_throughput(WRITE_COUNT, result.write_time),
                BenchmarkResult::format_bandwidth(write_bytes, result.write_time)
            );
            println!(
                "  len():       {}",
                BenchmarkResult::format_duration(result.len_time)
            );
            println!(
                "  Linear:      {} | {}",
                BenchmarkResult::format_throughput(WRITE_COUNT, result.linear_read_time),
                BenchmarkResult::format_bandwidth(write_bytes, result.linear_read_time)
            );
            println!(
                "  Random:      {} | {}",
                BenchmarkResult::format_throughput(
                    RANDOM_READ_COUNT as u64,
                    result.random_read_time
                ),
                BenchmarkResult::format_bandwidth(random_bytes, result.random_read_time)
            );
            println!(
                "  Disk Size:   {}",
                BenchmarkResult::format_size(result.disk_size)
            );
            println!();
        }
    }

    pub fn write_readme(results: &[BenchmarkResult]) -> Result<()> {
        let readme_path = Path::new("README.md");
        let mut file = std::fs::File::create(readme_path)?;

        writeln!(file, "# VecDB Benchmark")?;
        writeln!(file)?;
        writeln!(
            file,
            "Benchmark comparing vecdb against popular embedded databases: fjall, redb, and lmdb."
        )?;
        writeln!(file)?;
        writeln!(
            file,
            "**Test**: {} million sequential u64 writes, linear reads, and {} million random reads.",
            WRITE_COUNT / 1_000_000,
            RANDOM_READ_COUNT / 1_000_000
        )?;
        writeln!(file)?;
        writeln!(file, "## Results")?;
        writeln!(file)?;

        // Find best values (min for time/size, max for throughput)
        let best_open = results.iter().map(|r| r.open_time).min().unwrap();
        let best_write = results.iter().map(|r| r.write_time).min().unwrap();
        let best_len = results.iter().map(|r| r.len_time).min().unwrap();
        let best_linear = results.iter().map(|r| r.linear_read_time).min().unwrap();
        let best_random = results.iter().map(|r| r.random_read_time).min().unwrap();
        let best_disk = results.iter().map(|r| r.disk_size).min().unwrap();

        // Results table
        writeln!(
            file,
            "| Database | Open | Write | len() | Linear Read | Random Read | Disk Size |"
        )?;
        writeln!(
            file,
            "|----------|------|-------|-------|-------------|-------------|-----------|"
        )?;

        let write_bytes = WRITE_COUNT * 8;
        let random_bytes = RANDOM_READ_COUNT as u64 * 8;
        for result in results {
            let open_str = BenchmarkResult::format_duration(result.open_time);
            let open_val = if result.open_time == best_open {
                format!("**{}**", open_str)
            } else {
                open_str
            };

            let write_info = format!(
                "{} ({})",
                BenchmarkResult::format_throughput(WRITE_COUNT, result.write_time),
                BenchmarkResult::format_bandwidth(write_bytes, result.write_time)
            );
            let write_val = if result.write_time == best_write {
                format!("**{}**", write_info)
            } else {
                write_info
            };

            let len_str = BenchmarkResult::format_duration(result.len_time);
            let len_val = if result.len_time == best_len {
                format!("**{}**", len_str)
            } else {
                len_str
            };

            let linear_info = format!(
                "{} ({})",
                BenchmarkResult::format_throughput(WRITE_COUNT, result.linear_read_time),
                BenchmarkResult::format_bandwidth(write_bytes, result.linear_read_time)
            );
            let linear_val = if result.linear_read_time == best_linear {
                format!("**{}**", linear_info)
            } else {
                linear_info
            };

            let random_info = format!(
                "{} ({})",
                BenchmarkResult::format_throughput(
                    RANDOM_READ_COUNT as u64,
                    result.random_read_time
                ),
                BenchmarkResult::format_bandwidth(random_bytes, result.random_read_time)
            );
            let random_val = if result.random_read_time == best_random {
                format!("**{}**", random_info)
            } else {
                random_info
            };

            let disk_str = BenchmarkResult::format_size(result.disk_size);
            let disk_val = if result.disk_size == best_disk {
                format!("**{}**", disk_str)
            } else {
                disk_str
            };

            writeln!(
                file,
                "| {} | {} | {} | {} | {} | {} | {} |",
                result.name, open_val, write_val, len_val, linear_val, random_val, disk_val,
            )?;
        }

        writeln!(file)?;
        writeln!(file, "## Run")?;
        writeln!(file)?;
        writeln!(file, "```bash")?;
        writeln!(file, "cargo run --release --bin vecdb_bench")?;
        writeln!(file, "```")?;

        Ok(())
    }
}
