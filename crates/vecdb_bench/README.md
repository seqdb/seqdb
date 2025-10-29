# VecDB Benchmark

Benchmark comparing vecdb against popular embedded databases: fjall, redb, and lmdb.

## System Information

- **CPU**: Apple M3 Pro
- **CPU Cores**: 12
- **Total Memory**: 36.00 GB
- **OS**: Darwin 26.0.1

## Benchmark 1

**Test**: 100 million sequential u64 writes, linear reads, and 1 million random reads.

**Iterations**: 1 pass

**Random Seed**: 42

**Databases**: vecdb, vecdb_old, fjall2, fjall3, redb, lmdb, rocksdb

### Results

| Metric | vecdb | vecdb_old | fjall2 | fjall3 | redb | lmdb | rocksdb |
|--------|--------|--------|--------|--------|--------|--------|--------|
| **Open** | 0.25 ms | **0.22 ms** | 22.08 ms | 8.61 ms | 11.22 ms | 0.40 ms | 2.90 ms |
| **Write** | **933.28 M ops/s<br>6.95 GB/s<br>1.07 ns** | 929.91 M ops/s<br>6.93 GB/s<br>1.08 ns | 16.65 M ops/s<br>127.05 MB/s<br>60.05 ns | 21.75 M ops/s<br>165.90 MB/s<br>45.99 ns | 1.14 M ops/s<br>8.71 MB/s<br>876.35 ns | 645.32 K ops/s<br>4.92 MB/s<br>1.55 µs | 2.16 M ops/s<br>16.51 MB/s<br>461.98 ns |
| **len()** | **0.00 ms** | 0.00 ms | 6.78 s | 6.75 s | 0.01 ms | 0.00 ms | 13.50 s |
| **Linear Read** | **1553.99 M ops/s<br>11.58 GB/s<br>0.64 ns** | 234.43 M ops/s<br>1.75 GB/s<br>4.27 ns | 14.30 M ops/s<br>109.09 MB/s<br>69.93 ns | 14.40 M ops/s<br>109.84 MB/s<br>69.46 ns | 30.39 M ops/s<br>231.84 MB/s<br>32.91 ns | 103.67 M ops/s<br>790.91 MB/s<br>9.65 ns | 7.41 M ops/s<br>56.53 MB/s<br>134.95 ns |
| **Seq 2t** | **2859.74 M ops/s<br>21.31 GB/s<br>0.35 ns** | 80.72 M ops/s<br>615.81 MB/s<br>12.39 ns | 4.02 M ops/s<br>30.69 MB/s<br>248.58 ns | 2.04 M ops/s<br>15.53 MB/s<br>491.13 ns | 9.28 M ops/s<br>70.76 MB/s<br>107.81 ns | 2.04 M ops/s<br>15.53 MB/s<br>491.26 ns | 459.82 K ops/s<br>3.51 MB/s<br>2.17 µs |
| **Seq 4t** | **5201.01 M ops/s<br>38.75 GB/s<br>0.19 ns** | 45.48 M ops/s<br>347.00 MB/s<br>21.99 ns | 5.36 M ops/s<br>40.89 MB/s<br>186.56 ns | 3.17 M ops/s<br>24.21 MB/s<br>315.12 ns | 11.21 M ops/s<br>85.50 MB/s<br>89.23 ns | 3.99 M ops/s<br>30.42 MB/s<br>250.82 ns | 855.48 K ops/s<br>6.53 MB/s<br>1.17 µs |
| **Seq 8t** | **7003.31 M ops/s<br>52.18 GB/s<br>0.14 ns** | 9.39 M ops/s<br>71.61 MB/s<br>106.55 ns | 2.73 M ops/s<br>20.82 MB/s<br>366.48 ns | 3.82 M ops/s<br>29.17 MB/s<br>261.54 ns | 6.23 M ops/s<br>47.49 MB/s<br>160.64 ns | 6.38 M ops/s<br>48.71 MB/s<br>156.63 ns | 1.24 M ops/s<br>9.47 MB/s<br>805.32 ns |
| **Random 1t** | 179.18 K ops/s<br>1.37 MB/s<br>5.58 µs | **2363.37 M ops/s<br>17.61 GB/s<br>0.42 ns** | 96.26 K ops/s<br>0.73 MB/s<br>10.39 µs | 49.55 K ops/s<br>0.38 MB/s<br>20.18 µs | 8.42 K ops/s<br>0.06 MB/s<br>118.78 µs | 45.25 K ops/s<br>0.35 MB/s<br>22.10 µs | 62.09 K ops/s<br>0.47 MB/s<br>16.10 µs |
| **Random 4t** | 636.80 K ops/s<br>4.86 MB/s<br>1.57 µs | **1.05 M ops/s<br>8.01 MB/s<br>952.43 ns** | 359.61 K ops/s<br>2.74 MB/s<br>2.78 µs | 343.72 K ops/s<br>2.62 MB/s<br>2.91 µs | 23.12 K ops/s<br>0.18 MB/s<br>43.25 µs | 193.96 K ops/s<br>1.48 MB/s<br>5.16 µs | 340.02 K ops/s<br>2.59 MB/s<br>2.94 µs |
| **Random 8t** | 795.56 K ops/s<br>6.07 MB/s<br>1.26 µs | **1.85 M ops/s<br>14.10 MB/s<br>540.97 ns** | 557.32 K ops/s<br>4.25 MB/s<br>1.79 µs | 616.32 K ops/s<br>4.70 MB/s<br>1.62 µs | 39.75 K ops/s<br>0.30 MB/s<br>25.16 µs | 575.03 K ops/s<br>4.39 MB/s<br>1.74 µs | 604.04 K ops/s<br>4.61 MB/s<br>1.66 µs |
| **Random 12t** | 856.77 K ops/s<br>6.54 MB/s<br>1.17 µs | 2.47 M ops/s<br>18.86 MB/s<br>404.59 ns | 604.03 K ops/s<br>4.61 MB/s<br>1.66 µs | 786.00 K ops/s<br>6.00 MB/s<br>1.27 µs | 237.79 K ops/s<br>1.81 MB/s<br>4.21 µs | **2.77 M ops/s<br>21.14 MB/s<br>360.94 ns** | 972.32 K ops/s<br>7.42 MB/s<br>1.03 µs |
| **Random 16t** | 913.08 K ops/s<br>6.97 MB/s<br>1.10 µs | **34.36 M ops/s<br>262.18 MB/s<br>29.10 ns** | 768.52 K ops/s<br>5.86 MB/s<br>1.30 µs | 2.16 M ops/s<br>16.47 MB/s<br>463.29 ns | 619.08 K ops/s<br>4.72 MB/s<br>1.62 µs | 5.45 M ops/s<br>41.55 MB/s<br>183.62 ns | 1.27 M ops/s<br>9.67 MB/s<br>789.23 ns |
| **Disk Size** | 1.00 GB | 1.00 GB | **916.32 MB** | 1.50 GB | 3.03 GB | 3.58 GB | 1.03 GB |
---

## Benchmark 2

**Test**: 1000 million sequential u64 writes, linear reads, and 1 million random reads.

**Iterations**: 1 pass

**Random Seed**: 21

**Databases**: vecdb, vecdb_old

### Results

| Metric | vecdb | vecdb_old |
|--------|--------|--------|
| **Open** | 0.67 ms | **0.15 ms** |
| **Write** | 882.13 M ops/s<br>6.57 GB/s<br>1.13 ns | **883.60 M ops/s<br>6.58 GB/s<br>1.13 ns** |
| **len()** | 0.00 ms | **0.00 ms** |
| **Linear Read** | **825.77 M ops/s<br>6.15 GB/s<br>1.21 ns** | 195.12 M ops/s<br>1.45 GB/s<br>5.13 ns |
| **Seq 2t** | **2932.93 M ops/s<br>21.85 GB/s<br>0.34 ns** | 81.13 M ops/s<br>618.99 MB/s<br>12.33 ns |
| **Seq 4t** | **4246.03 M ops/s<br>31.64 GB/s<br>0.24 ns** | 42.37 M ops/s<br>323.27 MB/s<br>23.60 ns |
| **Seq 8t** | **8199.72 M ops/s<br>61.09 GB/s<br>0.12 ns** | 10.05 M ops/s<br>76.69 MB/s<br>99.49 ns |
| **Random 1t** | 150.58 K ops/s<br>1.15 MB/s<br>6.64 µs | **1980.69 M ops/s<br>14.76 GB/s<br>0.50 ns** |
| **Random 4t** | 1.16 M ops/s<br>8.84 MB/s<br>863.49 ns | **8.67 M ops/s<br>66.17 MB/s<br>115.31 ns** |
| **Random 8t** | 2.02 M ops/s<br>15.38 MB/s<br>496.19 ns | **6.91 M ops/s<br>52.71 MB/s<br>144.73 ns** |
| **Random 12t** | 1.27 M ops/s<br>9.67 MB/s<br>789.35 ns | **5.94 M ops/s<br>45.28 MB/s<br>168.48 ns** |
| **Random 16t** | 1.21 M ops/s<br>9.23 MB/s<br>826.67 ns | **5.36 M ops/s<br>40.91 MB/s<br>186.47 ns** |
| **Disk Size** | **8.00 GB** | **8.00 GB** |
---

## Benchmark 3

**Test**: 10000 million sequential u64 writes, linear reads, and 1 million random reads.

**Iterations**: 1 pass

**Random Seed**: 128

**Databases**: vecdb, vecdb_old

### Results

| Metric | vecdb | vecdb_old |
|--------|--------|--------|
| **Open** | 2.25 ms | **0.66 ms** |
| **Write** | 802.55 M ops/s<br>5.98 GB/s<br>1.25 ns | **820.48 M ops/s<br>6.11 GB/s<br>1.22 ns** |
| **len()** | 0.00 ms | **0.00 ms** |
| **Linear Read** | **814.61 M ops/s<br>6.07 GB/s<br>1.23 ns** | 63.27 M ops/s<br>482.69 MB/s<br>15.81 ns |
| **Seq 2t** | **691.85 M ops/s<br>5.15 GB/s<br>1.45 ns** | 59.79 M ops/s<br>456.19 MB/s<br>16.72 ns |
| **Seq 4t** | **712.22 M ops/s<br>5.31 GB/s<br>1.40 ns** | 46.64 M ops/s<br>355.87 MB/s<br>21.44 ns |
| **Seq 8t** | **799.71 M ops/s<br>5.96 GB/s<br>1.25 ns** | 14.61 M ops/s<br>111.46 MB/s<br>68.45 ns |
| **Random 1t** | 9.85 K ops/s<br>0.08 MB/s<br>101.51 µs | **2220.78 M ops/s<br>16.55 GB/s<br>0.45 ns** |
| **Random 4t** | 38.00 K ops/s<br>0.29 MB/s<br>26.32 µs | **41.86 K ops/s<br>0.32 MB/s<br>23.89 µs** |
| **Random 8t** | **72.02 K ops/s<br>0.55 MB/s<br>13.88 µs** | 71.29 K ops/s<br>0.54 MB/s<br>14.03 µs |
| **Random 12t** | **95.73 K ops/s<br>0.73 MB/s<br>10.45 µs** | 94.72 K ops/s<br>0.72 MB/s<br>10.56 µs |
| **Random 16t** | **122.02 K ops/s<br>0.93 MB/s<br>8.20 µs** | 110.88 K ops/s<br>0.85 MB/s<br>9.02 µs |
| **Disk Size** | **128.00 GB** | **128.00 GB** |

## Run

```bash
cargo run --release --bin vecdb_bench
```
