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

use fjall2_impl::*;
use fjall3_impl::*;
use lmdb_impl::*;
use redb_impl::*;
use rocksdb_impl::*;
use runner::*;
use vecdb_impl::*;

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

    let vecdb_write_time = runner.prepare_database::<VecDbBench>()?;
    let fjall2_write_time = runner.prepare_database::<Fjall2Bench>()?;
    let fjall3_write_time = runner.prepare_database::<Fjall3Bench>()?;
    let redb_write_time = runner.prepare_database::<RedbBench>()?;
    let lmdb_write_time = runner.prepare_database::<LmdbBench>()?;
    let rocksdb_write_time = runner.prepare_database::<RocksDbBench>()?;

    // Generate random indices (same for all databases and iterations)
    let indices = runner::BenchmarkRunner::generate_random_indices(runner::WRITE_COUNT);
    let indices_4t = runner::BenchmarkRunner::generate_indices_per_thread(runner::WRITE_COUNT, 4, runner::RANDOM_SEED + 1000);
    let indices_8t = runner::BenchmarkRunner::generate_indices_per_thread(runner::WRITE_COUNT, 8, runner::RANDOM_SEED + 2000);
    let indices_12t = runner::BenchmarkRunner::generate_indices_per_thread(runner::WRITE_COUNT, 12, runner::RANDOM_SEED + 3000);
    let indices_16t = runner::BenchmarkRunner::generate_indices_per_thread(runner::WRITE_COUNT, 16, runner::RANDOM_SEED + 4000);

    // Phase 2: Run interleaved iterations
    println!("\nRunning iterations:");

    let mut vecdb_times = (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let mut fjall2_times = (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let mut fjall3_times = (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let mut redb_times = (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let mut lmdb_times = (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let mut rocksdb_times = (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());

    for i in 1..=runner::NUM_ITERATIONS {
        print!("  Iteration {}/{} ... ", i, runner::NUM_ITERATIONS);
        std::io::Write::flush(&mut std::io::stdout()).ok();

        // Run all databases in this iteration
        let (open, len, linear, random, r4t, r8t, r12t, r16t) = runner.run_iteration::<VecDbBench>(&indices, &indices_4t, &indices_8t, &indices_12t, &indices_16t)?;
        vecdb_times.0.push(open);
        vecdb_times.1.push(len);
        vecdb_times.2.push(linear);
        vecdb_times.3.push(random);
        vecdb_times.4.push(r4t);
        vecdb_times.5.push(r8t);
        vecdb_times.6.push(r12t);
        vecdb_times.7.push(r16t);

        let (open, len, linear, random, r4t, r8t, r12t, r16t) = runner.run_iteration::<Fjall2Bench>(&indices, &indices_4t, &indices_8t, &indices_12t, &indices_16t)?;
        fjall2_times.0.push(open);
        fjall2_times.1.push(len);
        fjall2_times.2.push(linear);
        fjall2_times.3.push(random);
        fjall2_times.4.push(r4t);
        fjall2_times.5.push(r8t);
        fjall2_times.6.push(r12t);
        fjall2_times.7.push(r16t);

        let (open, len, linear, random, r4t, r8t, r12t, r16t) = runner.run_iteration::<Fjall3Bench>(&indices, &indices_4t, &indices_8t, &indices_12t, &indices_16t)?;
        fjall3_times.0.push(open);
        fjall3_times.1.push(len);
        fjall3_times.2.push(linear);
        fjall3_times.3.push(random);
        fjall3_times.4.push(r4t);
        fjall3_times.5.push(r8t);
        fjall3_times.6.push(r12t);
        fjall3_times.7.push(r16t);

        let (open, len, linear, random, r4t, r8t, r12t, r16t) = runner.run_iteration::<RedbBench>(&indices, &indices_4t, &indices_8t, &indices_12t, &indices_16t)?;
        redb_times.0.push(open);
        redb_times.1.push(len);
        redb_times.2.push(linear);
        redb_times.3.push(random);
        redb_times.4.push(r4t);
        redb_times.5.push(r8t);
        redb_times.6.push(r12t);
        redb_times.7.push(r16t);

        let (open, len, linear, random, r4t, r8t, r12t, r16t) = runner.run_iteration::<LmdbBench>(&indices, &indices_4t, &indices_8t, &indices_12t, &indices_16t)?;
        lmdb_times.0.push(open);
        lmdb_times.1.push(len);
        lmdb_times.2.push(linear);
        lmdb_times.3.push(random);
        lmdb_times.4.push(r4t);
        lmdb_times.5.push(r8t);
        lmdb_times.6.push(r12t);
        lmdb_times.7.push(r16t);

        let (open, len, linear, random, r4t, r8t, r12t, r16t) = runner.run_iteration::<RocksDbBench>(&indices, &indices_4t, &indices_8t, &indices_12t, &indices_16t)?;
        rocksdb_times.0.push(open);
        rocksdb_times.1.push(len);
        rocksdb_times.2.push(linear);
        rocksdb_times.3.push(random);
        rocksdb_times.4.push(r4t);
        rocksdb_times.5.push(r8t);
        rocksdb_times.6.push(r12t);
        rocksdb_times.7.push(r16t);

        println!("done");
    }

    // Phase 3: Measure disk sizes
    let vecdb_disk = runner.measure_disk_size::<VecDbBench>()?;
    let fjall2_disk = runner.measure_disk_size::<Fjall2Bench>()?;
    let fjall3_disk = runner.measure_disk_size::<Fjall3Bench>()?;
    let redb_disk = runner.measure_disk_size::<RedbBench>()?;
    let lmdb_disk = runner.measure_disk_size::<LmdbBench>()?;
    let rocksdb_disk = runner.measure_disk_size::<RocksDbBench>()?;

    // Build results
    let results = vec![
        BenchmarkResult {
            name: "vecdb".to_string(),
            open_time: avg(&vecdb_times.0),
            write_time: vecdb_write_time,
            len_time: avg(&vecdb_times.1),
            linear_read_time: avg(&vecdb_times.2),
            random_read_time: avg(&vecdb_times.3),
            random_read_4t: avg(&vecdb_times.4),
            random_read_8t: avg(&vecdb_times.5),
            random_read_12t: avg(&vecdb_times.6),
            random_read_16t: avg(&vecdb_times.7),
            disk_size: vecdb_disk,
        },
        BenchmarkResult {
            name: "fjall2".to_string(),
            open_time: avg(&fjall2_times.0),
            write_time: fjall2_write_time,
            len_time: avg(&fjall2_times.1),
            linear_read_time: avg(&fjall2_times.2),
            random_read_time: avg(&fjall2_times.3),
            random_read_4t: avg(&fjall2_times.4),
            random_read_8t: avg(&fjall2_times.5),
            random_read_12t: avg(&fjall2_times.6),
            random_read_16t: avg(&fjall2_times.7),
            disk_size: fjall2_disk,
        },
        BenchmarkResult {
            name: "fjall3".to_string(),
            open_time: avg(&fjall3_times.0),
            write_time: fjall3_write_time,
            len_time: avg(&fjall3_times.1),
            linear_read_time: avg(&fjall3_times.2),
            random_read_time: avg(&fjall3_times.3),
            random_read_4t: avg(&fjall3_times.4),
            random_read_8t: avg(&fjall3_times.5),
            random_read_12t: avg(&fjall3_times.6),
            random_read_16t: avg(&fjall3_times.7),
            disk_size: fjall3_disk,
        },
        BenchmarkResult {
            name: "redb".to_string(),
            open_time: avg(&redb_times.0),
            write_time: redb_write_time,
            len_time: avg(&redb_times.1),
            linear_read_time: avg(&redb_times.2),
            random_read_time: avg(&redb_times.3),
            random_read_4t: avg(&redb_times.4),
            random_read_8t: avg(&redb_times.5),
            random_read_12t: avg(&redb_times.6),
            random_read_16t: avg(&redb_times.7),
            disk_size: redb_disk,
        },
        BenchmarkResult {
            name: "lmdb".to_string(),
            open_time: avg(&lmdb_times.0),
            write_time: lmdb_write_time,
            len_time: avg(&lmdb_times.1),
            linear_read_time: avg(&lmdb_times.2),
            random_read_time: avg(&lmdb_times.3),
            random_read_4t: avg(&lmdb_times.4),
            random_read_8t: avg(&lmdb_times.5),
            random_read_12t: avg(&lmdb_times.6),
            random_read_16t: avg(&lmdb_times.7),
            disk_size: lmdb_disk,
        },
        BenchmarkResult {
            name: "rocksdb".to_string(),
            open_time: avg(&rocksdb_times.0),
            write_time: rocksdb_write_time,
            len_time: avg(&rocksdb_times.1),
            linear_read_time: avg(&rocksdb_times.2),
            random_read_time: avg(&rocksdb_times.3),
            random_read_4t: avg(&rocksdb_times.4),
            random_read_8t: avg(&rocksdb_times.5),
            random_read_12t: avg(&rocksdb_times.6),
            random_read_16t: avg(&rocksdb_times.7),
            disk_size: rocksdb_disk,
        },
    ];

    // Print summary
    println!();
    BenchmarkRunner::print_summary(&results);

    // Write README
    println!("\nWriting README.md...");
    BenchmarkRunner::write_readme(&results)?;
    println!("README.md updated!");

    // Cleanup
    runner.cleanup::<VecDbBench>()?;
    runner.cleanup::<Fjall2Bench>()?;
    runner.cleanup::<Fjall3Bench>()?;
    runner.cleanup::<RedbBench>()?;
    runner.cleanup::<LmdbBench>()?;
    runner.cleanup::<RocksDbBench>()?;

    Ok(())
}

fn avg(durations: &[Duration]) -> Duration {
    if durations.is_empty() {
        return Duration::from_secs(0);
    }
    let total: Duration = durations.iter().sum();
    total / durations.len() as u32
}
