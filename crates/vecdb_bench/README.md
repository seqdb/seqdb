# VecDB Benchmark

Benchmark comparing vecdb against popular embedded databases: fjall, redb, and lmdb.

## Benchmark 1

**Test**: 10 million sequential u64 writes, linear reads, and 1% random reads.

**Iterations**: 10 passes

### Results

| Metric | vecdb_compressed | vecdb_raw | fjall3 | fjall2 | redb | lmdb | rocksdb |
|--------|--------|--------|--------|--------|--------|--------|--------|
| **Open** | 0.15 ms | **0.07 ms** | 1.02 s | 630.72 ms | 5.39 ms | 0.15 ms | 2.27 ms |
| **Write** | 11.53 ms<br>867.3 Mo/s<br>6.462 GB/s<br>1.153 ns | **7.70 ms<br>1299 Mo/s<br>9.676 GB/s<br>0.770 ns** | 15.31 s<br>653.1 Ko/s<br>4.983 MB/s<br>1.531 µs | 12.50 s<br>800.2 Ko/s<br>6.105 MB/s<br>1.250 µs | 3.39 s<br>2.951 Mo/s<br>22.51 MB/s<br>338.9 ns | 7.06 s<br>1.417 Mo/s<br>10.81 MB/s<br>705.9 ns | 4.38 s<br>2.282 Mo/s<br>17.41 MB/s<br>438.3 ns |
| **Linear** | 12.78 ms<br>782.2 Mo/s<br>5.828 GB/s<br>1.278 ns | **6.64 ms<br>1506 Mo/s<br>11.22 GB/s<br>0.664 ns** | 706.95 ms<br>14.15 Mo/s<br>107.9 MB/s<br>70.69 ns | 740.61 ms<br>13.50 Mo/s<br>103.0 MB/s<br>74.06 ns | 293.47 ms<br>34.07 Mo/s<br>260.0 MB/s<br>29.35 ns | 91.73 ms<br>109.0 Mo/s<br>831.7 MB/s<br>9.173 ns | 1.07 s<br>9.316 Mo/s<br>71.08 MB/s<br>107.3 ns |
| **Random** | 172.55 ms<br>579.5 Ko/s<br>4.421 MB/s<br>1.726 µs | **0.05 ms<br>2202 Mo/s<br>16.40 GB/s<br>0.454 ns** | 190.02 ms<br>526.2 Ko/s<br>4.015 MB/s<br>1.900 µs | 418.94 ms<br>238.7 Ko/s<br>1.821 MB/s<br>4.189 µs | 108.29 ms<br>923.4 Ko/s<br>7.045 MB/s<br>1.083 µs | 96.08 ms<br>1.041 Mo/s<br>7.940 MB/s<br>960.8 ns | 599.77 ms<br>166.7 Ko/s<br>1.272 MB/s<br>5.998 µs |
| **Random Rayon** | 27.63 ms<br>3.620 Mo/s<br>27.62 MB/s<br>276.3 ns | **2.20 ms<br>45.53 Mo/s<br>347.4 MB/s<br>21.96 ns** | 40.52 ms<br>2.468 Mo/s<br>18.83 MB/s<br>405.2 ns | 143.61 ms<br>696.3 Ko/s<br>5.313 MB/s<br>1.436 µs | 15.47 ms<br>6.464 Mo/s<br>49.31 MB/s<br>154.7 ns | 17.37 ms<br>5.755 Mo/s<br>43.91 MB/s<br>173.7 ns | 153.07 ms<br>653.3 Ko/s<br>4.984 MB/s<br>1.531 µs |
| **Disk Size** | **392.00 KB** | 128.00 MB | 209.45 MB | 137.11 MB | 514.00 MB | 367.13 MB | 244.94 MB |
---

## Benchmark 2

**Test**: 100 million sequential u64 writes, linear reads, and 1% random reads.

**Iterations**: 10 passes

### Results

| Metric | vecdb_compressed | vecdb_raw | vecdb_raw_old | fjall3 | fjall2 | redb | lmdb |
|--------|--------|--------|--------|--------|--------|--------|--------|
| **Open** | 0.56 ms | 0.15 ms | **0.10 ms** | 243.52 ms | 32.55 ms | 5.64 ms | 0.16 ms |
| **Write** | **87.82 ms<br>1139 Mo/s<br>8.484 GB/s<br>0.878 ns** | 88.53 ms<br>1130 Mo/s<br>8.416 GB/s<br>0.885 ns | 119.51 ms<br>836.8 Mo/s<br>6.234 GB/s<br>1.195 ns | 2m 33.66s<br>650.8 Ko/s<br>4.965 MB/s<br>1.537 µs | 2m 5.70s<br>795.5 Ko/s<br>6.069 MB/s<br>1.257 µs | 1m 1.47s<br>1.627 Mo/s<br>12.41 MB/s<br>614.7 ns | 2m 38.76s<br>629.9 Ko/s<br>4.806 MB/s<br>1.588 µs |
| **Linear** | 126.00 ms<br>793.7 Mo/s<br>5.913 GB/s<br>1.260 ns | **72.72 ms<br>1375 Mo/s<br>10.25 GB/s<br>0.727 ns** | 387.86 ms<br>257.8 Mo/s<br>1.921 GB/s<br>3.879 ns | 7.73 s<br>12.94 Mo/s<br>98.72 MB/s<br>77.28 ns | 7.64 s<br>13.09 Mo/s<br>99.89 MB/s<br>76.38 ns | 4.99 s<br>20.04 Mo/s<br>152.9 MB/s<br>49.89 ns | 4.69 s<br>21.32 Mo/s<br>162.7 MB/s<br>46.91 ns |
| **Random** | 1.74 s<br>575.1 Ko/s<br>4.388 MB/s<br>1.739 µs | 0.42 ms<br>2382 Mo/s<br>17.74 GB/s<br>0.420 ns | **0.41 ms<br>2412 Mo/s<br>17.97 GB/s<br>0.415 ns** | 2.51 s<br>398.0 Ko/s<br>3.037 MB/s<br>2.513 µs | 6.37 s<br>157.0 Ko/s<br>1.198 MB/s<br>6.369 µs | 6.74 s<br>148.5 Ko/s<br>1.133 MB/s<br>6.735 µs | 1.33 s<br>751.8 Ko/s<br>5.736 MB/s<br>1.330 µs |
| **Random Rayon** | 252.32 ms<br>3.963 Mo/s<br>30.24 MB/s<br>252.3 ns | 130.78 ms<br>7.646 Mo/s<br>58.34 MB/s<br>130.8 ns | **38.08 ms<br>26.26 Mo/s<br>200.3 MB/s<br>38.08 ns** | 453.87 ms<br>2.203 Mo/s<br>16.81 MB/s<br>453.9 ns | 1.31 s<br>763.3 Ko/s<br>5.824 MB/s<br>1.310 µs | 462.10 ms<br>2.164 Mo/s<br>16.51 MB/s<br>462.1 ns | 176.35 ms<br>5.671 Mo/s<br>43.26 MB/s<br>176.3 ns |
| **Disk Size** | **3.01 MB** | 1.00 GB | 1.00 GB | 1.78 GB | 1.00 GB | 3.03 GB | 3.58 GB |

## Run

```bash
cargo run --release --bin vecdb_bench
```
