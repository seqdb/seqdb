# VecDB Benchmark

Benchmark comparing vecdb against popular embedded databases: fjall, redb, and lmdb.

**Test**: 100 million sequential u64 writes, linear reads, and 10 million random reads.

**Iterations**: 5 passes

## System Information

- **CPU**: Apple M3 Pro
- **CPU Cores**: 12
- **Total Memory**: 36.00 GB
- **OS**: Darwin 26.0.1

## Results

| Metric | vecdb | fjall2 | fjall3 | redb | lmdb | rocksdb |
|--------|--------|--------|--------|--------|--------|--------|
| **Open** | 1.28 ms | 35.37 ms | 6.35 ms | 7.11 ms | **0.96 ms** | 7.74 ms |
| **Write** | **922.30 M ops/s<br>6.87 GB/s<br>1.08 ns** | 16.03 M ops/s<br>122.29 MB/s<br>62.39 ns | 22.18 M ops/s<br>169.19 MB/s<br>45.09 ns | 1.15 M ops/s<br>8.80 MB/s<br>866.88 ns | 638.52 K ops/s<br>4.87 MB/s<br>1.57 µs | 2.17 M ops/s<br>16.59 MB/s<br>460.00 ns |
| **len()** | **0.00 ms** | 7.25 s | 7.07 s | 0.59 ms | 0.00 ms | 11.53 s |
| **Linear Read** | **835.98 M ops/s<br>6.23 GB/s<br>1.20 ns** | 14.07 M ops/s<br>107.38 MB/s<br>71.05 ns | 14.57 M ops/s<br>111.17 MB/s<br>68.63 ns | 7.64 M ops/s<br>58.29 MB/s<br>130.88 ns | 5.05 M ops/s<br>38.51 MB/s<br>198.10 ns | 8.77 M ops/s<br>66.94 MB/s<br>113.97 ns |
| **Random 1t** | **2.07 M ops/s<br>15.80 MB/s<br>483.02 ns** | 175.71 K ops/s<br>1.34 MB/s<br>5.69 µs | 437.81 K ops/s<br>3.34 MB/s<br>2.28 µs | 453.33 K ops/s<br>3.46 MB/s<br>2.21 µs | 975.20 K ops/s<br>7.44 MB/s<br>1.03 µs | 231.21 K ops/s<br>1.76 MB/s<br>4.33 µs |
| **Random 4t** | 2.98 M ops/s<br>22.72 MB/s<br>335.75 ns | 583.30 K ops/s<br>4.45 MB/s<br>1.71 µs | 1.46 M ops/s<br>11.16 MB/s<br>683.75 ns | 1.38 M ops/s<br>10.55 MB/s<br>723.02 ns | **3.72 M ops/s<br>28.35 MB/s<br>269.08 ns** | 820.81 K ops/s<br>6.26 MB/s<br>1.22 µs |
| **Random 8t** | 1.46 M ops/s<br>11.10 MB/s<br>687.23 ns | 759.42 K ops/s<br>5.79 MB/s<br>1.32 µs | 2.03 M ops/s<br>15.47 MB/s<br>493.10 ns | 1.60 M ops/s<br>12.22 MB/s<br>624.58 ns | **5.96 M ops/s<br>45.50 MB/s<br>167.68 ns** | 1.11 M ops/s<br>8.49 MB/s<br>899.12 ns |
| **Random 12t** | 1.27 M ops/s<br>9.67 MB/s<br>788.99 ns | 739.62 K ops/s<br>5.64 MB/s<br>1.35 µs | 2.34 M ops/s<br>17.82 MB/s<br>428.23 ns | 1.65 M ops/s<br>12.61 MB/s<br>605.21 ns | **7.42 M ops/s<br>56.60 MB/s<br>134.80 ns** | 1.18 M ops/s<br>9.00 MB/s<br>848.15 ns |
| **Random 16t** | 1.23 M ops/s<br>9.41 MB/s<br>811.03 ns | 723.25 K ops/s<br>5.52 MB/s<br>1.38 µs | 2.39 M ops/s<br>18.21 MB/s<br>418.88 ns | 1.65 M ops/s<br>12.60 MB/s<br>605.46 ns | **7.80 M ops/s<br>59.53 MB/s<br>128.15 ns** | 1.21 M ops/s<br>9.19 MB/s<br>829.76 ns |
| **Disk Size** | 1.00 GB | **916.32 MB** | 1.50 GB | 3.03 GB | 3.58 GB | 1.03 GB |

## Run

```bash
cargo run --release --bin vecdb_bench
```
