use vecdb_bench::{BenchConfig, Database, run};

fn main() {
    let configs = vec![
        BenchConfig::default(),
        BenchConfig {
            write_count: 100_000_000,
            databases: vec![
                Database::VecDb,
                Database::VecDbOld,
                Database::Fjall2,
                Database::Fjall3,
                Database::Redb,
                Database::Lmdb,
            ],
            ..Default::default()
        },
        BenchConfig {
            write_count: 200_000_000,
            databases: vec![
                Database::VecDb,
                Database::VecDbOld,
                Database::Fjall2,
                Database::Fjall3,
                Database::Redb,
                Database::Lmdb,
            ],
            ..Default::default()
        },
    ];
    run(&configs).unwrap();
}
