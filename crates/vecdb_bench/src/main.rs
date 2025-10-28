mod database;
mod fjall2_impl;
// mod fjall3_impl;
mod lmdb_impl;
mod redb_impl;
mod runner;
mod vecdb_impl;

use anyhow::Result;
use runner::{BenchmarkResult, BenchmarkRunner};
use std::path::Path;
use std::time::Duration;

fn main() -> Result<()> {
    println!("VecDB Benchmark Suite");

    let base_path = Path::new("bench_data");
    if base_path.exists() {
        std::fs::remove_dir_all(base_path)?;
    }
    std::fs::create_dir_all(base_path)?;

    let runner = BenchmarkRunner::new(base_path);

    // Phase 1: Prepare all databases (write data)
    println!("\nPreparing databases:");

    let vecdb_write_time = runner.prepare_database::<vecdb_impl::VecDbBench>()?;
    let fjall_write_time = runner.prepare_database::<fjall2_impl::FjallBench>()?;
    let redb_write_time = runner.prepare_database::<redb_impl::RedbBench>()?;
    let lmdb_write_time = runner.prepare_database::<lmdb_impl::LmdbBench>()?;

    // Generate random indices (same for all databases and iterations)
    let indices = runner::BenchmarkRunner::generate_random_indices(runner::WRITE_COUNT);

    // Phase 2: Run interleaved iterations
    println!("\nRunning iterations:");

    let mut vecdb_times = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let mut fjall_times = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let mut redb_times = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let mut lmdb_times = (Vec::new(), Vec::new(), Vec::new(), Vec::new());

    for i in 1..=runner::NUM_ITERATIONS {
        print!("  Iteration {}/{} ... ", i, runner::NUM_ITERATIONS);
        std::io::Write::flush(&mut std::io::stdout()).ok();

        // Run all databases in this iteration
        let (open, len, linear, random) = runner.run_iteration::<vecdb_impl::VecDbBench>(&indices)?;
        vecdb_times.0.push(open);
        vecdb_times.1.push(len);
        vecdb_times.2.push(linear);
        vecdb_times.3.push(random);

        let (open, len, linear, random) = runner.run_iteration::<fjall2_impl::FjallBench>(&indices)?;
        fjall_times.0.push(open);
        fjall_times.1.push(len);
        fjall_times.2.push(linear);
        fjall_times.3.push(random);

        let (open, len, linear, random) = runner.run_iteration::<redb_impl::RedbBench>(&indices)?;
        redb_times.0.push(open);
        redb_times.1.push(len);
        redb_times.2.push(linear);
        redb_times.3.push(random);

        let (open, len, linear, random) = runner.run_iteration::<lmdb_impl::LmdbBench>(&indices)?;
        lmdb_times.0.push(open);
        lmdb_times.1.push(len);
        lmdb_times.2.push(linear);
        lmdb_times.3.push(random);

        println!("done");
    }

    // Phase 3: Measure disk sizes
    let vecdb_disk = runner.measure_disk_size::<vecdb_impl::VecDbBench>()?;
    let fjall_disk = runner.measure_disk_size::<fjall2_impl::FjallBench>()?;
    let redb_disk = runner.measure_disk_size::<redb_impl::RedbBench>()?;
    let lmdb_disk = runner.measure_disk_size::<lmdb_impl::LmdbBench>()?;

    // Build results
    let results = vec![
        BenchmarkResult {
            name: "vecdb".to_string(),
            open_time: avg(&vecdb_times.0),
            write_time: vecdb_write_time,
            len_time: avg(&vecdb_times.1),
            linear_read_time: avg(&vecdb_times.2),
            random_read_time: avg(&vecdb_times.3),
            disk_size: vecdb_disk,
        },
        BenchmarkResult {
            name: "fjall2".to_string(),
            open_time: avg(&fjall_times.0),
            write_time: fjall_write_time,
            len_time: avg(&fjall_times.1),
            linear_read_time: avg(&fjall_times.2),
            random_read_time: avg(&fjall_times.3),
            disk_size: fjall_disk,
        },
        BenchmarkResult {
            name: "redb".to_string(),
            open_time: avg(&redb_times.0),
            write_time: redb_write_time,
            len_time: avg(&redb_times.1),
            linear_read_time: avg(&redb_times.2),
            random_read_time: avg(&redb_times.3),
            disk_size: redb_disk,
        },
        BenchmarkResult {
            name: "lmdb".to_string(),
            open_time: avg(&lmdb_times.0),
            write_time: lmdb_write_time,
            len_time: avg(&lmdb_times.1),
            linear_read_time: avg(&lmdb_times.2),
            random_read_time: avg(&lmdb_times.3),
            disk_size: lmdb_disk,
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
    runner.cleanup::<vecdb_impl::VecDbBench>()?;
    runner.cleanup::<fjall2_impl::FjallBench>()?;
    runner.cleanup::<redb_impl::RedbBench>()?;
    runner.cleanup::<lmdb_impl::LmdbBench>()?;

    Ok(())
}

fn avg(durations: &[Duration]) -> Duration {
    if durations.is_empty() {
        return Duration::from_secs(0);
    }
    let total: Duration = durations.iter().sum();
    total / durations.len() as u32
}
