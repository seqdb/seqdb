use vecdb_bench::{BenchConfig, Database, run};

fn main() {
    // Run with default configuration
    let configs = vec![
        BenchConfig::default(),
        BenchConfig {
            databases: vec![Database::VecDb, Database::VecDbOld],
            write_count: 1_000_000_000,
            random_seed: 21,
            ..Default::default()
        },
        BenchConfig {
            databases: vec![Database::VecDb, Database::VecDbOld],
            write_count: 10_000_000_000,
            random_seed: 128,
            ..Default::default()
        },
    ];
    run(&configs).unwrap();
}
