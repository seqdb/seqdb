# VecDB Benchmark

Benchmark comparing vecdb against popular embedded databases: fjall, redb, and lmdb.

## Benchmark 1

**Test**: 10 million sequential u64 writes, linear reads, and 1% random reads.

**Iterations**: 10 passes

### Results

| Metric | vecdb_compressed | vecdb_raw | fjall3 | fjall2 | redb | lmdb | rocksdb |
|--------|--------|--------|--------|--------|--------|--------|--------|
| **Open** | 0.12 ms | **0.07 ms** | 1.01 s | 633.16 ms | 5.76 ms | 0.14 ms | 2.22 ms |
| **Write** | 13.25 ms<br>754.6 Mo/s<br>5.622 GB/s<br>1.325 ns | **8.03 ms<br>1246 Mo/s<br>9.280 GB/s<br>0.803 ns** | 15.84 s<br>631.4 Ko/s<br>4.817 MB/s<br>1.584 µs | 12.83 s<br>779.7 Ko/s<br>5.949 MB/s<br>1.283 µs | 5.67 s<br>1.764 Mo/s<br>13.46 MB/s<br>567.0 ns | 7.10 s<br>1.408 Mo/s<br>10.74 MB/s<br>710.1 ns | 4.40 s<br>2.275 Mo/s<br>17.35 MB/s<br>439.6 ns |
| **Linear** | 12.76 ms<br>783.7 Mo/s<br>5.839 GB/s<br>1.276 ns | **6.45 ms<br>1550 Mo/s<br>11.55 GB/s<br>0.645 ns** | 700.09 ms<br>14.28 Mo/s<br>109.0 MB/s<br>70.01 ns | 732.06 ms<br>13.66 Mo/s<br>104.2 MB/s<br>73.21 ns | 288.11 ms<br>34.71 Mo/s<br>264.8 MB/s<br>28.81 ns | 82.08 ms<br>121.8 Mo/s<br>929.5 MB/s<br>8.208 ns | 1.07 s<br>9.387 Mo/s<br>71.62 MB/s<br>106.5 ns |
| **Random** | 171.23 ms<br>584.0 Ko/s<br>4.456 MB/s<br>1.712 µs | **0.05 ms<br>2189 Mo/s<br>16.31 GB/s<br>0.457 ns** | 182.94 ms<br>546.6 Ko/s<br>4.171 MB/s<br>1.829 µs | 415.20 ms<br>240.8 Ko/s<br>1.838 MB/s<br>4.152 µs | 106.32 ms<br>940.5 Ko/s<br>7.176 MB/s<br>1.063 µs | 94.18 ms<br>1.062 Mo/s<br>8.101 MB/s<br>941.8 ns | 586.63 ms<br>170.5 Ko/s<br>1.301 MB/s<br>5.866 µs |
| **Random Rayon** | 25.93 ms<br>3.857 Mo/s<br>29.43 MB/s<br>259.3 ns | **2.15 ms<br>46.44 Mo/s<br>354.3 MB/s<br>21.53 ns** | 38.88 ms<br>2.572 Mo/s<br>19.62 MB/s<br>388.8 ns | 146.08 ms<br>684.6 Ko/s<br>5.223 MB/s<br>1.461 µs | 15.82 ms<br>6.322 Mo/s<br>48.23 MB/s<br>158.2 ns | 16.67 ms<br>5.997 Mo/s<br>45.76 MB/s<br>166.7 ns | 153.85 ms<br>650.0 Ko/s<br>4.959 MB/s<br>1.539 µs |
| **Disk Size** | **392.00 KB** | 128.00 MB | 209.46 MB | 137.11 MB | 514.00 MB | 367.13 MB | 244.94 MB |
---

## Benchmark 2

**Test**: 100 million sequential u64 writes, linear reads, and 1% random reads.

**Iterations**: 10 passes

### Results

| Metric | vecdb_compressed | vecdb_raw | fjall3 | fjall2 | redb | lmdb |
|--------|--------|--------|--------|--------|--------|--------|
| **Open** | 0.33 ms | **0.11 ms** | 239.45 ms | 32.15 ms | 5.45 ms | 0.17 ms |
| **Write** | 86.82 ms<br>1152 Mo/s<br>8.582 GB/s<br>0.868 ns | **73.26 ms<br>1365 Mo/s<br>10.17 GB/s<br>0.733 ns** | 2m 33.66s<br>650.8 Ko/s<br>4.965 MB/s<br>1.537 µs | 2m 9.05s<br>774.9 Ko/s<br>5.912 MB/s<br>1.291 µs | 1m 3.13s<br>1.584 Mo/s<br>12.09 MB/s<br>631.3 ns | 2m 36.96s<br>637.1 Ko/s<br>4.861 MB/s<br>1.570 µs |
| **Linear** | 127.62 ms<br>783.6 Mo/s<br>5.838 GB/s<br>1.276 ns | **67.17 ms<br>1489 Mo/s<br>11.09 GB/s<br>0.672 ns** | 7.72 s<br>12.96 Mo/s<br>98.86 MB/s<br>77.17 ns | 7.66 s<br>13.06 Mo/s<br>99.64 MB/s<br>76.57 ns | 3.90 s<br>25.66 Mo/s<br>195.8 MB/s<br>38.97 ns | 1.94 s<br>51.60 Mo/s<br>393.6 MB/s<br>19.38 ns |
| **Random** | 1.75 s<br>570.5 Ko/s<br>4.352 MB/s<br>1.753 µs | **0.42 ms<br>2371 Mo/s<br>17.67 GB/s<br>0.422 ns** | 2.66 s<br>375.7 Ko/s<br>2.867 MB/s<br>2.661 µs | 5.88 s<br>170.0 Ko/s<br>1.297 MB/s<br>5.883 µs | 2.91 s<br>343.2 Ko/s<br>2.618 MB/s<br>2.914 µs | 1.72 s<br>579.8 Ko/s<br>4.424 MB/s<br>1.725 µs |
| **Random Rayon** | 237.85 ms<br>4.204 Mo/s<br>32.08 MB/s<br>237.9 ns | **52.74 ms<br>18.96 Mo/s<br>144.7 MB/s<br>52.74 ns** | 514.91 ms<br>1.942 Mo/s<br>14.82 MB/s<br>514.9 ns | 1.30 s<br>767.1 Ko/s<br>5.852 MB/s<br>1.304 µs | 466.86 ms<br>2.142 Mo/s<br>16.34 MB/s<br>466.9 ns | 192.74 ms<br>5.188 Mo/s<br>39.58 MB/s<br>192.7 ns |
| **Disk Size** | **3.01 MB** | 1.00 GB | 1.78 GB | 1.00 GB | 3.03 GB | 3.58 GB |
---

## Benchmark 3

**Test**: 1000 million sequential u64 writes, linear reads, and 1% random reads.

**Iterations**: 10 passes

### Results

| Metric | vecdb_raw_old | vecdb_raw | vecdb_compressed |
|--------|--------|--------|--------|
| **Open** | 0.76 ms | **0.07 ms** | 1.86 ms |
| **Write** | **876.30 ms<br>1141 Mo/s<br>8.502 GB/s<br>0.876 ns** | 893.17 ms<br>1120 Mo/s<br>8.342 GB/s<br>0.893 ns | 885.74 ms<br>1129 Mo/s<br>8.412 GB/s<br>0.886 ns |
| **Linear** | 3.88 s<br>257.9 Mo/s<br>1.921 GB/s<br>3.878 ns | **673.92 ms<br>1484 Mo/s<br>11.06 GB/s<br>0.674 ns** | 1.25 s<br>801.7 Mo/s<br>5.973 GB/s<br>1.247 ns |
| **Random** | 5.16 ms<br>1937 Mo/s<br>14.43 GB/s<br>0.516 ns | **4.17 ms<br>2396 Mo/s<br>17.85 GB/s<br>0.417 ns** | 18.30 s<br>546.5 Ko/s<br>4.170 MB/s<br>1.830 µs |
| **Random Rayon** | 434.66 ms<br>23.01 Mo/s<br>175.5 MB/s<br>43.47 ns | **202.36 ms<br>49.42 Mo/s<br>377.0 MB/s<br>20.24 ns** | 2.41 s<br>4.147 Mo/s<br>31.64 MB/s<br>241.2 ns |
| **Disk Size** | 8.00 GB | 8.00 GB | **40.01 MB** |

## Run

```bash
cargo run --release --bin vecdb_bench
```
