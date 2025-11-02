# VecDB Benchmark

Benchmark comparing vecdb against popular embedded databases: fjall, redb, and lmdb.

## Benchmark 1

**Test**: 10 million sequential u64 writes, linear reads, and 1% random reads.

**Iterations**: 10 passes

### Results

| Metric | vecdb_compressed | vecdb_raw | fjall3 | fjall2 | redb | lmdb | rocksdb |
|--------|--------|--------|--------|--------|--------|--------|--------|
| **Open** | 0.13 ms | **0.07 ms** | 1.03 s | 634.04 ms | 5.73 ms | 0.15 ms | 2.17 ms |
| **Write** | 9.47 ms<br>1056 Mo/s<br>7.868 GB/s<br>0.947 ns | **7.96 ms<br>1256 Mo/s<br>9.360 GB/s<br>0.796 ns** | 16.57 s<br>603.5 Ko/s<br>4.604 MB/s<br>1.657 µs | 13.42 s<br>744.9 Ko/s<br>5.683 MB/s<br>1.342 µs | 5.57 s<br>1.797 Mo/s<br>13.71 MB/s<br>556.6 ns | 7.20 s<br>1.390 Mo/s<br>10.60 MB/s<br>719.5 ns | 4.38 s<br>2.284 Mo/s<br>17.43 MB/s<br>437.7 ns |
| **Linear** | 12.73 ms<br>785.7 Mo/s<br>5.854 GB/s<br>1.273 ns | **6.46 ms<br>1549 Mo/s<br>11.54 GB/s<br>0.646 ns** | 670.34 ms<br>14.92 Mo/s<br>113.8 MB/s<br>67.03 ns | 725.13 ms<br>13.79 Mo/s<br>105.2 MB/s<br>72.51 ns | 289.12 ms<br>34.59 Mo/s<br>263.9 MB/s<br>28.91 ns | 84.36 ms<br>118.5 Mo/s<br>904.3 MB/s<br>8.436 ns | 1.07 s<br>9.317 Mo/s<br>71.08 MB/s<br>107.3 ns |
| **Random** | 171.23 ms<br>584.0 Ko/s<br>4.456 MB/s<br>1.712 µs | **0.04 ms<br>2295 Mo/s<br>17.10 GB/s<br>0.436 ns** | 184.36 ms<br>542.4 Ko/s<br>4.138 MB/s<br>1.844 µs | 413.82 ms<br>241.7 Ko/s<br>1.844 MB/s<br>4.138 µs | 104.64 ms<br>955.6 Ko/s<br>7.291 MB/s<br>1.046 µs | 93.47 ms<br>1.070 Mo/s<br>8.163 MB/s<br>934.7 ns | 585.72 ms<br>170.7 Ko/s<br>1.303 MB/s<br>5.857 µs |
| **Random Rayon** | 24.01 ms<br>4.165 Mo/s<br>31.77 MB/s<br>240.1 ns | **2.24 ms<br>44.69 Mo/s<br>340.9 MB/s<br>22.38 ns** | 42.38 ms<br>2.360 Mo/s<br>18.00 MB/s<br>423.8 ns | 151.16 ms<br>661.6 Ko/s<br>5.047 MB/s<br>1.512 µs | 15.64 ms<br>6.394 Mo/s<br>48.78 MB/s<br>156.4 ns | 17.10 ms<br>5.848 Mo/s<br>44.62 MB/s<br>171.0 ns | 153.77 ms<br>650.3 Ko/s<br>4.962 MB/s<br>1.538 µs |
| **Disk Size** | **392.00 KB** | 128.00 MB | 209.46 MB | 137.11 MB | 514.00 MB | 367.13 MB | 244.94 MB |
---

## Benchmark 2

**Test**: 100 million sequential u64 writes, linear reads, and 1% random reads.

**Iterations**: 10 passes

### Results

| Metric | vecdb_compressed | vecdb_raw | fjall3 | fjall2 | redb | lmdb |
|--------|--------|--------|--------|--------|--------|--------|
| **Open** | 0.30 ms | **0.11 ms** | 248.82 ms | 33.81 ms | 5.37 ms | 0.19 ms |
| **Write** | 87.54 ms<br>1142 Mo/s<br>8.511 GB/s<br>0.875 ns | **69.72 ms<br>1434 Mo/s<br>10.69 GB/s<br>0.697 ns** | 2m 38.87s<br>629.4 Ko/s<br>4.802 MB/s<br>1.589 µs | 2m 10.34s<br>767.2 Ko/s<br>5.853 MB/s<br>1.303 µs | 1m 2.69s<br>1.595 Mo/s<br>12.17 MB/s<br>626.9 ns | 2m 41.87s<br>617.8 Ko/s<br>4.713 MB/s<br>1.619 µs |
| **Linear** | 128.39 ms<br>778.9 Mo/s<br>5.803 GB/s<br>1.284 ns | **68.17 ms<br>1467 Mo/s<br>10.93 GB/s<br>0.682 ns** | 7.80 s<br>12.82 Mo/s<br>97.85 MB/s<br>77.97 ns | 7.82 s<br>12.79 Mo/s<br>97.60 MB/s<br>78.17 ns | 3.85 s<br>25.94 Mo/s<br>197.9 MB/s<br>38.55 ns | 1.81 s<br>55.23 Mo/s<br>421.3 MB/s<br>18.11 ns |
| **Random** | 1.80 s<br>557.1 Ko/s<br>4.250 MB/s<br>1.795 µs | **0.42 ms<br>2358 Mo/s<br>17.57 GB/s<br>0.424 ns** | 2.99 s<br>334.7 Ko/s<br>2.553 MB/s<br>2.988 µs | 5.89 s<br>169.7 Ko/s<br>1.295 MB/s<br>5.893 µs | 2.74 s<br>364.6 Ko/s<br>2.781 MB/s<br>2.743 µs | 1.40 s<br>712.5 Ko/s<br>5.436 MB/s<br>1.404 µs |
| **Random Rayon** | 255.25 ms<br>3.918 Mo/s<br>29.89 MB/s<br>255.2 ns | **41.42 ms<br>24.14 Mo/s<br>184.2 MB/s<br>41.42 ns** | 643.86 ms<br>1.553 Mo/s<br>11.85 MB/s<br>643.9 ns | 1.34 s<br>743.6 Ko/s<br>5.674 MB/s<br>1.345 µs | 396.31 ms<br>2.523 Mo/s<br>19.25 MB/s<br>396.3 ns | 226.97 ms<br>4.406 Mo/s<br>33.61 MB/s<br>227.0 ns |
| **Disk Size** | **3.01 MB** | 1.00 GB | 1.78 GB | 1.00 GB | 3.03 GB | 3.58 GB |
---

## Benchmark 3

**Test**: 1000 million sequential u64 writes, linear reads, and 1% random reads.

**Iterations**: 10 passes

### Results

| Metric | vecdb_raw_old | vecdb_raw | vecdb_compressed |
|--------|--------|--------|--------|
| **Open** | 0.78 ms | **0.08 ms** | 1.91 ms |
| **Write** | **890.70 ms<br>1123 Mo/s<br>8.365 GB/s<br>0.891 ns** | 992.39 ms<br>1008 Mo/s<br>7.508 GB/s<br>0.992 ns | 915.19 ms<br>1093 Mo/s<br>8.141 GB/s<br>0.915 ns |
| **Linear** | 3.79 s<br>263.6 Mo/s<br>1.964 GB/s<br>3.794 ns | **677.87 ms<br>1475 Mo/s<br>10.99 GB/s<br>0.678 ns** | 1.26 s<br>796.6 Mo/s<br>5.935 GB/s<br>1.255 ns |
| **Random** | 5.16 ms<br>1940 Mo/s<br>14.45 GB/s<br>0.516 ns | **4.19 ms<br>2388 Mo/s<br>17.79 GB/s<br>0.419 ns** | 18.57 s<br>538.5 Ko/s<br>4.108 MB/s<br>1.857 µs |
| **Random Rayon** | 514.28 ms<br>19.44 Mo/s<br>148.4 MB/s<br>51.43 ns | **201.92 ms<br>49.53 Mo/s<br>377.8 MB/s<br>20.19 ns** | 2.40 s<br>4.171 Mo/s<br>31.82 MB/s<br>239.8 ns |
| **Disk Size** | 8.00 GB | 8.00 GB | **40.01 MB** |

## Run

```bash
cargo run --release --bin vecdb_bench
```
