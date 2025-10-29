use std::{path::Path, time::Duration};

// Use this as inspiration: https://github.com/marvin-j97/rust-storage-bench/blob/v1/src/db/mod.rs

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

    fn push_iteration(
        &mut self,
        times: (
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
        ),
    ) {
        self.open.push(times.0);
        self.len.push(times.1);
        self.linear.push(times.2);
        self.seq_2t.push(times.3);
        self.seq_4t.push(times.4);
        self.seq_8t.push(times.5);
        self.random.push(times.6);
        self.random_4t.push(times.7);
        self.random_8t.push(times.8);
        self.random_12t.push(times.9);
        self.random_16t.push(times.10);
    }

    fn to_result(&self, name: String, write_time: Duration, disk_size: u64) -> BenchmarkResult {
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
        }
    }
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

    fn run_iteration(
        &mut self,
        runner: &BenchmarkRunner,
        indices: &[u64],
        indices_4t: &[Vec<u64>],
        indices_8t: &[Vec<u64>],
        indices_12t: &[Vec<u64>],
        indices_16t: &[Vec<u64>],
    ) -> Result<()> {
        let result = runner.run_iteration::<DB>(
            indices,
            indices_4t,
            indices_8t,
            indices_12t,
            indices_16t,
        )?;
        self.times.push_iteration(result);
        Ok(())
    }

    fn to_result(&self, runner: &BenchmarkRunner) -> Result<BenchmarkResult> {
        let disk_size = runner.measure_disk_size::<DB>()?;
        Ok(self
            .times
            .to_result(DB::name().to_string(), self.write_time, disk_size))
    }

    fn cleanup(&self, runner: &BenchmarkRunner) -> Result<()> {
        runner.cleanup::<DB>()
    }
}

pub fn run() -> Result<()> {
    println!("VecDB Benchmark Suite");

    let base_path = Path::new("bench_data");
    if base_path.exists() {
        std::fs::remove_dir_all(base_path)?;
    }
    std::fs::create_dir_all(base_path)?;

    let runner = BenchmarkRunner::new(base_path);

    // Phase 1: Prepare all databases (write data)
    println!("\nPreparing databases:");

    let mut vecdb = DbBenchmark::<VecDbBench>::new(&runner)?;
    let mut vecdb_old = DbBenchmark::<VecDbOldBench>::new(&runner)?;
    let mut fjall2 = DbBenchmark::<Fjall2Bench>::new(&runner)?;
    let mut fjall3 = DbBenchmark::<Fjall3Bench>::new(&runner)?;
    let mut redb = DbBenchmark::<RedbBench>::new(&runner)?;
    let mut lmdb = DbBenchmark::<LmdbBench>::new(&runner)?;
    let mut rocksdb = DbBenchmark::<RocksDbBench>::new(&runner)?;

    // Generate random indices (same for all databases and iterations)
    let indices = BenchmarkRunner::generate_random_indices(WRITE_COUNT);
    let indices_4t =
        BenchmarkRunner::generate_indices_per_thread(WRITE_COUNT, 4, RANDOM_SEED + 1000);
    let indices_8t =
        BenchmarkRunner::generate_indices_per_thread(WRITE_COUNT, 8, RANDOM_SEED + 2000);
    let indices_12t =
        BenchmarkRunner::generate_indices_per_thread(WRITE_COUNT, 12, RANDOM_SEED + 3000);
    let indices_16t =
        BenchmarkRunner::generate_indices_per_thread(WRITE_COUNT, 16, RANDOM_SEED + 4000);

    // Phase 2: Run interleaved iterations
    println!("\nRunning iterations:");

    for i in 1..=NUM_ITERATIONS {
        print!("  Iteration {}/{} ... ", i, NUM_ITERATIONS);
        std::io::Write::flush(&mut std::io::stdout()).ok();

        vecdb.run_iteration(
            &runner,
            &indices,
            &indices_4t,
            &indices_8t,
            &indices_12t,
            &indices_16t,
        )?;
        vecdb_old.run_iteration(
            &runner,
            &indices,
            &indices_4t,
            &indices_8t,
            &indices_12t,
            &indices_16t,
        )?;
        fjall2.run_iteration(
            &runner,
            &indices,
            &indices_4t,
            &indices_8t,
            &indices_12t,
            &indices_16t,
        )?;
        fjall3.run_iteration(
            &runner,
            &indices,
            &indices_4t,
            &indices_8t,
            &indices_12t,
            &indices_16t,
        )?;
        redb.run_iteration(
            &runner,
            &indices,
            &indices_4t,
            &indices_8t,
            &indices_12t,
            &indices_16t,
        )?;
        lmdb.run_iteration(
            &runner,
            &indices,
            &indices_4t,
            &indices_8t,
            &indices_12t,
            &indices_16t,
        )?;
        rocksdb.run_iteration(
            &runner,
            &indices,
            &indices_4t,
            &indices_8t,
            &indices_12t,
            &indices_16t,
        )?;

        println!("done");
    }

    // Phase 3: Build results
    let results = vec![
        vecdb.to_result(&runner)?,
        vecdb_old.to_result(&runner)?,
        fjall2.to_result(&runner)?,
        fjall3.to_result(&runner)?,
        redb.to_result(&runner)?,
        lmdb.to_result(&runner)?,
        rocksdb.to_result(&runner)?,
    ];

    // Print summary
    println!();
    BenchmarkRunner::print_summary(&results);

    // Write README
    println!("\nWriting README.md...");
    BenchmarkRunner::write_readme(&results)?;
    println!("README.md updated!");

    // Cleanup
    vecdb.cleanup(&runner)?;
    vecdb_old.cleanup(&runner)?;
    fjall2.cleanup(&runner)?;
    fjall3.cleanup(&runner)?;
    redb.cleanup(&runner)?;
    lmdb.cleanup(&runner)?;
    rocksdb.cleanup(&runner)?;

    Ok(())
}

fn avg(durations: &[Duration]) -> Duration {
    if durations.is_empty() {
        return Duration::from_secs(0);
    }
    let total: Duration = durations.iter().sum();
    total / durations.len() as u32
}
