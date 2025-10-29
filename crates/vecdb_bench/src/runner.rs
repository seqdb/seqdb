use crate::database::DatabaseBenchmark;
use anyhow::Result;
use rand::{Rng, SeedableRng};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use sysinfo::System;

pub const WRITE_COUNT: u64 = 100_000_000;
pub const RANDOM_READ_COUNT: usize = 10_000_000;
pub const RANDOM_SEED: u64 = 42;
pub const NUM_ITERATIONS: usize = 3;

pub struct BenchmarkResult {
    pub name: String,
    pub open_time: Duration,
    pub write_time: Duration,
    pub len_time: Duration,
    pub linear_read_time: Duration,
    pub seq_read_2t: Duration,
    pub seq_read_4t: Duration,
    pub seq_read_8t: Duration,
    pub random_read_time: Duration,
    pub random_read_4t: Duration,
    pub random_read_8t: Duration,
    pub random_read_12t: Duration,
    pub random_read_16t: Duration,
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

    fn format_latency(ops: u64, duration: Duration) -> String {
        let secs = duration.as_secs_f64();
        if secs < 0.000001 {
            return String::from("N/A");
        }
        let latency_secs = secs / ops as f64;
        let latency_ns = latency_secs * 1_000_000_000.0;
        if latency_ns >= 1_000_000.0 {
            format!("{:.2} ms", latency_ns / 1_000_000.0)
        } else if latency_ns >= 1_000.0 {
            format!("{:.2} µs", latency_ns / 1_000.0)
        } else {
            format!("{:.2} ns", latency_ns)
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

    pub fn generate_indices_per_thread(
        count: u64,
        num_threads: usize,
        base_seed: u64,
    ) -> Vec<Vec<u64>> {
        (0..num_threads)
            .map(|thread_id| {
                let seed = base_seed.wrapping_add(thread_id as u64);
                let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
                let per_thread = RANDOM_READ_COUNT / num_threads;
                (0..per_thread)
                    .map(|_| rng.random_range(0..count))
                    .collect()
            })
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

    #[allow(clippy::type_complexity)]
    pub fn run_iteration<DB: DatabaseBenchmark>(
        &self,
        indices: &[u64],
        indices_4t: &[Vec<u64>],
        indices_8t: &[Vec<u64>],
        indices_12t: &[Vec<u64>],
        indices_16t: &[Vec<u64>],
    ) -> Result<(
        Duration,
        Duration,
        Duration,
        Duration,
        Duration,
        Duration,
        Duration,
        Duration,
        Duration,
        Duration,
        Duration,
    )> {
        let name = DB::name();
        let path = self.db_path(name);

        // Open
        let start = Instant::now();
        println!("{name} open...");
        let db = DB::open(&path)?;
        let open_time = start.elapsed();

        // len()
        let start = Instant::now();
        println!("{name}: len...");
        let _len = db.len()?;
        let len_time = start.elapsed();

        // Linear reads
        let start = Instant::now();
        println!("{name}: read seq...");
        let _sum = db.read_sequential()?;
        let linear_read_time = start.elapsed();

        // Sequential reads (2 threads)
        let start = Instant::now();
        println!("{name}: read seq 2t...");
        let _sum = db.read_sequential_threaded(2)?;
        let seq_read_2t = start.elapsed();

        // Sequential reads (4 threads)
        let start = Instant::now();
        println!("{name}: read seq 4t...");
        let _sum = db.read_sequential_threaded(4)?;
        let seq_read_4t = start.elapsed();

        // Sequential reads (8 threads)
        let start = Instant::now();
        println!("{name}: read seq 8t...");
        let _sum = db.read_sequential_threaded(8)?;
        let seq_read_8t = start.elapsed();

        // Random reads (single-threaded)
        let start = Instant::now();
        println!("{name}: read rand 1t...");
        let _sum = db.read_random(indices)?;
        let random_read_time = start.elapsed();

        // Random reads (4 threads)
        let start = Instant::now();
        println!("{name}: read rand 4t...");
        let _sum = db.read_random_threaded(indices_4t)?;
        let random_read_4t = start.elapsed();

        // Random reads (8 threads)
        let start = Instant::now();
        println!("{name}: read rand 8t...");
        let _sum = db.read_random_threaded(indices_8t)?;
        let random_read_8t = start.elapsed();

        // Random reads (12 threads)
        let start = Instant::now();
        println!("{name}: read rand 12t...");
        let _sum = db.read_random_threaded(indices_12t)?;
        let random_read_12t = start.elapsed();

        // Random reads (16 threads)
        let start = Instant::now();
        println!("{name}: read rand 16t...");
        let _sum = db.read_random_threaded(indices_16t)?;
        let random_read_16t = start.elapsed();

        drop(db);

        Ok((
            open_time,
            len_time,
            linear_read_time,
            seq_read_2t,
            seq_read_4t,
            seq_read_8t,
            random_read_time,
            random_read_4t,
            random_read_8t,
            random_read_12t,
            random_read_16t,
        ))
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
            println!("  Write:");
            println!(
                "    {}",
                BenchmarkResult::format_throughput(WRITE_COUNT, result.write_time)
            );
            println!(
                "    {}",
                BenchmarkResult::format_bandwidth(write_bytes, result.write_time)
            );
            println!(
                "    {}",
                BenchmarkResult::format_latency(WRITE_COUNT, result.write_time)
            );
            println!(
                "  len():       {}",
                BenchmarkResult::format_duration(result.len_time)
            );
            println!("  Linear:");
            println!(
                "    {}",
                BenchmarkResult::format_throughput(WRITE_COUNT, result.linear_read_time)
            );
            println!(
                "    {}",
                BenchmarkResult::format_bandwidth(write_bytes, result.linear_read_time)
            );
            println!(
                "    {}",
                BenchmarkResult::format_latency(WRITE_COUNT, result.linear_read_time)
            );
            println!("  Seq 2t:");
            println!(
                "    {}",
                BenchmarkResult::format_throughput(WRITE_COUNT, result.seq_read_2t)
            );
            println!(
                "    {}",
                BenchmarkResult::format_bandwidth(write_bytes, result.seq_read_2t)
            );
            println!(
                "    {}",
                BenchmarkResult::format_latency(WRITE_COUNT, result.seq_read_2t)
            );
            println!("  Seq 4t:");
            println!(
                "    {}",
                BenchmarkResult::format_throughput(WRITE_COUNT, result.seq_read_4t)
            );
            println!(
                "    {}",
                BenchmarkResult::format_bandwidth(write_bytes, result.seq_read_4t)
            );
            println!(
                "    {}",
                BenchmarkResult::format_latency(WRITE_COUNT, result.seq_read_4t)
            );
            println!("  Seq 8t:");
            println!(
                "    {}",
                BenchmarkResult::format_throughput(WRITE_COUNT, result.seq_read_8t)
            );
            println!(
                "    {}",
                BenchmarkResult::format_bandwidth(write_bytes, result.seq_read_8t)
            );
            println!(
                "    {}",
                BenchmarkResult::format_latency(WRITE_COUNT, result.seq_read_8t)
            );
            println!("  Random 1t:");
            println!(
                "    {}",
                BenchmarkResult::format_throughput(
                    RANDOM_READ_COUNT as u64,
                    result.random_read_time
                )
            );
            println!(
                "    {}",
                BenchmarkResult::format_bandwidth(random_bytes, result.random_read_time)
            );
            println!(
                "    {}",
                BenchmarkResult::format_latency(RANDOM_READ_COUNT as u64, result.random_read_time)
            );
            println!("  Random 4t:");
            println!(
                "    {}",
                BenchmarkResult::format_throughput(RANDOM_READ_COUNT as u64, result.random_read_4t)
            );
            println!(
                "    {}",
                BenchmarkResult::format_bandwidth(random_bytes, result.random_read_4t)
            );
            println!(
                "    {}",
                BenchmarkResult::format_latency(RANDOM_READ_COUNT as u64, result.random_read_4t)
            );
            println!("  Random 8t:");
            println!(
                "    {}",
                BenchmarkResult::format_throughput(RANDOM_READ_COUNT as u64, result.random_read_8t)
            );
            println!(
                "    {}",
                BenchmarkResult::format_bandwidth(random_bytes, result.random_read_8t)
            );
            println!(
                "    {}",
                BenchmarkResult::format_latency(RANDOM_READ_COUNT as u64, result.random_read_8t)
            );
            println!("  Random 12t:");
            println!(
                "    {}",
                BenchmarkResult::format_throughput(
                    RANDOM_READ_COUNT as u64,
                    result.random_read_12t
                )
            );
            println!(
                "    {}",
                BenchmarkResult::format_bandwidth(random_bytes, result.random_read_12t)
            );
            println!(
                "    {}",
                BenchmarkResult::format_latency(RANDOM_READ_COUNT as u64, result.random_read_12t)
            );
            println!("  Random 16t:");
            println!(
                "    {}",
                BenchmarkResult::format_throughput(
                    RANDOM_READ_COUNT as u64,
                    result.random_read_16t
                )
            );
            println!(
                "    {}",
                BenchmarkResult::format_bandwidth(random_bytes, result.random_read_16t)
            );
            println!(
                "    {}",
                BenchmarkResult::format_latency(RANDOM_READ_COUNT as u64, result.random_read_16t)
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
        writeln!(
            file,
            "**Iterations**: {} pass{}",
            NUM_ITERATIONS,
            if NUM_ITERATIONS > 1 { "es" } else { "" }
        )?;
        writeln!(file)?;

        // Get system information
        let mut sys = System::new_all();
        sys.refresh_all();

        writeln!(file, "## System Information")?;
        writeln!(file)?;

        if let Some(cpu_brand) = sys.cpus().first().map(|cpu| cpu.brand()) {
            writeln!(file, "- **CPU**: {}", cpu_brand)?;
        }
        writeln!(file, "- **CPU Cores**: {}", sys.cpus().len())?;
        writeln!(
            file,
            "- **Total Memory**: {:.2} GB",
            sys.total_memory() as f64 / (1024.0 * 1024.0 * 1024.0)
        )?;

        if let Some(os_name) = System::name() {
            write!(file, "- **OS**: {}", os_name)?;
            if let Some(os_version) = System::os_version() {
                write!(file, " {}", os_version)?;
            }
            writeln!(file)?;
        }

        writeln!(file)?;
        writeln!(file, "## Results")?;
        writeln!(file)?;

        // Find best values (min for time/size, max for throughput)
        let best_open = results.iter().map(|r| r.open_time).min().unwrap();
        let best_write = results.iter().map(|r| r.write_time).min().unwrap();
        let best_len = results.iter().map(|r| r.len_time).min().unwrap();
        let best_linear = results.iter().map(|r| r.linear_read_time).min().unwrap();
        let best_seq_2t = results.iter().map(|r| r.seq_read_2t).min().unwrap();
        let best_seq_4t = results.iter().map(|r| r.seq_read_4t).min().unwrap();
        let best_seq_8t = results.iter().map(|r| r.seq_read_8t).min().unwrap();
        let best_random = results.iter().map(|r| r.random_read_time).min().unwrap();
        let best_random_4t = results.iter().map(|r| r.random_read_4t).min().unwrap();
        let best_random_8t = results.iter().map(|r| r.random_read_8t).min().unwrap();
        let best_random_12t = results.iter().map(|r| r.random_read_12t).min().unwrap();
        let best_random_16t = results.iter().map(|r| r.random_read_16t).min().unwrap();
        let best_disk = results.iter().map(|r| r.disk_size).min().unwrap();

        let write_bytes = WRITE_COUNT * 8;
        let random_bytes = RANDOM_READ_COUNT as u64 * 8;

        // Results table (transposed: metrics as rows, databases as columns)
        // Header row
        write!(file, "| Metric |")?;
        for result in results {
            write!(file, " {} |", result.name)?;
        }
        writeln!(file)?;

        // Separator row
        write!(file, "|--------|")?;
        for _ in results {
            write!(file, "--------|")?;
        }
        writeln!(file)?;

        // Open row
        write!(file, "| **Open** |")?;
        for result in results {
            let open_str = BenchmarkResult::format_duration(result.open_time);
            if result.open_time == best_open {
                write!(file, " **{}** |", open_str)?;
            } else {
                write!(file, " {} |", open_str)?;
            }
        }
        writeln!(file)?;

        // Write row
        write!(file, "| **Write** |")?;
        for result in results {
            let info = format!(
                "{}<br>{}<br>{}",
                BenchmarkResult::format_throughput(WRITE_COUNT, result.write_time),
                BenchmarkResult::format_bandwidth(write_bytes, result.write_time),
                BenchmarkResult::format_latency(WRITE_COUNT, result.write_time)
            );
            if result.write_time == best_write {
                write!(file, " **{}** |", info)?;
            } else {
                write!(file, " {} |", info)?;
            }
        }
        writeln!(file)?;

        // len() row
        write!(file, "| **len()** |")?;
        for result in results {
            let len_str = BenchmarkResult::format_duration(result.len_time);
            if result.len_time == best_len {
                write!(file, " **{}** |", len_str)?;
            } else {
                write!(file, " {} |", len_str)?;
            }
        }
        writeln!(file)?;

        // Linear Read row
        write!(file, "| **Linear Read** |")?;
        for result in results {
            let info = format!(
                "{}<br>{}<br>{}",
                BenchmarkResult::format_throughput(WRITE_COUNT, result.linear_read_time),
                BenchmarkResult::format_bandwidth(write_bytes, result.linear_read_time),
                BenchmarkResult::format_latency(WRITE_COUNT, result.linear_read_time)
            );
            if result.linear_read_time == best_linear {
                write!(file, " **{}** |", info)?;
            } else {
                write!(file, " {} |", info)?;
            }
        }
        writeln!(file)?;

        // Seq Read 2t row
        write!(file, "| **Seq 2t** |")?;
        for result in results {
            let info = format!(
                "{}<br>{}<br>{}",
                BenchmarkResult::format_throughput(WRITE_COUNT, result.seq_read_2t),
                BenchmarkResult::format_bandwidth(write_bytes, result.seq_read_2t),
                BenchmarkResult::format_latency(WRITE_COUNT, result.seq_read_2t)
            );
            if result.seq_read_2t == best_seq_2t {
                write!(file, " **{}** |", info)?;
            } else {
                write!(file, " {} |", info)?;
            }
        }
        writeln!(file)?;

        // Seq Read 4t row
        write!(file, "| **Seq 4t** |")?;
        for result in results {
            let info = format!(
                "{}<br>{}<br>{}",
                BenchmarkResult::format_throughput(WRITE_COUNT, result.seq_read_4t),
                BenchmarkResult::format_bandwidth(write_bytes, result.seq_read_4t),
                BenchmarkResult::format_latency(WRITE_COUNT, result.seq_read_4t)
            );
            if result.seq_read_4t == best_seq_4t {
                write!(file, " **{}** |", info)?;
            } else {
                write!(file, " {} |", info)?;
            }
        }
        writeln!(file)?;

        // Seq Read 8t row
        write!(file, "| **Seq 8t** |")?;
        for result in results {
            let info = format!(
                "{}<br>{}<br>{}",
                BenchmarkResult::format_throughput(WRITE_COUNT, result.seq_read_8t),
                BenchmarkResult::format_bandwidth(write_bytes, result.seq_read_8t),
                BenchmarkResult::format_latency(WRITE_COUNT, result.seq_read_8t)
            );
            if result.seq_read_8t == best_seq_8t {
                write!(file, " **{}** |", info)?;
            } else {
                write!(file, " {} |", info)?;
            }
        }
        writeln!(file)?;

        // Random Read 1t row
        write!(file, "| **Random 1t** |")?;
        for result in results {
            let info = format!(
                "{}<br>{}<br>{}",
                BenchmarkResult::format_throughput(
                    RANDOM_READ_COUNT as u64,
                    result.random_read_time
                ),
                BenchmarkResult::format_bandwidth(random_bytes, result.random_read_time),
                BenchmarkResult::format_latency(RANDOM_READ_COUNT as u64, result.random_read_time)
            );
            if result.random_read_time == best_random {
                write!(file, " **{}** |", info)?;
            } else {
                write!(file, " {} |", info)?;
            }
        }
        writeln!(file)?;

        // Random Read 4t row
        write!(file, "| **Random 4t** |")?;
        for result in results {
            let info = format!(
                "{}<br>{}<br>{}",
                BenchmarkResult::format_throughput(RANDOM_READ_COUNT as u64, result.random_read_4t),
                BenchmarkResult::format_bandwidth(random_bytes, result.random_read_4t),
                BenchmarkResult::format_latency(RANDOM_READ_COUNT as u64, result.random_read_4t)
            );
            if result.random_read_4t == best_random_4t {
                write!(file, " **{}** |", info)?;
            } else {
                write!(file, " {} |", info)?;
            }
        }
        writeln!(file)?;

        // Random Read 8t row
        write!(file, "| **Random 8t** |")?;
        for result in results {
            let info = format!(
                "{}<br>{}<br>{}",
                BenchmarkResult::format_throughput(RANDOM_READ_COUNT as u64, result.random_read_8t),
                BenchmarkResult::format_bandwidth(random_bytes, result.random_read_8t),
                BenchmarkResult::format_latency(RANDOM_READ_COUNT as u64, result.random_read_8t)
            );
            if result.random_read_8t == best_random_8t {
                write!(file, " **{}** |", info)?;
            } else {
                write!(file, " {} |", info)?;
            }
        }
        writeln!(file)?;

        // Random Read 12t row
        write!(file, "| **Random 12t** |")?;
        for result in results {
            let info = format!(
                "{}<br>{}<br>{}",
                BenchmarkResult::format_throughput(
                    RANDOM_READ_COUNT as u64,
                    result.random_read_12t
                ),
                BenchmarkResult::format_bandwidth(random_bytes, result.random_read_12t),
                BenchmarkResult::format_latency(RANDOM_READ_COUNT as u64, result.random_read_12t)
            );
            if result.random_read_12t == best_random_12t {
                write!(file, " **{}** |", info)?;
            } else {
                write!(file, " {} |", info)?;
            }
        }
        writeln!(file)?;

        // Random Read 16t row
        write!(file, "| **Random 16t** |")?;
        for result in results {
            let info = format!(
                "{}<br>{}<br>{}",
                BenchmarkResult::format_throughput(
                    RANDOM_READ_COUNT as u64,
                    result.random_read_16t
                ),
                BenchmarkResult::format_bandwidth(random_bytes, result.random_read_16t),
                BenchmarkResult::format_latency(RANDOM_READ_COUNT as u64, result.random_read_16t)
            );
            if result.random_read_16t == best_random_16t {
                write!(file, " **{}** |", info)?;
            } else {
                write!(file, " {} |", info)?;
            }
        }
        writeln!(file)?;

        // Disk Size row
        write!(file, "| **Disk Size** |")?;
        for result in results {
            let disk_str = BenchmarkResult::format_size(result.disk_size);
            if result.disk_size == best_disk {
                write!(file, " **{}** |", disk_str)?;
            } else {
                write!(file, " {} |", disk_str)?;
            }
        }
        writeln!(file)?;

        writeln!(file)?;
        writeln!(file, "## Run")?;
        writeln!(file)?;
        writeln!(file, "```bash")?;
        writeln!(file, "cargo run --release --bin vecdb_bench")?;
        writeln!(file, "```")?;

        Ok(())
    }
}
