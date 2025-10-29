# VecDB Benchmark

Benchmark comparing vecdb against popular embedded databases: fjall, redb, and lmdb.

**Test**: 10 million sequential u64 writes, linear reads, and 1 million random reads.

**Iterations**: 3 passes

## System Information

- **CPU**: Apple M3 Pro
- **CPU Cores**: 12
- **Total Memory**: 36.00 GB
- **OS**: Darwin 26.0.1

## Results

| Database | Open | Write | len() | Linear Read | Random 1t | Random 4t | Random 8t | Random 12t | Random 16t | Disk Size |
|----------|------|-------|-------|-------------|-----------|-----------|-----------|------------|------------|-----------|
| vecdb | 1.71 ms | **418.35 M ops/s<br>3.12 GB/s<br>2.39 ns** | 0.01 ms | **716.72 M ops/s<br>5.34 GB/s<br>1.40 ns** | **2.17 M ops/s<br>16.52 MB/s<br>461.80 ns** | 2.94 M ops/s<br>22.46 MB/s<br>339.71 ns | 1.50 M ops/s<br>11.48 MB/s<br>664.64 ns | 1.34 M ops/s<br>10.24 MB/s<br>744.77 ns | 1.33 M ops/s<br>10.11 MB/s<br>754.44 ns | 128.00 MB |
| fjall2 | 13.78 ms | 17.91 M ops/s<br>136.67 MB/s<br>55.82 ns | 723.51 ms | 14.29 M ops/s<br>109.00 MB/s<br>69.99 ns | 231.29 K ops/s<br>1.76 MB/s<br>4.32 µs | 726.24 K ops/s<br>5.54 MB/s<br>1.38 µs | 593.48 K ops/s<br>4.53 MB/s<br>1.68 µs | 457.33 K ops/s<br>3.49 MB/s<br>2.19 µs | 361.13 K ops/s<br>2.76 MB/s<br>2.77 µs | **91.55 MB** |
| fjall3 | 4.66 ms | 20.20 M ops/s<br>154.13 MB/s<br>49.50 ns | 727.41 ms | 14.51 M ops/s<br>110.70 MB/s<br>68.92 ns | 761.96 K ops/s<br>5.81 MB/s<br>1.31 µs | 2.51 M ops/s<br>19.15 MB/s<br>398.50 ns | 3.34 M ops/s<br>25.48 MB/s<br>299.48 ns | 3.84 M ops/s<br>29.26 MB/s<br>260.73 ns | 3.83 M ops/s<br>29.23 MB/s<br>261.00 ns | 153.26 MB |
| redb | 6.68 ms | 1.26 M ops/s<br>9.58 MB/s<br>796.06 ns | 0.41 ms | 4.14 M ops/s<br>31.59 MB/s<br>241.52 ns | 2.06 M ops/s<br>15.75 MB/s<br>484.49 ns | **7.07 M ops/s<br>53.92 MB/s<br>141.48 ns** | **8.33 M ops/s<br>63.59 MB/s<br>119.98 ns** | 8.28 M ops/s<br>63.18 MB/s<br>120.76 ns | 8.15 M ops/s<br>62.20 MB/s<br>122.67 ns | 514.00 MB |
| lmdb | **0.79 ms** | 1.45 M ops/s<br>11.08 MB/s<br>688.32 ns | **0.00 ms** | 6.14 M ops/s<br>46.86 MB/s<br>162.82 ns | 1.28 M ops/s<br>9.77 MB/s<br>780.64 ns | 4.64 M ops/s<br>35.36 MB/s<br>215.74 ns | 7.57 M ops/s<br>57.78 MB/s<br>132.03 ns | **10.26 M ops/s<br>78.26 MB/s<br>97.49 ns** | **9.78 M ops/s<br>74.63 MB/s<br>102.23 ns** | 367.13 MB |
| rocksdb | 3.09 ms | 2.29 M ops/s<br>17.44 MB/s<br>437.41 ns | 1.03 s | 9.93 M ops/s<br>75.79 MB/s<br>100.67 ns | 168.96 K ops/s<br>1.29 MB/s<br>5.92 µs | 524.92 K ops/s<br>4.00 MB/s<br>1.91 µs | 659.03 K ops/s<br>5.03 MB/s<br>1.52 µs | 635.71 K ops/s<br>4.85 MB/s<br>1.57 µs | 657.65 K ops/s<br>5.02 MB/s<br>1.52 µs | 243.80 MB |

## Run

```bash
cargo run --release --bin vecdb_bench
```
