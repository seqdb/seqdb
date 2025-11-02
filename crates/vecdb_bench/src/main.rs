use vecdb_bench::{BenchConfig, Database, run};

fn main() {
    let configs = vec![
        BenchConfig::default(),
        BenchConfig {
            write_count: 100_000_000,
            databases: vec![
                Database::VecDbCompressed,
                Database::VecDbRaw,
                Database::Fjall3,
                Database::Fjall2,
                Database::Redb,
                Database::Lmdb,
            ],
            ..Default::default()
        },
        BenchConfig {
            write_count: 1_000_000_000,
            databases: vec![
                Database::VecDbRawOld,
                Database::VecDbRaw,
                Database::VecDbCompressed,
            ],
            ..Default::default()
        },
    ];
    run(&configs).unwrap();
}
