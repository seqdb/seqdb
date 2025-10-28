# VecDB Benchmark

Benchmark comparing vecdb against popular embedded databases: fjall, redb, and lmdb.

**Test**: 100 million sequential u64 writes, linear reads, and 10 million random reads.

## Results

| Database | Open | Write | len() | Linear Read | Random Read | Disk Size |
|----------|------|-------|-------|-------------|-------------|-----------|
| vecdb | 1.69 ms | **670.37 M ops/s (4.99 GB/s)** | **0.00 ms** | **834.01 M ops/s (6.21 GB/s)** | **1.99 M ops/s (15.16 MB/s)** | **1.00 GB** |
| fjall2 | 61.69 ms | 1.10 M ops/s (8.41 MB/s) | 7.49 s | 13.85 M ops/s (105.67 MB/s) | 162.22 K ops/s (1.24 MB/s) | 1.00 GB |
| redb | 8.29 ms | 1.14 M ops/s (8.70 MB/s) | 0.65 ms | 7.61 M ops/s (58.04 MB/s) | 426.55 K ops/s (3.25 MB/s) | 3.03 GB |
| lmdb | **0.83 ms** | 599.23 K ops/s (4.57 MB/s) | 0.00 ms | 6.30 M ops/s (48.04 MB/s) | 953.09 K ops/s (7.27 MB/s) | 3.58 GB |

## Run

```bash
cargo run --release --bin vecdb_bench
```
