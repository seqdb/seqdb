use std::{fs, time::Duration};

use anyhow::Result;

mod database;
mod fjall2_impl;
mod fjall3_impl;
mod lmdb_impl;
mod redb_impl;
mod rocksdb_impl;
mod runner;
mod vecdb_impl;
mod vecdb_old_impl;

use database::DatabaseBenchmark;
use fjall2_impl::*;
use fjall3_impl::*;
use lmdb_impl::*;
use redb_impl::*;
use rocksdb_impl::*;
use runner::*;
pub use runner::{BenchConfig, Database};
use vecdb_impl::*;
use vecdb_old_impl::*;

struct AccumulatedTimes {
    open: Vec<Duration>,
    len: Vec<Duration>,
    linear: Vec<Duration>,
    seq_2t: Vec<Duration>,
    seq_4t: Vec<Duration>,
    seq_8t: Vec<Duration>,
    random: Vec<Duration>,
    random_4t: Vec<Duration>,
    random_8t: Vec<Duration>,
    random_12t: Vec<Duration>,
    random_16t: Vec<Duration>,
}

impl AccumulatedTimes {
    fn new() -> Self {
        Self {
            open: Vec::new(),
            len: Vec::new(),
            linear: Vec::new(),
            seq_2t: Vec::new(),
            seq_4t: Vec::new(),
            seq_8t: Vec::new(),
            random: Vec::new(),
            random_4t: Vec::new(),
            random_8t: Vec::new(),
            random_12t: Vec::new(),
            random_16t: Vec::new(),
        }
    }

    fn to_result(
        &self,
        name: String,
        write_time: Duration,
        disk_size: u64,
        config: BenchConfig,
        run_index: usize,
    ) -> BenchmarkResult {
        BenchmarkResult {
            name,
            open_time: avg(&self.open),
            write_time,
            len_time: avg(&self.len),
            linear_read_time: avg(&self.linear),
            seq_read_2t: avg(&self.seq_2t),
            seq_read_4t: avg(&self.seq_4t),
            seq_read_8t: avg(&self.seq_8t),
            random_read_time: avg(&self.random),
            random_read_4t: avg(&self.random_4t),
            random_read_8t: avg(&self.random_8t),
            random_read_12t: avg(&self.random_12t),
            random_read_16t: avg(&self.random_16t),
            disk_size,
            config,
            run_index,
        }
    }
}

trait DatabaseBenchmarkTrait {
    fn run_open(&mut self, runner: &BenchmarkRunner) -> Result<Duration>;
    fn run_len(&mut self, runner: &BenchmarkRunner) -> Result<Duration>;
    fn run_read_sequential(&mut self, runner: &BenchmarkRunner) -> Result<Duration>;
    fn run_read_seq_2t(&mut self, runner: &BenchmarkRunner) -> Result<Duration>;
    fn run_read_seq_4t(&mut self, runner: &BenchmarkRunner) -> Result<Duration>;
    fn run_read_seq_8t(&mut self, runner: &BenchmarkRunner) -> Result<Duration>;
    fn run_read_random(&mut self, runner: &BenchmarkRunner, indices: &[u64]) -> Result<Duration>;
    fn run_read_random_4t(
        &mut self,
        runner: &BenchmarkRunner,
        indices_4t: &[Vec<u64>],
    ) -> Result<Duration>;
    fn run_read_random_8t(
        &mut self,
        runner: &BenchmarkRunner,
        indices_8t: &[Vec<u64>],
    ) -> Result<Duration>;
    fn run_read_random_12t(
        &mut self,
        runner: &BenchmarkRunner,
        indices_12t: &[Vec<u64>],
    ) -> Result<Duration>;
    fn run_read_random_16t(
        &mut self,
        runner: &BenchmarkRunner,
        indices_16t: &[Vec<u64>],
    ) -> Result<Duration>;
    fn push_open(&mut self, duration: Duration);
    fn push_len(&mut self, duration: Duration);
    fn push_linear(&mut self, duration: Duration);
    fn push_seq_2t(&mut self, duration: Duration);
    fn push_seq_4t(&mut self, duration: Duration);
    fn push_seq_8t(&mut self, duration: Duration);
    fn push_random(&mut self, duration: Duration);
    fn push_random_4t(&mut self, duration: Duration);
    fn push_random_8t(&mut self, duration: Duration);
    fn push_random_12t(&mut self, duration: Duration);
    fn push_random_16t(&mut self, duration: Duration);
    fn to_result(&self, runner: &BenchmarkRunner, run_index: usize) -> Result<BenchmarkResult>;
    fn cleanup(&self, runner: &BenchmarkRunner) -> Result<()>;
}

struct DbBenchmark<DB: DatabaseBenchmark> {
    write_time: Duration,
    times: AccumulatedTimes,
    _phantom: std::marker::PhantomData<DB>,
}

impl<DB: DatabaseBenchmark> DbBenchmark<DB> {
    fn new(runner: &BenchmarkRunner) -> Result<Self> {
        let write_time = runner.prepare_database::<DB>()?;
        Ok(Self {
            write_time,
            times: AccumulatedTimes::new(),
            _phantom: std::marker::PhantomData,
        })
    }
}

impl<DB: DatabaseBenchmark + 'static> DatabaseBenchmarkTrait for DbBenchmark<DB> {
    fn run_open(&mut self, runner: &BenchmarkRunner) -> Result<Duration> {
        let name = DB::name();
        let path = runner.db_path(name);
        let start = std::time::Instant::now();
        let db = DB::open(&path)?;
        let duration = start.elapsed();
        drop(db);
        Ok(duration)
    }

    fn run_len(&mut self, runner: &BenchmarkRunner) -> Result<Duration> {
        let name = DB::name();
        let path = runner.db_path(name);
        let db = DB::open(&path)?;
        let start = std::time::Instant::now();
        let _len = db.len()?;
        let duration = start.elapsed();
        drop(db);
        Ok(duration)
    }

    fn run_read_sequential(&mut self, runner: &BenchmarkRunner) -> Result<Duration> {
        let name = DB::name();
        let path = runner.db_path(name);
        let db = DB::open(&path)?;
        let start = std::time::Instant::now();
        let _sum = db.read_sequential()?;
        let duration = start.elapsed();
        drop(db);
        Ok(duration)
    }

    fn run_read_seq_2t(&mut self, runner: &BenchmarkRunner) -> Result<Duration> {
        let name = DB::name();
        let path = runner.db_path(name);
        let db = DB::open(&path)?;
        let start = std::time::Instant::now();
        let _sum = db.read_sequential_threaded(2)?;
        let duration = start.elapsed();
        drop(db);
        Ok(duration)
    }

    fn run_read_seq_4t(&mut self, runner: &BenchmarkRunner) -> Result<Duration> {
        let name = DB::name();
        let path = runner.db_path(name);
        let db = DB::open(&path)?;
        let start = std::time::Instant::now();
        let _sum = db.read_sequential_threaded(4)?;
        let duration = start.elapsed();
        drop(db);
        Ok(duration)
    }

    fn run_read_seq_8t(&mut self, runner: &BenchmarkRunner) -> Result<Duration> {
        let name = DB::name();
        let path = runner.db_path(name);
        let db = DB::open(&path)?;
        let start = std::time::Instant::now();
        let _sum = db.read_sequential_threaded(8)?;
        let duration = start.elapsed();
        drop(db);
        Ok(duration)
    }

    fn run_read_random(&mut self, runner: &BenchmarkRunner, indices: &[u64]) -> Result<Duration> {
        let name = DB::name();
        let path = runner.db_path(name);
        let db = DB::open(&path)?;
        let start = std::time::Instant::now();
        let _sum = db.read_random(indices)?;
        let duration = start.elapsed();
        drop(db);
        Ok(duration)
    }

    fn run_read_random_4t(
        &mut self,
        runner: &BenchmarkRunner,
        indices_4t: &[Vec<u64>],
    ) -> Result<Duration> {
        let name = DB::name();
        let path = runner.db_path(name);
        let db = DB::open(&path)?;
        let start = std::time::Instant::now();
        let _sum = db.read_random_threaded(indices_4t)?;
        let duration = start.elapsed();
        drop(db);
        Ok(duration)
    }

    fn run_read_random_8t(
        &mut self,
        runner: &BenchmarkRunner,
        indices_8t: &[Vec<u64>],
    ) -> Result<Duration> {
        let name = DB::name();
        let path = runner.db_path(name);
        let db = DB::open(&path)?;
        let start = std::time::Instant::now();
        let _sum = db.read_random_threaded(indices_8t)?;
        let duration = start.elapsed();
        drop(db);
        Ok(duration)
    }

    fn run_read_random_12t(
        &mut self,
        runner: &BenchmarkRunner,
        indices_12t: &[Vec<u64>],
    ) -> Result<Duration> {
        let name = DB::name();
        let path = runner.db_path(name);
        let db = DB::open(&path)?;
        let start = std::time::Instant::now();
        let _sum = db.read_random_threaded(indices_12t)?;
        let duration = start.elapsed();
        drop(db);
        Ok(duration)
    }

    fn run_read_random_16t(
        &mut self,
        runner: &BenchmarkRunner,
        indices_16t: &[Vec<u64>],
    ) -> Result<Duration> {
        let name = DB::name();
        let path = runner.db_path(name);
        let db = DB::open(&path)?;
        let start = std::time::Instant::now();
        let _sum = db.read_random_threaded(indices_16t)?;
        let duration = start.elapsed();
        drop(db);
        Ok(duration)
    }

    fn to_result(&self, runner: &BenchmarkRunner, run_index: usize) -> Result<BenchmarkResult> {
        let disk_size = runner.measure_disk_size::<DB>()?;
        Ok(self.times.to_result(
            DB::name().to_string(),
            self.write_time,
            disk_size,
            runner.config().clone(),
            run_index,
        ))
    }

    fn push_open(&mut self, duration: Duration) {
        self.times.open.push(duration);
    }

    fn push_len(&mut self, duration: Duration) {
        self.times.len.push(duration);
    }

    fn push_linear(&mut self, duration: Duration) {
        self.times.linear.push(duration);
    }

    fn push_seq_2t(&mut self, duration: Duration) {
        self.times.seq_2t.push(duration);
    }

    fn push_seq_4t(&mut self, duration: Duration) {
        self.times.seq_4t.push(duration);
    }

    fn push_seq_8t(&mut self, duration: Duration) {
        self.times.seq_8t.push(duration);
    }

    fn push_random(&mut self, duration: Duration) {
        self.times.random.push(duration);
    }

    fn push_random_4t(&mut self, duration: Duration) {
        self.times.random_4t.push(duration);
    }

    fn push_random_8t(&mut self, duration: Duration) {
        self.times.random_8t.push(duration);
    }

    fn push_random_12t(&mut self, duration: Duration) {
        self.times.random_12t.push(duration);
    }

    fn push_random_16t(&mut self, duration: Duration) {
        self.times.random_16t.push(duration);
    }

    fn cleanup(&self, runner: &BenchmarkRunner) -> Result<()> {
        runner.cleanup::<DB>()
    }
}

pub fn run(configs: &[BenchConfig]) -> Result<()> {
    println!("VecDB Benchmark Suite");

    // Create bench_data in the crate directory (where Cargo.toml is)
    let crate_dir = env!("CARGO_MANIFEST_DIR");
    let base_path = std::path::PathBuf::from(crate_dir).join("bench_data");

    if base_path.exists() {
        std::fs::remove_dir_all(&base_path)?;
    }
    std::fs::create_dir_all(&base_path)?;

    let mut all_results = Vec::new();

    for (config_idx, config) in configs.iter().enumerate() {
        println!("\n=== Running Benchmark {} ===", config_idx + 1);
        println!(
            "Config: {} writes, {} random reads, {} iterations",
            config.write_count, config.random_read_count, config.num_iterations
        );

        let runner = BenchmarkRunner::new(&base_path, config.clone());

        // Generate random indices (same for all databases and iterations)
        let indices = runner.generate_random_indices();
        let indices_4t = runner.generate_indices_per_thread(4, config.random_seed + 1000);
        let indices_8t = runner.generate_indices_per_thread(8, config.random_seed + 2000);
        let indices_12t = runner.generate_indices_per_thread(12, config.random_seed + 3000);
        let indices_16t = runner.generate_indices_per_thread(16, config.random_seed + 4000);

        // Phase 1: Prepare all databases (write data)
        println!("\nPreparing databases:");
        let mut db_benchmarks: Vec<Box<dyn DatabaseBenchmarkTrait>> = Vec::new();

        for db in &config.databases {
            match db {
                Database::VecDb => {
                    db_benchmarks.push(Box::new(DbBenchmark::<VecDbBench>::new(&runner)?));
                }
                Database::VecDbOld => {
                    db_benchmarks.push(Box::new(DbBenchmark::<VecDbOldBench>::new(&runner)?));
                }
                Database::Fjall2 => {
                    db_benchmarks.push(Box::new(DbBenchmark::<Fjall2Bench>::new(&runner)?));
                }
                Database::Fjall3 => {
                    db_benchmarks.push(Box::new(DbBenchmark::<Fjall3Bench>::new(&runner)?));
                }
                Database::Redb => {
                    db_benchmarks.push(Box::new(DbBenchmark::<RedbBench>::new(&runner)?));
                }
                Database::Lmdb => {
                    db_benchmarks.push(Box::new(DbBenchmark::<LmdbBench>::new(&runner)?));
                }
                Database::RocksDb => {
                    db_benchmarks.push(Box::new(DbBenchmark::<RocksDbBench>::new(&runner)?));
                }
            }
        }

        // Phase 2: Run interleaved iterations
        println!("\nRunning iterations:");
        for i in 1..=config.num_iterations {
            println!("  Iteration {}/{}:", i, config.num_iterations);

            // open()
            print!("    open() ... ");
            std::io::Write::flush(&mut std::io::stdout()).ok();
            for db_bench in &mut db_benchmarks {
                let duration = db_bench.run_open(&runner)?;
                db_bench.push_open(duration);
            }
            println!("done");

            // len()
            print!("    len() ... ");
            std::io::Write::flush(&mut std::io::stdout()).ok();
            for db_bench in &mut db_benchmarks {
                let duration = db_bench.run_len(&runner)?;
                db_bench.push_len(duration);
            }
            println!("done");

            // read_sequential
            print!("    read_seq(1) ... ");
            std::io::Write::flush(&mut std::io::stdout()).ok();
            for db_bench in &mut db_benchmarks {
                let duration = db_bench.run_read_sequential(&runner)?;
                db_bench.push_linear(duration);
            }
            println!("done");

            // read_seq_2t
            print!("    read_seq(2) ... ");
            std::io::Write::flush(&mut std::io::stdout()).ok();
            for db_bench in &mut db_benchmarks {
                let duration = db_bench.run_read_seq_2t(&runner)?;
                db_bench.push_seq_2t(duration);
            }
            println!("done");

            // read_seq_4t
            print!("    read_seq(4) ... ");
            std::io::Write::flush(&mut std::io::stdout()).ok();
            for db_bench in &mut db_benchmarks {
                let duration = db_bench.run_read_seq_4t(&runner)?;
                db_bench.push_seq_4t(duration);
            }
            println!("done");

            // read_seq_8t
            print!("    read_seq(8) ... ");
            std::io::Write::flush(&mut std::io::stdout()).ok();
            for db_bench in &mut db_benchmarks {
                let duration = db_bench.run_read_seq_8t(&runner)?;
                db_bench.push_seq_8t(duration);
            }
            println!("done");

            // read_random
            print!("    read_rand(1) ... ");
            std::io::Write::flush(&mut std::io::stdout()).ok();
            for db_bench in &mut db_benchmarks {
                let duration = db_bench.run_read_random(&runner, &indices)?;
                db_bench.push_random(duration);
            }
            println!("done");

            // read_random_4t
            print!("    read_rand(4) ... ");
            std::io::Write::flush(&mut std::io::stdout()).ok();
            for db_bench in &mut db_benchmarks {
                let duration = db_bench.run_read_random_4t(&runner, &indices_4t)?;
                db_bench.push_random_4t(duration);
            }
            println!("done");

            // read_random_8t
            print!("    read_rand(8) ... ");
            std::io::Write::flush(&mut std::io::stdout()).ok();
            for db_bench in &mut db_benchmarks {
                let duration = db_bench.run_read_random_8t(&runner, &indices_8t)?;
                db_bench.push_random_8t(duration);
            }
            println!("done");

            // read_random_12t
            print!("    read_rand(12) ... ");
            std::io::Write::flush(&mut std::io::stdout()).ok();
            for db_bench in &mut db_benchmarks {
                let duration = db_bench.run_read_random_12t(&runner, &indices_12t)?;
                db_bench.push_random_12t(duration);
            }
            println!("done");

            // read_random_16t
            print!("    read_rand(16) ... ");
            std::io::Write::flush(&mut std::io::stdout()).ok();
            for db_bench in &mut db_benchmarks {
                let duration = db_bench.run_read_random_16t(&runner, &indices_16t)?;
                db_bench.push_random_16t(duration);
            }
            println!("done");
        }

        // Phase 3: Build results and cleanup
        for db_bench in db_benchmarks {
            let result = db_bench.to_result(&runner, config_idx)?;
            db_bench.cleanup(&runner)?;
            all_results.push(result);
        }

        // Cleanup after each benchmark config
        println!("\nCleaning up benchmark data...");
        if base_path.exists() {
            fs::remove_dir_all(&base_path)?;
            fs::create_dir_all(&base_path)?;
        }
    }

    // Print summary
    println!();
    BenchmarkRunner::print_summary(&all_results);

    // Write README
    println!();
    BenchmarkRunner::write_readme(&all_results)?;
    println!("README.md updated!");

    // Cleanup
    if base_path.exists() {
        std::fs::remove_dir_all(base_path)?;
    }

    Ok(())
}

fn avg(durations: &[Duration]) -> Duration {
    if durations.is_empty() {
        return Duration::from_secs(0);
    }
    let total: Duration = durations.iter().sum();
    total / durations.len() as u32
}
