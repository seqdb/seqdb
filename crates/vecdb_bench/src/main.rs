use vecdb_bench::{BenchConfig, Database, run};

fn main() {
    // Run with default configuration
    let configs = vec![
        BenchConfig::default(),
        BenchConfig {
            write_count: 100_000_000,
            random_read_count: 10_000_000,
            databases: vec![
                Database::VecDb,
                Database::VecDbOld,
                Database::Fjall2,
                Database::Fjall3,
                Database::Redb,
                Database::Lmdb,
                // Database::RocksDb,
            ],
            ..Default::default()
        },
        // BenchConfig::default(),
        // BenchConfig::default(),
        // BenchConfig {
        //     write_count: 1_000_000_000,
        //     random_read_count: 100_000_000,
        //     ..Default::default()
        // },
        // BenchConfig {
        //     write_count: 1_000_000_000,
        //     random_read_count: 100_000_000,
        //     ..Default::default()
        // },
        // BenchConfig {
        //     write_count: 1_000_000_000,
        //     random_read_count: 100_000_000,
        //     ..Default::default()
        // },
        // BenchConfig {
        //     databases: vec![Database::VecDb, Database::VecDbOld],
        //     write_count: 1_000_000_000,
        //     random_seed: 21,
        //     ..Default::default()
        // },
        // BenchConfig {
        //     databases: vec![Database::VecDb, Database::VecDbOld],
        //     write_count: 10_000_000_000,
        //     random_seed: 128,
        //     ..Default::default()
        // },
    ];
    run(&configs).unwrap();
}
